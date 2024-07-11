use egui::{Color32, Context, Id, RichText};

use prototypes::Money;
use simulation::economy::Government;
use simulation::Simulation;

use crate::debug_gui::debug_inspect::debug_inspector;
use crate::debug_gui::debug_window::debug_window;
use crate::gui::{ErrorTooltip, GuiState, PotentialCommands};
use crate::uiworld::UiWorld;

/// Root GUI entrypoint
pub fn render_oldgui(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::render");
    if uiworld.read::<GuiState>().hidden {
        return;
    }

    debug_inspector(ui, uiworld, sim);

    debug_window(ui, uiworld, sim);

    tooltip(ui, uiworld, sim);
}

pub fn tooltip(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("gui::tooltip");
    let tooltip = std::mem::take(&mut *uiworld.write::<ErrorTooltip>());
    if let Some(msg) = tooltip.msg {
        if !(tooltip.isworld && ui.is_pointer_over_area()) {
            let s = ui.available_rect().size();
            egui::show_tooltip_at(
                ui,
                Id::new("tooltip_error"),
                egui::Pos2::new(s.x, s.y),
                |ui| ui.label(RichText::new(msg).color(Color32::from_rgb(255, 100, 100))),
            );
        }
    }

    if ui.is_pointer_over_area() {
        return;
    }
    let pot = &mut uiworld.write::<PotentialCommands>().0;
    let cost: Money = pot
        .drain(..)
        .map(|cmd| Government::action_cost(&cmd, sim))
        .sum();

    if cost == Money::ZERO {
        return;
    }

    egui::show_tooltip(ui, Id::new("tooltip_command_cost"), |ui| {
        if cost > sim.read::<Government>().money {
            ui.colored_label(Color32::RED, format!("{cost} too expensive"));
        } else {
            ui.label(cost.to_string());
        }
    });
}
