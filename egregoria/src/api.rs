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

#[derive(Eq, PartialEq)]
pub enum Action {
    DoNothing,
    WalkTo(Entity, Location),
}

impl Default for Action {
    fn default() -> Self {
        Self::DoNothing
    }
}

impl Action {
    pub fn apply(self, goria: &mut Egregoria) -> Option<()> {
        match self {
            Action::WalkTo(body, Location::Building(build_id)) => {
                match *goria.comp::<Location>(body).unwrap() {
                    Location::Outside(pos) => {
                        let itin = goria.comp::<Itinerary>(body).unwrap();
                        if itin.is_none() {
                            let map = goria.read::<Map>();

                            let door_pos = map.buildings()[build_id].door_pos;

                            let itin = Itinerary::route(pos, door_pos, &*map, &PedestrianPath)?;
                            drop(map);

                            *goria.comp_mut::<Itinerary>(body).unwrap() = itin;
                            return Some(());
                        }

                        if itin.has_ended(goria.read::<TimeInfo>().time) {
                            walk_in(goria, body, build_id);
                            *goria.comp_mut::<Itinerary>(body).unwrap() = Itinerary::none();
                        }
                    }
                    Location::Building(current_building_id) => {
                        if current_building_id == build_id {
                            return Some(());
                        }

                        walk_out(goria, body, current_building_id)
                    }
                    Location::Car(_) => unimplemented!(),
                }
            }
            Action::WalkTo(_, Location::Outside(_)) => {}
            _ => {}
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
