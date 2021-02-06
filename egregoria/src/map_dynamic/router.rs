use crate::map_dynamic::{Itinerary, ParkingManagement};
use crate::pedestrians::{put_pedestrian_in_coworld, Location};
use crate::physics::{Collider, CollisionWorld, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::{put_vehicle_in_coworld, Vehicle, VehicleID, VehicleState};
use crate::{Egregoria, ParCommandBuffer};
use geom::{Spline, Transform, Vec2};
use imgui_inspect_derive::*;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore};
use map_model::{BuildingID, CarPath, Map, ParkingSpotID, PedestrianPath};
use serde::{Deserialize, Serialize};

#[derive(Clone, Inspect)]
pub struct Router {
    steps: Vec<RoutingStep>,
    cur_step: Option<RoutingStep>,
    dest: Option<Destination>,
    reroute: bool,
    vehicle: Option<VehicleID>,
    pub personal_car: Option<VehicleID>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    Outside(Vec2),
    Building(BuildingID),
}

debug_inspect_impl!(Destination);

#[derive(Copy, Clone, Debug)]
pub enum RoutingStep {
    WalkTo(Vec2),
    DriveTo(VehicleID, Vec2),
    Park(VehicleID, ParkingSpotID),
    Unpark(VehicleID),
    GetInVehicle(VehicleID),
    GetOutVehicle(VehicleID),
    GetInBuilding(BuildingID),
    GetOutBuilding(BuildingID),
}

debug_inspect_impl!(RoutingStep);

register_system!(routing_update);
#[system(par_for_each)]
#[read_component(Transform)]
#[read_component(Vehicle)]
#[read_component(Itinerary)]
pub fn routing_update(
    #[resource] map: &Map,
    #[resource] cbuf: &ParCommandBuffer,
    #[resource] parking: &ParkingManagement,
    body: &Entity,
    router: &mut Router,
    loc: &mut Location,
    mr: &mut MeshRender,
    kin: &mut Kinematics,
    subworld: &SubWorld,
) {
    let trans = *subworld
        .entry_ref(*body)
        .unwrap()
        .get_component::<Transform>()
        .unwrap();
    let pos = trans.position();
    if !router.reroute {
        let next_step = unwrap_or!(router.steps.last(), {
            router.cur_step = None;
            return;
        });

        let mut cur_step_over = true;

        if let Some(step) = router.cur_step {
            cur_step_over = match step {
                RoutingStep::WalkTo(_) => subworld
                    .entry_ref(*body)
                    .unwrap()
                    .get_component::<Itinerary>()
                    .unwrap()
                    .has_ended(0.0),
                RoutingStep::DriveTo(vehicle, _) => subworld
                    .entry_ref(vehicle.0)
                    .unwrap()
                    .get_component::<Itinerary>()
                    .unwrap()
                    .has_ended(0.0),
                RoutingStep::Park(vehicle, _) => matches!(
                    subworld
                        .entry_ref(vehicle.0)
                        .unwrap()
                        .get_component::<Vehicle>()
                        .unwrap()
                        .state,
                    VehicleState::Parked(_)
                ),
                RoutingStep::Unpark(_) => true,
                RoutingStep::GetInVehicle(_) => true,
                RoutingStep::GetOutVehicle(_) => true,
                RoutingStep::GetInBuilding(_) => true,
                RoutingStep::GetOutBuilding(_) => true,
            };
        }

        let next_step_ready = match next_step {
            RoutingStep::WalkTo(_) => true,
            RoutingStep::DriveTo(_, _) => true,
            RoutingStep::Park(_, _) => true,
            RoutingStep::Unpark(_) => true,
            RoutingStep::GetInVehicle(vehicle) => subworld
                .entry_ref(vehicle.0)
                .unwrap()
                .get_component::<Transform>()
                .unwrap()
                .position()
                .is_close(pos, 3.0),
            RoutingStep::GetOutVehicle(_) => true,
            &RoutingStep::GetInBuilding(build) => {
                map.buildings()[build].door_pos.is_close(pos, 3.0)
            }
            RoutingStep::GetOutBuilding(_) => true,
        };

        if !(next_step_ready && cur_step_over) {
            return;
        }

        router.cur_step = Some(router.steps.pop().unwrap());

        match router.cur_step.unwrap() {
            RoutingStep::WalkTo(obj) => {
                if let Some(route) = Itinerary::route(pos, obj, &*map, &PedestrianPath) {
                    cbuf.add_component(*body, route);
                }
            }
            RoutingStep::DriveTo(vehicle, obj) => {
                if let Some(route) = Itinerary::route(pos, obj, &*map, &CarPath) {
                    cbuf.add_component(vehicle.0, route);
                }
            }
            RoutingStep::Park(vehicle, spot) => {
                if !map.parking.contains(spot) {
                    router.reroute = true;
                    return;
                }

                cbuf.exec(park(vehicle, spot));
            }
            RoutingStep::Unpark(vehicle) => {
                cbuf.exec(unpark(vehicle));
            }
            RoutingStep::GetInVehicle(vehicle) => {
                *loc = Location::Vehicle(vehicle);
                walk_inside(*body, cbuf, mr, kin);
            }
            RoutingStep::GetOutVehicle(vehicle) => {
                let vtrans = *subworld
                    .entry_ref(vehicle.0)
                    .unwrap()
                    .get_component::<Transform>()
                    .unwrap();
                let pos = vtrans.position() + vtrans.direction().perpendicular() * 2.0;
                walk_outside(*body, pos, cbuf, mr, loc);
            }
            RoutingStep::GetInBuilding(build) => {
                *loc = Location::Building(build);
                walk_inside(*body, cbuf, mr, kin);
            }
            RoutingStep::GetOutBuilding(build) => {
                let wpos = map.buildings()[build].door_pos;
                walk_outside(*body, wpos, cbuf, mr, loc);
            }
        }
        return;
    }

    router.reroute = false;

    // router is dirty
    let dest = router.dest.expect("destination is empty but dirty is true");
    router.clear_steps(parking);
    match dest {
        Destination::Outside(pos) => {
            router.steps = router.steps_to(pos, parking, map, loc, subworld)
        }
        Destination::Building(build) => {
            if let Location::Building(cur_build) = loc {
                if *cur_build == build {
                    return;
                }
            }

            let door_pos = map.buildings()[build].door_pos;
            router.steps = router.steps_to(door_pos, parking, map, loc, subworld);
            router.steps.push(RoutingStep::GetInBuilding(build));
        }
    }

    router.steps.reverse();
}

fn walk_inside(body: Entity, cbuf: &ParCommandBuffer, mr: &mut MeshRender, kin: &mut Kinematics) {
    mr.hide = true;
    cbuf.remove_component::<Collider>(body);
    kin.velocity = Vec2::ZERO;
    cbuf.add_component(body, Itinerary::none())
}

fn walk_outside(
    body: Entity,
    pos: Vec2,
    cbuf: &ParCommandBuffer,
    mr: &mut MeshRender,
    loc: &mut Location,
) {
    mr.hide = false;
    *loc = Location::Outside;
    cbuf.exec(move |goria| {
        goria.comp_mut::<Transform>(body).unwrap().set_position(pos);
        let coll = put_pedestrian_in_coworld(&mut goria.write::<CollisionWorld>(), pos);
        goria.add_comp(body, coll);
    });
}

fn park(vehicle: VehicleID, spot_id: ParkingSpotID) -> impl FnOnce(&mut Egregoria) {
    move |goria| {
        let trans = goria.comp::<Transform>(vehicle.0).unwrap();
        let map = goria.read::<Map>();
        let spot = match map.parking.get(spot_id) {
            Some(x) => x,
            None => {
                log::warn!("Couldn't park at {:?} because it doesn't exist", spot_id);
                return;
            }
        };

        let s = Spline {
            from: trans.position(),
            to: spot.trans.position(),
            from_derivative: trans.direction() * 2.0,
            to_derivative: spot.trans.direction() * 2.0,
        };
        drop(map);

        goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state =
            VehicleState::RoadToPark(s, 0.0, spot_id);
        goria.comp_mut::<Kinematics>(vehicle.0).unwrap().velocity = Vec2::ZERO;
    }
}

fn unpark(vehicle: VehicleID) -> impl FnOnce(&mut Egregoria) {
    move |goria| {
        let v = goria.comp::<Vehicle>(vehicle.0).unwrap();
        let w = v.kind.width();

        if let VehicleState::Parked(spot) = v.state {
            goria.read::<ParkingManagement>().free(spot);
        } else {
            log::warn!("Trying to unpark {:?} that wasn't parked", vehicle);
        }

        let coll = put_vehicle_in_coworld(goria, w, *goria.comp::<Transform>(vehicle.0).unwrap());
        goria.add_comp(vehicle.0, coll);
        goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state = VehicleState::Driving;
    }
}

impl Router {
    pub fn new(personal_car: Option<VehicleID>) -> Self {
        Self {
            steps: vec![],
            cur_step: None,
            dest: None,
            reroute: false,
            personal_car,
            vehicle: personal_car,
        }
    }

    pub fn use_vehicle(&mut self, v: Option<VehicleID>) {
        self.vehicle = v;
    }

    fn clear_steps(&mut self, parking: &ParkingManagement) {
        for s in self.steps.drain(..) {
            if let RoutingStep::Park(_, spot) = s {
                parking.free(spot);
            }
        }
    }

    /// Returns wheter or not the destination was already attained
    pub fn go_to(&mut self, dest: Destination) -> bool {
        if let Some(router_dest) = self.dest {
            if router_dest == dest {
                return !self.reroute && self.steps.is_empty() && self.cur_step.is_none();
            }
        }
        self.dest = Some(dest);
        self.reroute = true;
        false
    }

    fn steps_to(
        &self,
        obj: Vec2,
        parking: &ParkingManagement,
        map: &Map,
        loc: &Location,
        subworld: &SubWorld,
    ) -> Vec<RoutingStep> {
        let mut steps = vec![];
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }

        if let Some(car) = self.vehicle {
            if let Some(spot_id) = parking.reserve_near(obj, &map) {
                let lane = map.parking_to_drive(spot_id).unwrap();
                let spot = *map.parking.get(spot_id).unwrap();

                let (pos, _, dir) = map.lanes()[lane]
                    .points
                    .project_segment_dir(spot.trans.position());
                let parking_pos = pos - dir * 4.0;

                if !matches!(loc, Location::Vehicle(_)) {
                    // safety: only pedestrians have transforms, not cars
                    let carpos = subworld
                        .entry_ref(car.0)
                        .unwrap()
                        .get_component::<Transform>()
                        .unwrap()
                        .position();

                    steps.push(RoutingStep::WalkTo(carpos));
                    steps.push(RoutingStep::GetInVehicle(car));
                    steps.push(RoutingStep::Unpark(car));
                }

                steps.push(RoutingStep::DriveTo(car, parking_pos));
                steps.push(RoutingStep::Park(car, spot_id));
                steps.push(RoutingStep::GetOutVehicle(car));
            }
        }

        steps.push(RoutingStep::WalkTo(obj));
        steps
    }
}
