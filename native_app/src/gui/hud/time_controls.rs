use yakui::widgets::{List, Pad};
use yakui::{
    constrained, opaque, reflow, row, spacer, Alignment, Color, Constraints, CrossAxisAlignment,
    Dim2, MainAxisAlignment, MainAxisSize, Pivot, Vec2,
};

use goryak::{
    blur_bg, button_primary, button_secondary, constrained_viewport, icon_button, monospace,
    on_secondary_container, padx, padxy, secondary_container,
};
use prototypes::GameTime;
use simulation::Simulation;

use crate::gui::windows::settings::Settings;
use crate::gui::GuiState;
use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;

pub fn time_controls(uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::time_controls");
    let time = sim.read::<GameTime>().daytime;
    let warp = &mut uiworld.write::<Settings>().time_warp;
    let mut gui = uiworld.write::<GuiState>();
    let depause_warp = &mut gui.depause_warp;
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

    let time_text = || {
        padx(5.0, || {
            row(|| {
                monospace(on_secondary_container(), format!("Day {}", time.day));
                spacer(1);
                monospace(
                    on_secondary_container(),
                    format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second),
                );
            });
        });
        let mut l = List::row();
        l.main_axis_alignment = MainAxisAlignment::SpaceBetween;
        l.show(|| {
            let mut time_button = |text: &str, b_warp: u32| {
                let mut b = if *warp == b_warp {
                    icon_button(button_primary(text))
                } else {
                    icon_button(button_secondary(text))
                };

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

    reflow(
        Alignment::TOP_LEFT,
        Pivot::TOP_LEFT,
        Dim2::pixels(-10.0, 10.0),
        || {
            constrained_viewport(|| {
                let mut l = List::row();
                l.main_axis_alignment = MainAxisAlignment::End;
                l.show(|| {
                    opaque(|| {
                        blur_bg(secondary_container().with_alpha(0.5), 10.0, || {
                            padxy(10.0, 5.0, || {
                                constrained(
                                    Constraints::loose(Vec2::new(170.0, f32::INFINITY)),
                                    || {
                                        let mut l = List::column();
                                        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                                        l.main_axis_size = MainAxisSize::Min;
                                        l.item_spacing = 5.0;
                                        l.show(time_text);
                                    },
                                );
                            });
                        });
                    });
                });
            });
        },
    );
}
