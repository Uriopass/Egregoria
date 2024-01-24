use ordered_float::OrderedFloat;
use yakui::widgets::{CutOut, List, Pad};
use yakui::{
    constrained, reflow, row, spacer, Alignment, Color, Constraints, CrossAxisAlignment, Dim2,
    MainAxisAlignment, MainAxisSize, Vec2,
};

use goryak::{
    blur_bg, button_primary, button_secondary, constrained_viewport, icon_map, monospace, padx,
    padxy, secondary_container,
};
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
                let w = ctx.layout.viewport().size().length() * 0.002;
                yakui::shapes::outline(
                    ctx.paint,
                    ctx.layout.viewport(),
                    w,
                    Color::rgba(255, 0, 0, 128),
                );
            });
        }

        let mut time_text = || {
            padx(5.0, || {
                row(|| {
                    monospace(format!("Day {}", time.day));
                    spacer(1);
                    monospace(format!(
                        "{:02}:{:02}:{:02}",
                        time.hour, time.minute, time.second
                    ));
                });
            });
            let mut l = List::row();
            l.main_axis_alignment = MainAxisAlignment::SpaceBetween;
            l.show(|| {
                let mut time_button = |text: &str, b_warp: u32| {
                    let (mapped, name) = icon_map(text);
                    let mut b = if *warp == b_warp {
                        button_primary(mapped)
                    } else {
                        button_secondary(mapped)
                    };

                    b.style.text.font = name.clone();
                    b.hover_style.text.font = name.clone();
                    b.down_style.text.font = name;

                    b.padding = Pad::balanced(10.0, 3.0);
                    if b.show().clicked {
                        if b_warp == 0 {
                            if *warp == 0 {
                                *warp = *depause_warp;
                            } else {
                                *depause_warp = *warp;
                            }
                        }
                        *warp = b_warp;
                    }
                };

                time_button("pause", 0);
                time_button("play", 1);
                time_button("forward", 3);
                time_button("fast-forward", 1000);
            });
        };

        reflow(Alignment::TOP_LEFT, Dim2::pixels(-10.0, 30.0), || {
            constrained_viewport(|| {
                let mut l = List::row();
                l.main_axis_alignment = MainAxisAlignment::End;
                l.show(|| {
                    blur_bg(secondary_container().with_alpha(0.5), 10.0, || {
                        padxy(10.0, 5.0, || {
                            constrained(
                                Constraints::loose(Vec2::new(170.0, f32::INFINITY)),
                                || {
                                    let mut l = List::column();
                                    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                                    l.main_axis_size = MainAxisSize::Min;
                                    l.item_spacing = tweak!(5.0);
                                    l.show(|| time_text());
                                },
                            );
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

            let mut buildings_with_issues = Vec::with_capacity(network.buildings.len());

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

                buildings_with_issues.push((screenpos, size));
            }

            buildings_with_issues.sort_by_key(|x| OrderedFloat(x.1));

            for (screenpos, size) in buildings_with_issues {
                reflow(
                    Alignment::TOP_LEFT,
                    Dim2::pixels(screenpos.x - size * 0.5, screenpos.y - size * 0.5),
                    || {
                        let mut image =
                            yakui::widgets::Image::new(no_power_img, Vec2::new(size, size));
                        image.color = Color::WHITE.with_alpha(0.7);
                        image.show();
                    },
                );
            }
        }
    }
}
