use crate::gui::InspectedEntity;
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use common::Z_TOOL;
use egregoria::pedestrians::Location;
use egregoria::Egregoria;
use geom::Color;
use map_model::Map;

pub fn inspected_aura(goria: &Egregoria, uiworld: &mut UiWorld) {
    let inspected = uiworld.write::<InspectedEntity>();
    let map = goria.read::<Map>();
    let mut draw = uiworld.write::<ImmediateDraw>();

    if let Some(sel) = inspected.e {
        let mut pos = goria.pos(sel);

        if let Some(loc) = goria.comp::<Location>(sel) {
            match *loc {
                Location::Outside => {}
                Location::Vehicle(v) => pos = goria.pos(v.0),
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
