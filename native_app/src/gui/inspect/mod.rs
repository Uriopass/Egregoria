use egui::{Context, Window};

use inspect_debug::InspectRenderer;
use simulation::Simulation;

use crate::gui::windows::debug::DebugState;
use crate::newgui::InspectedEntity;
use crate::uiworld::UiWorld;

mod inspect_debug;

pub fn inspector(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::inspector");
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
