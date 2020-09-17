use crate::engine_interaction::TimeInfo;
use crate::map_dynamic::{BuildingInfos, Itinerary};
use crate::pedestrians::put_pedestrian_in_coworld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::meshrender_component::MeshRender;
use crate::{Egregoria, ParCommandBuffer};
use geom::{Transform, Vec2};
use legion::Entity;
use map_model::{BuildingID, Map, PedestrianPath};

#[derive(Eq, PartialEq)]
pub enum Location {
    Outside(Vec2),
    Car(Entity),
    Building(BuildingID),
}

pub enum Action {
    DoNothing,
    GetOutBuilding(Entity, BuildingID),
    GetInBuilding(Entity, BuildingID),
    Navigate(Entity, Itinerary),
}

impl Default for Action {
    fn default() -> Self {
        Self::DoNothing
    }
}

impl Action {
    pub fn walk_to(goria: &Egregoria, body: Entity, loc: Location) -> Action {
        match loc {
            Location::Building(build_id) => match *goria.comp::<Location>(body).unwrap() {
                Location::Outside(pos) => {
                    let map = goria.read::<Map>();
                    if map.buildings()[build_id].door_pos.distance2()

                    let itin = goria.comp::<Itinerary>(body).unwrap();

                    if itin.is_none() {

                        let door_pos = map.buildings()[build_id].door_pos;

                        let itin = unwrap_or!(
                            Itinerary::route(pos, door_pos, &*map, &PedestrianPath),
                            return Action::DoNothing
                        );

                        return Action::Navigate(body, itin);
                    }

                    if itin.has_ended(goria.read::<TimeInfo>().time) {
                        return Action::GetInBuilding(body, build_id);
                    }
                }
                Location::Building(cur_build) => {
                    if cur_build == build_id {
                        return Action::DoNothing;
                    }
                    return Action::GetOutBuilding(body, cur_build);
                }
                Location::Car(_) => unimplemented!(),
            },
            Location::Outside(_) => unimplemented!(),
            Location::Car(_) => unimplemented!(),
        };
        Action::DoNothing
    }

    pub fn apply(self, goria: &mut Egregoria) -> Option<()> {
        match self {
            Action::DoNothing => {}
            Action::GetOutBuilding(body, building) => {
                walk_out(goria, body, building);
            }
            Action::GetInBuilding(body, building) => {
                walk_in(goria, body, building);
            }
            Action::Navigate(e, itin) => {
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

fn walk_in(goria: &mut Egregoria, body: Entity, building: BuildingID) {
    goria.comp_mut::<MeshRender>(body).unwrap().hide = true;
    goria.write::<BuildingInfos>().get_in(building, body);
    goria
        .read::<ParCommandBuffer>()
        .remove_component::<Collider>(body);
    goria.comp_mut::<Kinematics>(body).unwrap().velocity = Vec2::ZERO;
    *goria.comp_mut::<Itinerary>(body).unwrap() = Itinerary::none();
    *goria.comp_mut::<Location>(body).unwrap() = Location::Building(building);
}

fn walk_out(goria: &mut Egregoria, body: Entity, building: BuildingID) {
    goria.write::<BuildingInfos>().get_out(building, body);
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
