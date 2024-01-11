use crate::map::{BuildingID, Map, PathKind};
use crate::map_dynamic::{Itinerary, ParkingManagement, ParkingReserveError, SpotReservation};
use crate::transportation::TransportGrid;
use crate::transportation::{put_pedestrian_in_transport_grid, unpark, Location, VehicleState};
use crate::utils::resources::Resources;
use crate::world::{HumanEnt, HumanID, VehicleEnt, VehicleID};
use crate::{ParCommandBuffer, World};
use egui_inspect::Inspect;
use geom::{Spline3, Transform, Vec3};
use serde::{Deserialize, Serialize};
use slotmapd::HopSlotMap;

#[derive(Inspect, Serialize, Deserialize)]
pub struct Router {
    steps: Vec<RoutingStep>,
    cur_step: Option<RoutingStep>,
    pub target_dest: Option<Destination>,
    cur_dest: Option<Destination>,
    vehicle: Option<VehicleID>,
    pub personal_car: Option<VehicleID>,
    pub last_error: Option<RouterError>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RouterError {
    ReservingParkingSpot(ParkingReserveError),
    TranslatingParkingSpotToDrivePos,
    LocatingVehicle,
}

debug_inspect_impl!(RouterError);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    Outside(Vec3),
    Building(BuildingID),
}

debug_inspect_impl!(Destination);

#[derive(Debug, Serialize, Deserialize)]
pub enum RoutingStep {
    WalkTo(Vec3),
    DriveTo(VehicleID, Vec3),
    Park(VehicleID, Option<SpotReservation>),
    Unpark(VehicleID),
    GetInVehicle(VehicleID),
    GetOutVehicle(VehicleID),
    GetInBuilding(BuildingID),
    GetOutBuilding(BuildingID),
}

debug_inspect_impl!(RoutingStep);

pub fn routing_changed_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("map_dynamic::routing_changed_system");
    let map: &Map = &resources.read();
    let parking: &mut ParkingManagement = &mut resources.write();

    world.humans.values_mut().for_each(|h| {
        let router = &mut h.router;
        let loc = &h.location;
        if router.cur_dest == router.target_dest {
            return;
        }
        let dest = unwrap_ret!(router.target_dest);

        router.clear_steps(parking);
        match dest {
            Destination::Outside(pos) => {
                router.steps = match router.steps_to(pos, parking, map, loc, &world.vehicles) {
                    Ok(x) => x,
                    Err(e) => {
                        router.last_error = Some(e);
                        return;
                    }
                };
            }
            Destination::Building(build) => {
                if let Location::Building(cur_build) = loc {
                    if *cur_build == build {
                        router.cur_dest = router.target_dest;
                        return;
                    }
                }

                let bobj = match map.buildings.get(build) {
                    Some(x) => x,
                    None => {
                        router.cur_dest = router.target_dest;
                        return;
                    }
                };
                let door_pos = bobj.door_pos;
                router.steps = match router.steps_to(door_pos, parking, map, loc, &world.vehicles) {
                    Ok(x) => x,
                    Err(e) => {
                        router.last_error = Some(e);
                        return;
                    }
                };
                router.steps.push(RoutingStep::GetInBuilding(build));
            }
        }

        router.cur_dest = router.target_dest;

        router.steps.reverse();
    });
}

pub fn routing_update_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("map_dynamic::routing_update_system");
    let map: &Map = &resources.read();
    let cbuf_human: &ParCommandBuffer<HumanEnt> = &resources.read();
    let cbuf_vehicle: &ParCommandBuffer<VehicleEnt> = &resources.read();

    world.humans.iter_mut().for_each(|(body, h)| {
        if h.router.cur_step.is_none() && h.router.steps.is_empty() {
            return;
        }

        let trans: &Transform = &h.trans;
        let itin: &Itinerary = &h.it;

        let pos = match h.location {
            Location::Outside => trans.pos,
            Location::Vehicle(id) => world
                .vehicles
                .get(id)
                .map(|x| x.trans.pos)
                .unwrap_or_else(|| trans.pos),
            Location::Building(id) => map
                .buildings()
                .get(id)
                .map(|b| b.door_pos)
                .unwrap_or_else(|| trans.pos),
        };

        let mut cur_step_over = true;

        if let Some(ref step) = h.router.cur_step {
            cur_step_over = match *step {
                RoutingStep::WalkTo(_) => itin.has_ended(0.0),
                RoutingStep::DriveTo(vehicle, _) => world
                    .vehicles
                    .get(vehicle)
                    .map(|x| &x.it)
                    .map(|x| x.has_ended(0.0))
                    .unwrap_or(true),
                RoutingStep::Park(vehicle, _) => world
                    .vehicles
                    .get(vehicle)
                    .map(|x| &x.vehicle)
                    .map(|x| matches!(x.state, VehicleState::Parked(_)))
                    .unwrap_or(true),
                RoutingStep::Unpark(_) => true,
                RoutingStep::GetInVehicle(_) => true,
                RoutingStep::GetOutVehicle(_) => true,
                RoutingStep::GetInBuilding(_) => true,
                RoutingStep::GetOutBuilding(_) => true,
            };
        }
        let mut next_step_ready = true;

        if let Some(step) = h.router.steps.last() {
            next_step_ready = match *step {
                RoutingStep::WalkTo(_) => true,
                RoutingStep::DriveTo(_, _) => true,
                RoutingStep::Park(_, _) => true,
                RoutingStep::Unpark(_) => true,
                RoutingStep::GetInVehicle(vehicle) => world
                    .vehicles
                    .get(vehicle)
                    .map(|v| v.trans.pos.is_close(pos, 3.0))
                    .unwrap_or(true),
                RoutingStep::GetOutVehicle(_) => true,
                RoutingStep::GetInBuilding(build) => map
                    .buildings()
                    .get(build)
                    .map(|b| b.door_pos.is_close(pos, 3.0))
                    .unwrap_or(true),
                RoutingStep::GetOutBuilding(_) => true,
            };
        }

        if !(next_step_ready && cur_step_over) {
            return;
        }

        h.router.cur_step = h.router.steps.pop();

        if let Some(ref mut next_step) = h.router.cur_step {
            match *next_step {
                RoutingStep::WalkTo(obj) => {
                    h.it = Itinerary::wait_for_reroute(PathKind::Pedestrian, obj);
                }
                RoutingStep::DriveTo(vehicle, obj) => {
                    let route = Itinerary::wait_for_reroute(PathKind::Vehicle, obj);
                    if let Some(x) = world.vehicles.get_mut(vehicle) {
                        x.it = route
                    }
                }
                RoutingStep::Park(vehicle, ref mut spot) => {
                    if let Some(spot_resa) = spot.take() {
                        if !spot_resa.exists(&map.parking) {
                            h.router.reset_dest();
                            return;
                        }

                        if let Some(vehicle) = world.vehicles.get_mut(vehicle) {
                            park(map, vehicle, spot_resa)
                        }
                    }
                }
                RoutingStep::Unpark(vehicle) => {
                    cbuf_vehicle.exec_ent(vehicle, move |sim| unpark(sim, vehicle));
                }
                RoutingStep::GetInVehicle(vehicle) => {
                    if !world.vehicles.contains_key(vehicle) {
                        h.router.reset_dest();
                        return;
                    }
                    h.location = Location::Vehicle(vehicle);
                    walk_inside(body, h, cbuf_human);
                }
                RoutingStep::GetOutVehicle(vehicle) => {
                    let pos = world
                        .vehicles
                        .get(vehicle)
                        .map(|v| v.trans)
                        .map(|vtrans| vtrans.pos + vtrans.dir.cross(Vec3::Z) * 2.0)
                        .unwrap_or(pos);
                    walk_outside(body, pos, cbuf_human, &mut h.location);
                }
                RoutingStep::GetInBuilding(build) => {
                    if !map.buildings().contains_key(build) {
                        h.router.reset_dest();
                        return;
                    }
                    h.location = Location::Building(build);
                    walk_inside(body, h, cbuf_human);
                }
                RoutingStep::GetOutBuilding(build) => {
                    let wpos = map
                        .buildings()
                        .get(build)
                        .map(|x| x.door_pos)
                        .unwrap_or(pos);
                    walk_outside(body, wpos, cbuf_human, &mut h.location);
                }
            }
        }
    })
}

fn walk_inside(body: HumanID, h: &mut HumanEnt, cbuf: &ParCommandBuffer<HumanEnt>) {
    if let Some(coll) = h.collider.take() {
        cbuf.exec_ent(body, coll.destroy());
    }
    h.speed.0 = 0.0;
}

fn walk_outside(body: HumanID, pos: Vec3, cbuf: &ParCommandBuffer<HumanEnt>, loc: &mut Location) {
    *loc = Location::Outside;
    cbuf.exec_ent(body, move |sim| {
        let coll = put_pedestrian_in_transport_grid(&mut sim.write::<TransportGrid>(), pos);
        let h = unwrap_ret!(sim.world.humans.get_mut(body));
        h.trans.pos = pos;
        h.collider = Some(coll);
    });
}

fn park(map: &Map, vehicle: &mut VehicleEnt, spot_resa: SpotReservation) {
    let trans = vehicle.trans;
    let spot = match spot_resa.get(&map.parking) {
        Some(x) => x,
        None => {
            log::warn!("Couldn't park at {:?} because it doesn't exist", spot_resa);
            return;
        }
    };

    let s = Spline3 {
        from: trans.pos,
        to: spot.trans.pos,
        from_derivative: trans.dir * 2.0,
        to_derivative: spot.trans.dir * 2.0,
    };

    vehicle.vehicle.state = VehicleState::RoadToPark(s, 0.0, spot_resa);
    vehicle.speed.0 = 0.0;
}

impl Router {
    pub fn new(personal_car: Option<VehicleID>) -> Self {
        Self {
            steps: vec![],
            cur_step: None,
            target_dest: None,
            personal_car,
            vehicle: personal_car,
            cur_dest: None,
            last_error: None,
        }
    }

    pub fn use_vehicle(&mut self, v: Option<VehicleID>) {
        self.vehicle = v;
    }

    pub(crate) fn clear_steps(&mut self, parking: &mut ParkingManagement) {
        for s in self.steps.drain(..).chain(self.cur_step.take()) {
            if let RoutingStep::Park(_, Some(spot)) = s {
                parking.free(spot);
            }
        }
    }

    pub fn reset_dest(&mut self) {
        self.cur_dest = None;
    }

    /// Returns wheter or not the destination was already attained
    pub fn go_to(&mut self, dest: Destination) -> bool {
        if let Some(router_dest) = self.cur_dest {
            if router_dest == dest {
                return self.steps.is_empty() && self.cur_step.is_none();
            }
        }
        self.target_dest = Some(dest);
        false
    }

    fn steps_to(
        &mut self,
        obj: Vec3,
        parking: &mut ParkingManagement,
        map: &Map,
        loc: &Location,
        cars: &HopSlotMap<VehicleID, VehicleEnt>,
    ) -> Result<Vec<RoutingStep>, RouterError> {
        let mut steps = vec![];
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }

        if let Some(car) = self.vehicle {
            let spot_resa = parking
                .reserve_near(obj, map)
                .map_err(RouterError::ReservingParkingSpot)?;
            let parking_pos = match spot_resa.park_pos(map) {
                Some(x) => x,
                None => {
                    parking.free(spot_resa);
                    return Err(RouterError::TranslatingParkingSpotToDrivePos);
                }
            };

            if !matches!(loc, Location::Vehicle(_)) {
                if let Some(pos) = cars.get(car).map(|x| x.trans.pos) {
                    steps.push(RoutingStep::WalkTo(pos));
                    steps.push(RoutingStep::GetInVehicle(car));
                    steps.push(RoutingStep::Unpark(car));
                } else {
                    parking.free(spot_resa);
                    self.vehicle = None;
                    return Err(RouterError::LocatingVehicle);
                }
            }

            steps.push(RoutingStep::DriveTo(car, parking_pos));
            steps.push(RoutingStep::Park(car, Some(spot_resa)));
            steps.push(RoutingStep::GetOutVehicle(car));
        }

        steps.push(RoutingStep::WalkTo(obj));
        Ok(steps)
    }
}
