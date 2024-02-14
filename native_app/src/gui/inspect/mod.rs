use crate::gui::windows::debug::DebugState;
use crate::gui::FollowEntity;
use crate::newgui::{InspectedBuilding, InspectedEntity};
use crate::uiworld::UiWorld;
use egui::{Context, Ui, Window};
use inspect_building::inspect_building;
use inspect_debug::InspectRenderer;
use simulation::{AnyEntity, Simulation};

mod inspect_building;
mod inspect_debug;

pub fn inspector(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::inspector");
    let inspected_building = *uiworld.read::<InspectedBuilding>();
    if let Some(b) = inspected_building.e {
        inspect_building(uiworld, sim, ui, b);
    }

    let e = unwrap_or!(uiworld.read::<InspectedEntity>().e, return);

    let force_debug_inspect = uiworld.read::<DebugState>().debug_inspector;

    let mut is_open = true;
    if force_debug_inspect {
        Window::new("Inspect")
            .default_size([400.0, 500.0])
            .default_pos([30.0, 160.0])
            .resizable(true)
            .open(&mut is_open)
            .show(ui, |ui| {
                let mut ins = InspectRenderer { entity: e };
                ins.render(uiworld, sim, ui);
                uiworld.write::<InspectedEntity>().e = Some(ins.entity);
            });
    }

    if !is_open {
        uiworld.write::<InspectedEntity>().e = None;
    }
}

pub fn entity_link(uiworld: &UiWorld, sim: &Simulation, ui: &mut Ui, e: impl Into<AnyEntity>) {
    entity_link_inner(uiworld, sim, ui, e.into())
}

fn entity_link_inner(uiworld: &UiWorld, sim: &Simulation, ui: &mut Ui, e: AnyEntity) {
    let linkname = match e {
        AnyEntity::HumanID(id) => {
            if let Some(human) = sim.world().humans.get(id) {
                human.personal_info.name.to_string()
            } else {
                "???".to_string()
            }
        }
        _ => format!("{}", e),
    };

    if ui.link(linkname).clicked() {
        uiworld.write::<InspectedEntity>().e = Some(e);
        if sim.pos_any(e).is_some() {
            uiworld.write::<FollowEntity>().0 = Some(e);
        }
    }
}
