use yakui::widgets::{List, Pad, PadWidget};
use yakui::{
    colored_box, column, constrained, draggable, offset, pad, reflow, row, spacer, use_state,
    Alignment, Color, Constraints, CrossAxisAlignment, Dim2, MainAxisAlignment, MainAxisSize, Vec2,
};

use goryak::{
    blur_bg, button_primary, constrained_viewport, labelc, monospace, on_primary_container,
    widget_inner,
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

            reflow(Alignment::TOP_LEFT, Dim2::ZERO, || {
                let off = use_state(|| Vec2::ZERO);

                offset(off.get(), || {
                    let v = draggable(|| {
                        blur_bg(goryak::primary_container().with_alpha(0.3), 10.0, || {
                            column(|| {
                                colored_box(
                                    on_primary_container().with_alpha(0.3),
                                    Vec2::new(tweak!(500.0), 50.0),
                                );
                                pad(Pad::all(200.0), || {
                                    labelc(on_primary_container(), "Blurring test!");
                                });
                            });
                        });
                    });
                    if let Some(v) = v.dragging {
                        off.set(v.current);
                    }
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
            row(|| {
                monospace(format!("Day {}", time.day));
                spacer(1);
                monospace(format!(
                    "{:02}:{:02}:{:02}",
                    time.hour, time.minute, time.second
                ));
            });
            row(|| {
                let time_button = |text: &str| {
                    widget_inner::<PadWidget, _, _>(
                        || {
                            let mut b = button_primary(text);
                            b.padding = Pad::balanced(10.0, 3.0);
                            b.show()
                        },
                        Pad::all(3.0),
                    )
                };

                if time_button("||").clicked {
                    *depause_warp = *warp;
                    *warp = 0;
                }
                if time_button("1x").clicked {
                    *warp = 1;
                }
                if time_button("3x").clicked {
                    *warp = 3;
                }
                if time_button("Max").clicked {
                    *warp = 1000;
                }
            });
        };

        reflow(Alignment::TOP_LEFT, Dim2::pixels(0.0, 30.0), || {
            constrained_viewport(|| {
                let mut l = List::row();
                l.main_axis_alignment = MainAxisAlignment::End;
                l.show(|| {
                    blur_bg(goryak::primary_container().with_alpha(0.5), 10.0, || {
                        pad(Pad::all(10.0), || {
                            constrained(
                                Constraints::loose(Vec2::new(200.0, f32::INFINITY)),
                                || {
                                    let mut l = List::column();
                                    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                                    l.main_axis_size = MainAxisSize::Min;
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
