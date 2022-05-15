use crate::map_dynamic::{Itinerary, ParkingManagement, SpotReservation};
use crate::pedestrians::{put_pedestrian_in_coworld, Location};
use crate::physics::{Collider, CollisionWorld, Kinematics};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::vehicles::{unpark, Vehicle, VehicleID, VehicleState};
use crate::{Egregoria, ParCommandBuffer};
use geom::{Spline3, Transform, Vec3};
use hecs::{Component, Entity, Ref, World};
use imgui_inspect_derive::Inspect;
use map_model::{BuildingID, Map, PathKind};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(Inspect, Serialize, Deserialize)]
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

#[profiling::function]
pub fn routing_changed_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    let rb = &mut *resources.get_mut().unwrap();
    world
        .query::<(&mut Router, &Location)>()
        .iter()
        .for_each(|(_, (a, b))| {
            routing_changed(ra, rb, a, b, world);
        });
}

pub fn routing_changed(
    map: &Map,
    parking: &mut ParkingManagement,
    router: &mut Router,
    loc: &Location,
    world: &World,
) {
    if router.cur_dest != router.target_dest {
        let dest = unwrap_ret!(router.target_dest);

        router.clear_steps(parking);
        match dest {
            Destination::Outside(pos) => {
                router.steps = unwrap_ret!(router.steps_to(pos, parking, map, loc, world));
            }
            Destination::Building(build) => {
                if let Location::Building(cur_build) = loc {
                    if *cur_build == build {
                        router.cur_dest = router.target_dest;
                        return;
                    }
                }

                let door_pos = unwrap_ret!(map.buildings().get(build)).door_pos;
                router.steps = unwrap_ret!(router.steps_to(door_pos, parking, map, loc, world));
                router.steps.push(RoutingStep::GetInBuilding(build));
            }
        }

        router.cur_dest = router.target_dest;

        router.steps.reverse();
    }
}

#[profiling::function]
pub fn routing_update_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    let rb = &*resources.get().unwrap();
    world
        .query::<(
            &Transform,
            &Itinerary,
            &mut Router,
            &mut Location,
            &mut Kinematics,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| {
            batch.for_each(|(e, (a, b, c, d, f))| routing_update(ra, rb, e, a, b, c, d, f, world))
        });
}

pub fn routing_update(
    map: &Map,
    cbuf: &ParCommandBuffer,
    body: Entity,
    trans: &Transform,
    itin: &Itinerary,
    router: &mut Router,
    loc: &mut Location,
    kin: &mut Kinematics,
    world: &World,
) {
    if router.cur_step.is_none() && router.steps.is_empty() {
        return;
    }

    let pos = match *loc {
        Location::Outside => trans.position,
        Location::Vehicle(id) => comp::<Transform>(world, id.0)
            .map(|x| x.position)
            .unwrap_or_else(|| trans.position),
        Location::Building(id) => map
            .buildings()
            .get(id)
            .map(|b| b.door_pos)
            .unwrap_or_else(|| trans.position),
    };

    let mut cur_step_over = true;

    if let Some(ref step) = router.cur_step {
        cur_step_over = match *step {
            RoutingStep::WalkTo(_) => itin.has_ended(0.0),
            RoutingStep::DriveTo(vehicle, _) => comp::<Itinerary>(world, vehicle.0)
                .map(|x| x.has_ended(0.0))
                .unwrap_or(true),
            RoutingStep::Park(vehicle, _) => comp::<Vehicle>(world, vehicle.0)
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

    if let Some(step) = router.steps.last() {
        next_step_ready = match *step {
            RoutingStep::WalkTo(_) => true,
            RoutingStep::DriveTo(_, _) => true,
            RoutingStep::Park(_, _) => true,
            RoutingStep::Unpark(_) => true,
            RoutingStep::GetInVehicle(vehicle) => comp::<Transform>(world, vehicle.0)
                .map(|x| x.position.is_close(pos, 3.0))
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

    router.cur_step = router.steps.pop();

    if let Some(ref mut next_step) = router.cur_step {
        match *next_step {
            RoutingStep::WalkTo(obj) => {
                cbuf.add_component(body, Itinerary::wait_for_reroute(PathKind::Pedestrian, obj));
            }
            RoutingStep::DriveTo(vehicle, obj) => {
                let route = Itinerary::wait_for_reroute(PathKind::Vehicle, obj);
                cbuf.add_component(vehicle.0, route);
            }
            RoutingStep::Park(vehicle, ref mut spot) => {
                if let Some(x) = spot.take() {
                    if !x.exists(&map.parking) {
                        router.reset_dest();
                        return;
                    }

                    cbuf.exec_ent(vehicle.0, park(vehicle, x));
                }
            }
            RoutingStep::Unpark(vehicle) => {
                cbuf.exec_ent(vehicle.0, move |goria| unpark(goria, vehicle));
            }
            RoutingStep::GetInVehicle(vehicle) => {
                if !world.contains(vehicle.0) {
                    router.reset_dest();
                    return;
                }
                *loc = Location::Vehicle(vehicle);
                walk_inside(body, cbuf, kin);
            }
            RoutingStep::GetOutVehicle(vehicle) => {
                let pos = comp::<Transform>(world, vehicle.0)
                    .map(|vtrans| vtrans.position + vtrans.dir.cross(Vec3::Z) * 2.0)
                    .unwrap_or(pos);
                walk_outside(body, pos, cbuf, loc);
            }
            RoutingStep::GetInBuilding(build) => {
                if !map.buildings().contains_key(build) {
                    router.reset_dest();
                    return;
                }
                *loc = Location::Building(build);
                walk_inside(body, cbuf, kin);
            }
            RoutingStep::GetOutBuilding(build) => {
                let wpos = map
                    .buildings()
                    .get(build)
                    .map(|x| x.door_pos)
                    .unwrap_or(pos);
                walk_outside(body, wpos, cbuf, loc);
            }
        }
    }
}

impl ComponentDrop for Router {
    fn drop(&mut self, res: &mut Resources, _: Entity) {
        self.clear_steps(&mut *res.get_mut::<ParkingManagement>().unwrap())
    }
}

fn comp<T: Component>(sw: &World, e: Entity) -> Option<Ref<T>> {
    sw.get(e).ok()
}

fn walk_inside(body: Entity, cbuf: &ParCommandBuffer, kin: &mut Kinematics) {
    cbuf.remove_component_drop::<Collider>(body);
    kin.speed = 0.0;
    cbuf.add_component(body, Itinerary::NONE)
}

fn walk_outside(body: Entity, pos: Vec3, cbuf: &ParCommandBuffer, loc: &mut Location) {
    *loc = Location::Outside;
    cbuf.exec_ent(body, move |goria| {
        unwrap_ret!(goria.comp_mut::<Transform>(body)).position = pos;
        let coll = put_pedestrian_in_coworld(&mut goria.write::<CollisionWorld>(), pos);
        goria.add_comp(body, coll);
    });
}

fn park(vehicle: VehicleID, spot_resa: SpotReservation) -> impl FnOnce(&mut Egregoria) {
    move |goria| {
        let trans = unwrap_ret!(goria.comp::<Transform>(vehicle.0));
        let map = goria.map();
        let spot = match spot_resa.get(&map.parking) {
            Some(x) => x,
            None => {
                log::warn!("Couldn't park at {:?} because it doesn't exist", spot_resa);
                return;
            }
        };

        let s = Spline3 {
            from: trans.position,
            to: spot.trans.position,
            from_derivative: trans.dir * 2.0,
            to_derivative: spot.trans.dir * 2.0,
        };
        drop(map);
        drop(trans);

        unwrap_ret!(goria.comp_mut::<Vehicle>(vehicle.0)).state =
            VehicleState::RoadToPark(s, 0.0, spot_resa);
        unwrap_ret!(goria.comp_mut::<Kinematics>(vehicle.0)).speed = 0.0;
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
        world: &World,
    ) -> Option<Vec<RoutingStep>> {
        let mut steps = vec![];
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }

        if let Some(car) = self.vehicle {
            let spot_resa = parking.reserve_near(obj, map)?;
            let parking_pos = match spot_resa.park_pos(map) {
                Some(x) => x,
                None => {
                    parking.free(spot_resa);
                    return None;
                }
            };

            if !matches!(loc, Location::Vehicle(_)) {
                if let Some(trans) = comp::<Transform>(world, car.0) {
                    steps.push(RoutingStep::WalkTo(trans.position));
                    steps.push(RoutingStep::GetInVehicle(car));
                    steps.push(RoutingStep::Unpark(car));
                } else {
                    parking.free(spot_resa);
                    self.vehicle = None;
                    return None;
                }
            }

            steps.push(RoutingStep::DriveTo(car, parking_pos));
            steps.push(RoutingStep::Park(car, Some(spot_resa)));
            steps.push(RoutingStep::GetOutVehicle(car));
        }

        steps.push(RoutingStep::WalkTo(obj));
        Some(steps)
    }
}
