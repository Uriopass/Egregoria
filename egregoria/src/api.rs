use crate::engine_interaction::TimeInfo;
use crate::map_dynamic::{BuildingInfos, Itinerary};
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

#[derive(Debug)]
pub enum RoutingStep {
    WalkTo(Itinerary),
    DriveTo(VehicleID, Itinerary, ParkingSpotID),
}

#[derive(Debug)]
pub enum Action {
    DoNothing,
    GetOutBuilding(PedestrianID, BuildingID),
    GetInBuilding(PedestrianID, BuildingID),
    Navigate(PedestrianID, Vec<RoutingStep>),
}

impl Default for Action {
    fn default() -> Self {
        Self::DoNothing
    }
}

impl Action {
    pub fn go_to(goria: &Egregoria, body: PedestrianID, obj: Vec2) -> Action {
        match *goria.comp::<Location>(body.0).unwrap() {
            Location::Outside => {
                let pos = goria.pos(body.0).unwrap();

                let map = goria.read::<Map>();

                let itin = goria.comp::<Itinerary>(body.0).unwrap();
                if itin.end_pos().map(|x| x.approx_eq(obj)).unwrap_or(false) {
                    return Action::DoNothing;
                }

                if let Some(itin) = Itinerary::route(pos, obj, &*map, &PedestrianPath) {
                    return Action::Navigate(body, vec![RoutingStep::WalkTo(itin)]);
                }
            }
            Location::Building(cur_build) => {
                return Action::GetOutBuilding(body, cur_build);
            }
            Location::Car(_) => unimplemented!(),
        };
        Action::DoNothing
    }

    pub fn go_to_building(goria: &Egregoria, body: PedestrianID, obj: BuildingID) -> Action {
        if let Location::Building(id) = *goria.comp::<Location>(body.0).unwrap() {
            if id == obj {
                return Action::DoNothing;
            }
        }
        let bpos = goria.read::<Map>().buildings()[obj].door_pos;
        if bpos.is_close(goria.pos(body.0).unwrap(), 3.0) {
            return Action::GetInBuilding(body, obj);
        }

        Action::go_to(goria, body, bpos)
    }

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
                if let Some(v) = goria.comp_mut(e.0) {
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

    *goria.comp_mut::<Location>(body).unwrap() = Location::Outside(wpos);
    goria
        .comp_mut::<Transform>(body)
        .unwrap()
        .set_position(wpos);
    goria.comp_mut::<MeshRender>(body).unwrap().hide = false;
    let coll = put_pedestrian_in_coworld(goria, wpos);
    goria.read::<ParCommandBuffer>().add_component(body, coll);
}
