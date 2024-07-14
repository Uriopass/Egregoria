use std::time::{Duration, Instant};

use goryak::{image_button, minrow, on_secondary_container, textc};
use ordered_float::OrderedFloat;
use prototypes::ItemID;
use yakui::{reflow, Alignment, Color, Dim2, Pivot, Vec2};

use simulation::map_dynamic::ElectricityFlow;
use simulation::Simulation;

use crate::gui::hud::menu::menu_bar;
use crate::gui::hud::time_controls::time_controls;
use crate::gui::hud::toolbox::new_toolbox;
use crate::gui::inspect::new_inspector;
use crate::gui::textures::UiTextures;
use crate::gui::windows::settings::Settings;
use crate::gui::GuiState;
use crate::uiworld::{SaveLoadState, UiWorld};

pub mod chat;
pub mod keybinds;
mod menu;
mod time_controls;
pub mod toolbox;
pub mod windows;

/// Root GUI entrypoint
pub fn render_newgui(uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::render");
    auto_save(uiworld);

    if uiworld.read::<GuiState>().hidden {
        return;
    }

    yakui::column(|| {
        power_errors(uiworld, sim);
        new_toolbox(uiworld, sim);
        menu_bar(uiworld, sim);
        chat::chat(uiworld, sim);
        new_inspector(uiworld, sim);
        uiworld.write::<GuiState>().windows.render(uiworld, sim);
        time_controls(uiworld, sim);
        keybinds::keybind_modal(uiworld, sim)
    });
    //goryak::debug_layout();
}

fn auto_save(uiworld: &UiWorld) {
    let every: Option<Duration> = uiworld.read::<Settings>().auto_save_every.into();
    let mut gui = uiworld.write::<GuiState>();
    if let Some(every) = every {
        if gui.last_save.elapsed() > every {
            uiworld.write::<SaveLoadState>().please_save = true;
            uiworld.save_to_disk();
            gui.last_save = Instant::now();
        }
    }
}

fn power_errors(uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::power_errors");
    let map = sim.map();
    let flow = sim.read::<ElectricityFlow>();

    let no_power_img = uiworld.read::<UiTextures>().get("no_power");

    for network in map.electricity.networks() {
        if !flow.blackout(network.id) {
            continue;
        }

        let mut buildings_with_issues = Vec::with_capacity(network.buildings.len());

        for &building in &network.buildings {
            let Some(b) = map.get(building) else {
                continue;
            };

            let center = b.obb.center();

            let pos = center
                .z(b.height + 20.0 + 1.0 * f32::cos(uiworld.time_always() + center.mag() * 0.05));
            let (screenpos, depth) = uiworld.camera().project(pos);

            let size = 10000.0 / depth;

            buildings_with_issues.push((screenpos, size));
        }

        buildings_with_issues.sort_by_key(|x| OrderedFloat(x.1));

        for (screenpos, size) in buildings_with_issues {
            reflow(
                Alignment::TOP_LEFT,
                Pivot::TOP_LEFT,
                Dim2::pixels(screenpos.x - size * 0.5, screenpos.y - size * 0.5),
                || {
                    let mut image = yakui::widgets::Image::new(no_power_img, Vec2::new(size, size));
                    image.color = Color::WHITE.with_alpha(0.7);
                    image.show();
                },
            );
        }
    }
}

pub fn item_icon_yakui(uiworld: &UiWorld, id: ItemID, multiplier: i32) {
    let item = id.prototype();
    minrow(5.0, || {
        if let Some(id) = uiworld
            .read::<UiTextures>()
            .try_get(&format!("icon/{}", item.name))
        {
            if image_button(
                id,
                Vec2::new(32.0, 32.0),
                Color::WHITE,
                Color::WHITE,
                Color::WHITE,
                "",
            )
            .hovering
            {
                reflow(Alignment::CENTER, Pivot::TOP_LEFT, Dim2::ZERO, || {
                    textc(
                        on_secondary_container(),
                        format!("{} x{}", item.name, multiplier),
                    );
                });
            }
        } else {
            textc(on_secondary_container(), format!("- {} ", &item.label));
        }
        textc(on_secondary_container(), format!("x{multiplier}"))
    });
}
