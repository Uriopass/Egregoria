use egui::load::SizedTexture;
use egui::{Color32, Context, Id, Response, RichText, Ui};

use prototypes::{ItemID, Money};
use simulation::economy::Government;
use simulation::Simulation;

use crate::gui::chat::chat;
use crate::gui::inspect::inspector;
use crate::newgui::{ErrorTooltip, GuiState, PotentialCommands, UiTextures};
use crate::uiworld::UiWorld;

/// Root GUI entrypoint
pub fn render_oldgui(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::render");
    if uiworld.read::<GuiState>().hidden {
        return;
    }

    inspector(ui, uiworld, sim);

    chat(ui, uiworld, sim);

    uiworld
        .write::<GuiState>()
        .old_windows
        .render(ui, uiworld, sim);

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
                Some(egui::Pos2::new(s.x, s.y)),
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

pub fn item_icon(ui: &mut Ui, uiworld: &UiWorld, id: ItemID, multiplier: i32) -> Response {
    let item = id.prototype();
    ui.horizontal(move |ui| {
        if let Some(id) = uiworld
            .read::<UiTextures>()
            .try_get_egui(&format!("icon/{}", item.name))
        {
            if ui.image(SizedTexture::new(id, (32.0, 32.0))).hovered() {
                egui::show_tooltip(ui.ctx(), ui.make_persistent_id("icon tooltip"), |ui| {
                    ui.image(SizedTexture::new(id, (64.0, 64.0)));
                    ui.label(format!("{} x{}", item.name, multiplier));
                });
            }
        } else {
            ui.label(format!("- {} ", &item.label));
        }
        ui.label(format!("x{multiplier}"))
    })
    .inner
}
