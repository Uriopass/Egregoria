use crate::gui::{InspectedBuilding, InspectedEntity};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::Selectable;
use egregoria::transportation::Location;
use egregoria::Egregoria;
use geom::Color;

#[profiling::function]
pub(crate) fn inspected_aura(goria: &Egregoria, uiworld: &mut UiWorld) {
    let inspected = uiworld.write::<InspectedEntity>();
    let inspected_b = uiworld.write::<InspectedBuilding>();
    let map = goria.map();
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

        if let Some((pos, selectable)) = pos.zip(goria.comp::<Selectable>(sel)) {
            draw.stroke_circle(
                pos.up(0.25),
                selectable.radius,
                (selectable.radius * 0.01).max(0.1),
            )
            .color(Color::gray(0.7));
        }
    }

    if let Some(sel) = inspected_b.e {
        let b = map.buildings().get(sel).unwrap();

        // already shown by zonedit
        if b.zone.is_some() {
            return;
        }

        draw.obb(b.obb, b.height + 0.01)
            .color(egregoria::config().gui_primary);
    }
}
