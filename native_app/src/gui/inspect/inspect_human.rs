use egui::{Context, Widget};

use egregoria::economy::{ItemRegistry, Market};
use egregoria::map_dynamic::Destination;
use egregoria::souls::desire::WorkKind;
use egregoria::transportation::Location;
use egregoria::{Egregoria, HumanID};

use crate::gui::inspect::{building_link, follow_button};
use crate::gui::item_icon;
use crate::uiworld::UiWorld;

/// Inspect a specific building, showing useful information about it
pub fn inspect_human(uiworld: &mut UiWorld, goria: &Egregoria, ui: &Context, id: HumanID) -> bool {
    let Some(human) = goria.get(id) else { return false; };

    let mut is_open = true;
    egui::Window::new("Human")
        .resizable(false)
        .auto_sized()
        .open(&mut is_open)
        .show(ui, |ui| {
            if cfg!(debug_assertions) {
                ui.label(format!("{:?}", id));
            }
            let pinfo = &human.personal_info;
            ui.label(format!("{}{:?} • {}", pinfo.age, pinfo.gender, pinfo.name));

            match human.location {
                Location::Outside => {}
                Location::Vehicle(_) => {
                    ui.label("In a vehicle");
                }
                Location::Building(x) => {
                    ui.horizontal(|ui| {
                        ui.label("In a building:");
                        building_link(uiworld, goria, ui, x);
                    });
                }
            }

            if let Some(ref dest) = human.router.target_dest {
                match dest {
                    Destination::Outside(pos) => {
                        ui.label(format!("Going to {}", pos));
                    }
                    Destination::Building(b) => {
                        ui.horizontal(|ui| {
                            ui.label("Going to building");
                            building_link(uiworld, goria, ui, *b);
                        });
                    }
                }
            }

            ui.horizontal(|ui| {
                ui.label("House is");
                building_link(uiworld, goria, ui, human.home.house);
            });

            ui.label(format!("Last ate: {}", human.food.last_ate));

            if let Some(ref x) = human.work {
                ui.horizontal(|ui| {
                    ui.label("Working at");
                    building_link(uiworld, goria, ui, x.workplace);
                    match x.kind {
                        WorkKind::Driver { .. } => {
                            ui.label("as a driver");
                        }
                        WorkKind::Worker => {
                            ui.label("as a worker");
                        }
                    }
                });
            }

            ui.add_space(10.0);
            ui.label("Desires");
            ui.horizontal(|ui| {
                let mut score = human.food.last_score;
                egui::DragValue::new(&mut score).ui(ui);
                ui.label("Food");
            });
            ui.horizontal(|ui| {
                let mut score = human.home.last_score;
                egui::DragValue::new(&mut score).ui(ui);
                ui.label("Home");
            });
            ui.horizontal(|ui| {
                let mut score = human.work.as_ref().map(|x| x.last_score).unwrap_or(0.0);
                egui::DragValue::new(&mut score).ui(ui);
                ui.label("Work");
            });

            let market = goria.read::<Market>();
            let itemregistry = goria.read::<ItemRegistry>();

            ui.add_space(10.0);

            let jobopening = itemregistry.id("job-opening");
            for (&item_id, m) in market.iter() {
                let Some(v) = m.capital(id.into()) else { continue };
                if item_id == jobopening {
                    continue;
                }
                let Some(item) = itemregistry.get(item_id) else { continue };

                item_icon(ui, uiworld, item, v);
            }

            follow_button(uiworld, ui, id);
        });
    is_open
}
