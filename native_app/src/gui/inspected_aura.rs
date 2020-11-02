use crate::gui::Z_TOOL;
use common::inspect::InspectedEntity;
use egregoria::api::Location;
use egregoria::rendering::immediate::ImmediateDraw;
use geom::Color;
use geom::Transform;
use legion::world::SubWorld;
use legion::{system, EntityStore};
use map_model::Map;

#[system]
#[read_component(Location)]
#[read_component(Transform)]
pub fn inspected_aura(
    #[resource] inspected: &mut InspectedEntity,
    #[resource] map: &Map,
    #[resource] draw: &mut ImmediateDraw,
    sw: &SubWorld,
) {
    if let Some(sel) = inspected.e {
        let mut pos = sw
            .entry_ref(sel)
            .unwrap()
            .get_component::<Transform>()
            .ok()
            .map(|x| x.position());

        if let Ok(loc) = sw.entry_ref(sel).unwrap().get_component::<Location>() {
            match *loc {
                Location::Outside => {}
                Location::Vehicle(v) => {
                    pos = sw
                        .entry_ref(v.0)
                        .unwrap()
                        .get_component::<Transform>()
                        .ok()
                        .map(|x| x.position())
                }
                Location::Building(b) => pos = map.buildings().get(b).map(|b| b.door_pos),
            }
        }

        if let Some(pos) = pos {
            draw.stroke_circle(pos, 3.0, 0.1)
                .z(Z_TOOL)
                .color(Color::gray(0.7));
        }
    }
}
