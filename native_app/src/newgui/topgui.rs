use yakui::widgets::{List, Pad};
use yakui::{
    colored_box_container, label, pad, reflow, row, Alignment, Color, Dim2, MainAxisAlignment,
    MainAxisSize, Vec2,
};

use goryak::{blur_bg, button_primary, constrained_viewport, labelc, on_primary_container, text};
use prototypes::GameTime;
use simulation::map_dynamic::ElectricityFlow;
use simulation::Simulation;

use crate::gui::windows::settings::Settings;
use crate::gui::{Gui, UiTextures};
use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;

impl Gui {
    /// Root GUI entrypoint
    pub fn render_newgui(&mut self, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("topgui::render");
        self.auto_save(uiworld);

        if self.hidden {
            return;
        }

        yakui::column(|| {
            self.time_controls(uiworld, sim);
            self.power_errors(uiworld, sim);

            reflow(Alignment::CENTER, Dim2::pixels(-220.0, -220.0), || {
                blur_bg(goryak::primary_container().with_alpha(0.3), || {
                    pad(Pad::all(100.0), || {
                        labelc(on_primary_container(), "Blurring test!");
                    });
                });
            });
        });
    }

    pub fn time_controls(&mut self, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("topgui::time_controls");
        let time = sim.read::<GameTime>().daytime;
        let warp = &mut uiworld.write::<Settings>().time_warp;
        let depause_warp = &mut self.depause_warp;
        if uiworld
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::PausePlay)
        {
            if *warp == 0 {
                *warp = *depause_warp;
            } else {
                *depause_warp = *warp;
                *warp = 0;
            }
        }

        if *warp == 0 {
            yakui::canvas(|ctx| {
                yakui::shapes::outline(
                    ctx.paint,
                    ctx.layout.viewport(),
                    2.0,
                    Color::rgba(255, 0, 0, 196),
                );
            });
        }

        let mut time_text = || {
            row(|| {
                text(format!(" Day {}", time.day));

                text(format!(
                    "{:02}:{:02}:{:02}",
                    time.hour, time.minute, time.second
                ));
            });
            row(|| {
                if button_primary("||").clicked {
                    *depause_warp = *warp;
                    *warp = 0;
                }
                if button_primary("1x").clicked {
                    *warp = 1;
                }
                if button_primary("3x").clicked {
                    *warp = 3;
                }
                if button_primary("Max").clicked {
                    *warp = 1000;
                }
            });
        };

        reflow(Alignment::TOP_LEFT, Dim2::pixels(0.0, 40.0), || {
            constrained_viewport(|| {
                let mut l = List::row();
                l.main_axis_alignment = MainAxisAlignment::End;
                l.show(|| {
                    blur_bg(goryak::primary_container().with_alpha(0.5), || {
                        pad(Pad::all(3.0), || {
                            let mut l = List::column();
                            l.main_axis_size = MainAxisSize::Min;
                            l.show(|| time_text());
                        });
                    });
                });
            });
        });
    }

    fn power_errors(&mut self, uiworld: &UiWorld, sim: &Simulation) {
        profiling::scope!("topgui::power_errors");
        let map = sim.map();
        let flow = sim.read::<ElectricityFlow>();

        let no_power_img = uiworld.read::<UiTextures>().get_yakui("no_power");

        for network in map.electricity.networks() {
            if !flow.blackout(network.id) {
                continue;
            }
            for &building in &network.buildings {
                let Some(b) = map.get(building) else {
                    continue;
                };

                let center = b.obb.center();

                let pos = center.z(b.height
                    + 20.0
                    + 1.0 * f32::cos(uiworld.time_always() + center.mag() * 0.05));
                let (screenpos, depth) = uiworld.camera().project(pos);

                let size = 10000.0 / depth;

                yakui::reflow(
                    Alignment::TOP_LEFT,
                    Dim2::pixels(screenpos.x - size * 0.5, screenpos.y - size * 0.5),
                    || {
                        let mut image =
                            yakui::widgets::Image::new(no_power_img, Vec2::new(size, size));
                        image.color = Color::WHITE.with_alpha(0.5);
                        image.show();
                    },
                );
            }
        }
    }
}
