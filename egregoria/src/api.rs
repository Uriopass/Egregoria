use crate::map_dynamic::{BuildingInfos, Itinerary, ParkingManagement};
use crate::pedestrians::put_pedestrian_in_coworld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::{Egregoria, ParCommandBuffer};
use geom::{Transform, Vec2};
use legion::Entity;
use map_model::{BuildingID, Map, ParkingSpotID, PedestrianPath};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PedestrianID(pub Entity);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VehicleID(pub Entity);

#[derive(Eq, PartialEq)]
pub enum Location {
    Outside,
    Car(VehicleID),
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
    DriveTo(VehicleID, ParkingSpotID),
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
            RoutingStep::GetInVehicle(vehicle) => goria.pos(vehicle.0).unwrap().is_close(pos, 3.0),
            RoutingStep::GetOutVehicle(vehicle) => {
                goria.comp::<Itinerary>(vehicle.0).unwrap().has_ended(0.0)
            }
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
            RoutingStep::DriveTo(_, _) => unimplemented!(),
            RoutingStep::GetInVehicle(_) => unimplemented!(),
            RoutingStep::GetOutVehicle(_) => unimplemented!(),
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
            if let RoutingStep::DriveTo(_, spot) = s {
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
    Navigate(Entity, Itinerary),
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
                walk_out(goria, body, building);
            }
            Action::GetInBuilding(body, building) => {
                log::info!("{:?}", self);
                walk_in(goria, body, building);
            }
            Action::Navigate(e, itin) => {
                log::info!("Navigate {:?}", e);
                if let Some(v) = goria.comp_mut(e) {
                    *v = itin;
                } else {
                    log::warn!("Called navigate on entity that doesn't have itinerary component");
                }
            }
        }
        Some(())
    }
}

fn walk_in(goria: &mut Egregoria, body: PedestrianID, building: BuildingID) {
    goria.write::<BuildingInfos>().get_in(building, body);

    let body = body.0;
    goria.comp_mut::<MeshRender>(body).unwrap().hide = true;
    goria
        .read::<ParCommandBuffer>()
        .remove_component::<Collider>(body);
    goria.comp_mut::<Kinematics>(body).unwrap().velocity = Vec2::ZERO;
    *goria.comp_mut::<Itinerary>(body).unwrap() = Itinerary::none();
    *goria.comp_mut::<Location>(body).unwrap() = Location::Building(building);
}

fn walk_out(goria: &mut Egregoria, body: PedestrianID, building: BuildingID) {
    goria.write::<BuildingInfos>().get_out(building, body);

    let body = body.0;
    let wpos = goria.read::<Map>().buildings()[building].door_pos;

    *goria.comp_mut::<Location>(body).unwrap() = Location::Outside;
    goria
        .comp_mut::<Transform>(body)
        .unwrap()
        .set_position(wpos);
    goria.comp_mut::<MeshRender>(body).unwrap().hide = false;
    let coll = put_pedestrian_in_coworld(goria, wpos);
    goria.read::<ParCommandBuffer>().add_component(body, coll);
}
