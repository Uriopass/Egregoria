use crate::map_dynamic::{Itinerary, ParkingManagement};
use crate::pedestrians::{put_pedestrian_in_coworld, Location};
use crate::physics::{Collider, CollisionWorld, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::{unpark, Vehicle, VehicleID, VehicleState};
use crate::{Egregoria, ParCommandBuffer};
use geom::{Spline, Transform, Vec2};
use imgui_inspect_derive::*;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore};
use map_model::{BuildingID, Map, ParkingSpotID, PathKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Inspect, Serialize, Deserialize)]
pub struct Router {
    steps: Vec<RoutingStep>,
    cur_step: Option<RoutingStep>,
    target_dest: Option<Destination>,
    cur_dest: Option<Destination>,
    vehicle: Option<VehicleID>,
    pub personal_car: Option<VehicleID>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    Outside(Vec2),
    Building(BuildingID),
}

debug_inspect_impl!(Destination);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

register_system!(routing_changed);
register_system!(routing_update);

#[system(for_each)]
#[read_component(Transform)]
#[read_component(Vehicle)]
#[read_component(Itinerary)]
pub fn routing_changed(
    #[resource] map: &Map,
    #[resource] parking: &mut ParkingManagement,
    router: &mut Router,
    loc: &Location,
    subworld: &SubWorld,
) {
    if router.cur_dest != router.target_dest {
        let dest = unwrap_ret!(router.target_dest);

        router.clear_steps(parking);
        match dest {
            Destination::Outside(pos) => {
                router.steps = unwrap_ret!(router.steps_to(pos, parking, map, loc, subworld));
            }
            Destination::Building(build) => {
                if let Location::Building(cur_build) = loc {
                    if *cur_build == build {
                        return;
                    }
                }

                let door_pos = unwrap_ret!(map.buildings().get(build)).door_pos;
                router.steps = unwrap_ret!(router.steps_to(door_pos, parking, map, loc, subworld));
                router.steps.push(RoutingStep::GetInBuilding(build));
            }
        }

        router.cur_dest = router.target_dest;

        router.steps.reverse();
    }
}

#[system(par_for_each)]
#[read_component(Transform)]
#[read_component(Vehicle)]
#[read_component(Itinerary)]
pub fn routing_update(
    #[resource] map: &Map,
    #[resource] cbuf: &ParCommandBuffer,
    body: &Entity,
    trans: &Transform,
    itin: &Itinerary,
    router: &mut Router,
    loc: &mut Location,
    mr: &mut MeshRender,
    kin: &mut Kinematics,
    subworld: &SubWorld,
) {
    let pos = match *loc {
        Location::Outside => trans.position(),
        Location::Vehicle(id) => subworld
            .entry_ref(id.0)
            .ok()
            .and_then(|x| x.get_component::<Transform>().map(|x| x.position()).ok())
            .unwrap_or_else(|| trans.position()),
        Location::Building(id) => map
            .buildings()
            .get(id)
            .map(|b| b.door_pos)
            .unwrap_or_else(|| trans.position()),
    };

    let next_step = unwrap_or!(router.steps.last(), {
        router.cur_step = None;
        return;
    });

    let mut cur_step_over = true;

    if let Some(step) = router.cur_step {
        cur_step_over = match step {
            RoutingStep::WalkTo(_) => itin.has_ended(0.0),
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
            map.buildings()[build].door_pos.is_close(pos, 3.0) // fixme check building exists
        }
        RoutingStep::GetOutBuilding(_) => true,
    };

    if !(next_step_ready && cur_step_over) {
        return;
    }

    router.cur_step = Some(router.steps.pop().unwrap());

    match router.cur_step.unwrap() {
        RoutingStep::WalkTo(obj) => {
            if let Some(route) = Itinerary::route(pos, obj, &*map, PathKind::Pedestrian) {
                cbuf.add_component(*body, route);
            }
        }
        RoutingStep::DriveTo(vehicle, obj) => {
            if let Some(route) = Itinerary::route(pos, obj, &*map, PathKind::Vehicle) {
                cbuf.add_component(vehicle.0, route);
            }
        }
        RoutingStep::Park(vehicle, spot) => {
            if !map.parking.contains(spot) {
                router.cur_dest = None;
                return;
            }

            cbuf.exec_ent(vehicle.0, park(vehicle, spot));
        }
        RoutingStep::Unpark(vehicle) => {
            cbuf.exec_ent(vehicle.0, move |goria| unpark(goria, vehicle));
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
    cbuf.exec_ent(body, move |goria| {
        goria.comp_mut::<Transform>(body).unwrap().set_position(pos);
        let coll = put_pedestrian_in_coworld(&mut goria.write::<CollisionWorld>(), pos);
        goria.add_comp(body, coll);
    });
}

fn park(vehicle: VehicleID, spot_id: ParkingSpotID) -> impl FnOnce(&mut Egregoria) {
    move |goria| {
        let trans = goria.comp::<Transform>(vehicle.0).unwrap();
        let map = goria.map();
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

impl Router {
    pub fn new(personal_car: Option<VehicleID>) -> Self {
        Self {
            steps: vec![],
            cur_step: None,
            target_dest: None,
            personal_car,
            vehicle: personal_car,
            cur_dest: None,
        }
    }

    pub fn use_vehicle(&mut self, v: Option<VehicleID>) {
        self.vehicle = v;
    }

    fn clear_steps(&mut self, parking: &mut ParkingManagement) {
        for s in self.steps.drain(..).chain(self.cur_step.take()) {
            if let RoutingStep::Park(_, spot) = s {
                parking.free(spot);
            }
        }
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
        &self,
        obj: Vec2,
        parking: &mut ParkingManagement,
        map: &Map,
        loc: &Location,
        subworld: &SubWorld,
    ) -> Option<Vec<RoutingStep>> {
        let mut steps = vec![];
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }

        if let Some(car) = self.vehicle {
            let spot_id = parking.reserve_near(obj, map)?;
            let parking_pos = match map.parking_to_drive_pos(spot_id) {
                Some(x) => x,
                None => {
                    parking.free(spot_id);
                    return None;
                }
            };

            if !matches!(loc, Location::Vehicle(_)) {
                let ent = subworld.entry_ref(car.0).ok()?;
                let trans = ent.get_component::<Transform>().ok()?;
                steps.push(RoutingStep::WalkTo(trans.position()));
                steps.push(RoutingStep::GetInVehicle(car));
                steps.push(RoutingStep::Unpark(car));
            }

            steps.push(RoutingStep::DriveTo(car, parking_pos));
            steps.push(RoutingStep::Park(car, spot_id));
            steps.push(RoutingStep::GetOutVehicle(car));
        }

        steps.push(RoutingStep::WalkTo(obj));
        Some(steps)
    }
}
