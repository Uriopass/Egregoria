use crate::map_dynamic::{BuildingInfos, Itinerary, ParkingManagement};
use crate::pedestrians::data::PedestrianID;
use crate::pedestrians::put_pedestrian_in_coworld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::{put_vehicle_in_coworld, Vehicle, VehicleID, VehicleState};
use crate::{Egregoria, ParCommandBuffer};
use geom::{Spline, Transform, Vec2};
use legion::Entity;
use map_model::{BuildingID, CarPath, Map, ParkingSpotID, PedestrianPath};

#[derive(Eq, PartialEq)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Destination {
    Outside(Vec2),
    Building(BuildingID),
}

#[derive(Debug)]
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

impl RoutingStep {
    pub fn ready(&self, goria: &Egregoria, body: PedestrianID) -> bool {
        let pos = goria.pos(body.0).unwrap();
        match self {
            RoutingStep::WalkTo(_) => true,
            RoutingStep::DriveTo(_, _) => true,
            RoutingStep::Park(vehicle, _) => {
                goria.comp::<Itinerary>(vehicle.0).unwrap().has_ended(0.0)
            }
            RoutingStep::Unpark(_) => true,
            RoutingStep::GetInVehicle(vehicle) => goria.pos(vehicle.0).unwrap().is_close(pos, 3.0),
            RoutingStep::GetOutVehicle(vehicle) => matches!(
                goria.comp::<Vehicle>(vehicle.0).unwrap().state,
                VehicleState::Parked(_)
            ),
            &RoutingStep::GetInBuilding(build) => goria.read::<Map>().buildings()[build]
                .door_pos
                .is_close(pos, 3.0),
            RoutingStep::GetOutBuilding(_) => true,
        }
    }
    pub fn action(self, goria: &Egregoria, body: PedestrianID) -> Action {
        match self {
            RoutingStep::WalkTo(obj) => {
                let pos = goria.pos(body.0).unwrap();

                let map = goria.read::<Map>();

                if let Some(itin) = Itinerary::route(pos, obj, &*map, &PedestrianPath) {
                    Action::Navigate(body.0, itin)
                } else {
                    Action::DoNothing
                }
            }
            RoutingStep::DriveTo(vehicle, obj) => {
                let pos = goria.pos(body.0).unwrap();

                let map = goria.read::<Map>();

                if let Some(itin) = Itinerary::route(pos, obj, &*map, &CarPath) {
                    Action::Navigate(vehicle.0, itin)
                } else {
                    Action::DoNothing
                }
            }
            RoutingStep::Park(vehicle, spot) => Action::Park(vehicle, spot),
            RoutingStep::Unpark(vehicle) => Action::Unpark(vehicle),
            RoutingStep::GetInVehicle(vehicle) => Action::GetInVehicle(body, vehicle),
            RoutingStep::GetOutVehicle(vehicle) => Action::GetOutVehicle(body, vehicle),
            RoutingStep::GetInBuilding(build) => Action::GetInBuilding(body, build),
            RoutingStep::GetOutBuilding(build) => Action::GetOutBuilding(body, build),
        }
    }
}

pub struct Router {
    body: PedestrianID,
    steps: Vec<RoutingStep>,
    dest: Option<Destination>,
}

impl Router {
    pub fn new(body: PedestrianID) -> Self {
        Self {
            body,
            steps: vec![],
            dest: None,
        }
    }

    fn clear_steps(&mut self, goria: &Egregoria) {
        for s in self.steps.drain(..) {
            if let RoutingStep::Park(_, spot) = s {
                goria.read::<ParkingManagement>().free(spot);
            }
        }
    }

    pub fn go_to(&mut self, goria: &Egregoria, dest: Destination) -> Action {
        if self.dest.map(|x| x == dest).unwrap_or(false) {
            return self.action(goria);
        }

        self.dest = Some(dest);

        self.clear_steps(goria);
        match dest {
            Destination::Outside(pos) => self.steps = Self::steps_to(goria, self.body, pos),
            Destination::Building(build) => {
                let loc = goria.comp::<Location>(self.body.0).unwrap();
                if let Location::Building(cur_build) = loc {
                    if *cur_build == build {
                        return Action::DoNothing;
                    }
                }

                let door_pos = goria.read::<Map>().buildings()[build].door_pos;
                self.steps = Self::steps_to(goria, self.body, door_pos);
                self.steps.push(RoutingStep::GetInBuilding(build));
            }
        }

        self.steps.reverse();

        self.action(goria)
    }

    fn steps_to(goria: &Egregoria, body: PedestrianID, obj: Vec2) -> Vec<RoutingStep> {
        let mut steps = vec![];
        let loc = goria.comp::<Location>(body.0).unwrap();
        if let Location::Building(cur_build) = loc {
            steps.push(RoutingStep::GetOutBuilding(*cur_build));
        }
        steps.push(RoutingStep::WalkTo(obj));
        steps
    }

    pub fn action(&mut self, goria: &Egregoria) -> Action {
        let step = unwrap_or!(self.steps.last(), return Action::DoNothing);
        if step.ready(goria, self.body) {
            let step = self.steps.pop().unwrap();
            return step.action(goria, self.body);
        }
        Action::DoNothing
    }
}

#[derive(Debug)]
pub enum Action {
    DoNothing,
    GetOutBuilding(PedestrianID, BuildingID),
    GetInBuilding(PedestrianID, BuildingID),
    GetOutVehicle(PedestrianID, VehicleID),
    GetInVehicle(PedestrianID, VehicleID),
    Navigate(Entity, Itinerary),
    Park(VehicleID, ParkingSpotID),
    Unpark(VehicleID),
}

impl Default for Action {
    fn default() -> Self {
        Self::DoNothing
    }
}

impl Action {
    pub fn apply(self, goria: &mut Egregoria) -> Option<()> {
        match self {
            Action::DoNothing => {}
            Action::GetOutBuilding(body, building) => {
                log::info!("{:?}", self);
                goria.write::<BuildingInfos>().get_out(building, body);
                let wpos = goria.read::<Map>().buildings()[building].door_pos;
                walk_outside(goria, body, wpos);
            }
            Action::GetInBuilding(body, building) => {
                log::info!("{:?}", self);
                goria.write::<BuildingInfos>().get_in(building, body);
                *goria.comp_mut::<Location>(body.0).unwrap() = Location::Building(building);
                walk_inside(goria, body);
            }
            Action::GetOutVehicle(body, vehicle) => {
                log::info!("{:?}", self);
                let trans = *goria.comp::<Transform>(vehicle.0).unwrap();
                walk_outside(
                    goria,
                    body,
                    trans.position() + trans.direction().perpendicular() * 2.0,
                );
            }
            Action::GetInVehicle(body, vehicle) => {
                log::info!("{:?}", self);
                *goria.comp_mut::<Location>(body.0).unwrap() = Location::Vehicle(vehicle);
                walk_inside(goria, body);
            }
            Action::Navigate(e, itin) => {
                log::info!("Navigate {:?}", e);
                if let Some(v) = goria.comp_mut(e) {
                    *v = itin;
                } else {
                    log::warn!("Called navigate on entity that doesn't have itinerary component");
                }
            }
            Action::Park(vehicle, spot_id) => {
                let trans = goria.comp::<Transform>(vehicle.0).unwrap();
                let spot = *goria.read::<Map>().parking.get(spot_id).unwrap();

                let s = Spline {
                    from: trans.position(),
                    to: spot.trans.position(),
                    from_derivative: trans.direction() * 2.0,
                    to_derivative: spot.trans.direction() * 2.0,
                };

                goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state =
                    VehicleState::RoadToPark(s, 0.0, spot_id);
                goria.comp_mut::<Kinematics>(vehicle.0).unwrap().velocity = Vec2::ZERO;
            }
            Action::Unpark(vehicle) => {
                let v = goria.comp::<Vehicle>(vehicle.0).unwrap();
                let w = v.kind.width();

                if let VehicleState::Parked(spot) = v.state {
                    goria.read::<ParkingManagement>().free(spot);
                }
                put_vehicle_in_coworld(goria, w, *goria.comp::<Transform>(vehicle.0).unwrap());
                goria.comp_mut::<Vehicle>(vehicle.0).unwrap().state = VehicleState::Driving;
            }
        }
        Some(())
    }
}

fn walk_inside(goria: &mut Egregoria, body: PedestrianID) {
    let body = body.0;
    goria.comp_mut::<MeshRender>(body).unwrap().hide = true;
    goria
        .read::<ParCommandBuffer>()
        .remove_component::<Collider>(body);
    goria.comp_mut::<Kinematics>(body).unwrap().velocity = Vec2::ZERO;
    *goria.comp_mut::<Itinerary>(body).unwrap() = Itinerary::none();
}

fn walk_outside(goria: &mut Egregoria, body: PedestrianID, pos: Vec2) {
    let body = body.0;
    *goria.comp_mut::<Location>(body).unwrap() = Location::Outside;
    goria.comp_mut::<Transform>(body).unwrap().set_position(pos);
    goria.comp_mut::<MeshRender>(body).unwrap().hide = false;
    let coll = put_pedestrian_in_coworld(goria, pos);
    goria.read::<ParCommandBuffer>().add_component(body, coll);
}
