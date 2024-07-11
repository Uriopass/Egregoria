use crate::gui::selectable::select_radius;
use crate::gui::{InspectedBuilding, InspectedEntity};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use geom::Color;
use simulation::transportation::Location;
use simulation::{AnyEntity, Simulation};

/// InspectedAura shows the circle around the inspected entity
pub fn inspected_aura(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::inspected_aura");
    let inspected = uiworld.write::<InspectedEntity>();
    let mut inspected_b = uiworld.write::<InspectedBuilding>();
    let map = sim.map();
    let mut draw = uiworld.write::<ImmediateDraw>();

    if let Some(sel) = inspected.e {
        let mut pos = sim.pos_any(sel);

        if let AnyEntity::HumanID(id) = sel {
            let loc = &sim.world().get(id).unwrap().location;
            match *loc {
                Location::Outside => {}
                Location::Vehicle(v) => pos = sim.pos(v),
                Location::Building(b) => pos = map.buildings().get(b).map(|b| b.door_pos),
            }
        }

        if let Some(pos) = pos {
            let select_radius = select_radius(sel);

            if select_radius > 0.0 {
                draw.stroke_circle(pos.up(0.25), select_radius, (select_radius * 0.01).max(0.1))
                    .color(Color::gray(0.7));
            }
        }
    }

    if let Some(sel) = inspected_b.e {
        let Some(b) = map.buildings().get(sel) else {
            inspected_b.e = None;
            return;
        };

        // already shown by zonedit
        if b.zone.is_some() {
            return;
        }

        draw.obb(b.obb, b.height + 0.01)
            .color(simulation::colors().gui_primary);
    }
}
