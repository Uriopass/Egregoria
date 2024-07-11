use yakui::widgets::{Button, List, Pad};
use yakui::{
    colored_box_container, column, image, opaque, reflow, spacer, Alignment, Color,
    CrossAxisAlignment, Dim2, MainAxisAlignment, MainAxisSize, Pivot, Vec2,
};

use goryak::{
    blur_bg, button_primary, constrained_viewport, fixed_spacer, icon_button, image_button,
    monospace, on_primary, outline, padxy, primary, primary_container, round_rect,
    secondary_container,
};
use simulation::Simulation;

use crate::gui::textures::UiTextures;
use crate::gui::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;

pub mod building;
pub mod roadbuild;
pub mod roadedit;
pub mod terraforming;
pub mod train;

pub fn new_toolbox(uiworld: &UiWorld, sim: &Simulation) {
    if uiworld
        .read::<InputMap>()
        .just_act
        .contains(&InputAction::Close)
    {
        *uiworld.write::<Tool>() = Tool::Hand;
    }

    reflow(Alignment::TOP_LEFT, Pivot::TOP_LEFT, Dim2::ZERO, || {
        constrained_viewport(|| {
            let mut l = List::column();
            l.cross_axis_alignment = CrossAxisAlignment::Stretch;
            l.show(|| {
                spacer(1);
                opaque(|| {
                    let mut l = List::column();
                    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                    l.show(|| {
                        let mut needs_outline = false;
                        blur_bg(primary_container().with_alpha(0.3), 0.0, || {
                            needs_outline = tool_properties(uiworld, sim);
                        });
                        if needs_outline {
                            colored_box_container(outline().with_alpha(0.5), || {
                                fixed_spacer((1.0, 1.0));
                            });
                        }
                        blur_bg(secondary_container().with_alpha(0.3), 0.0, || {
                            padxy(0.0, 10.0, || {
                                let mut l = List::row();
                                l.main_axis_alignment = MainAxisAlignment::Center;
                                l.item_spacing = 10.0;
                                l.show(|| {
                                    tools_list(uiworld);
                                });
                            });
                        });
                    });
                });
            });
        });
    });
}

fn tool_properties(uiw: &UiWorld, _sim: &Simulation) -> bool {
    let tool = *uiw.read::<Tool>();

    match tool {
        Tool::Hand => return false,
        Tool::Bulldozer => return false,
        Tool::LotBrush => return false,
        Tool::RoadbuildStraight | Tool::RoadbuildCurved => {
            roadbuild::roadbuild_properties(uiw);
        }
        Tool::RoadEditor => {
            roadedit::roadedit_properties(uiw);
        }
        Tool::SpecialBuilding => {
            building::special_building_properties(uiw);
        }
        Tool::Train => {
            train::train_properties(uiw);
        }
        Tool::Terraforming => {
            terraforming::terraform_properties(uiw);
        }
    }
    true
}

fn tools_list(uiworld: &UiWorld) {
    let tools = [
        ("toolbar_straight_road", Tool::RoadbuildStraight),
        ("toolbar_curved_road", Tool::RoadbuildCurved),
        ("toolbar_road_edit", Tool::RoadEditor),
        ("toolbar_housetool", Tool::LotBrush),
        ("toolbar_companies", Tool::SpecialBuilding),
        ("toolbar_bulldozer", Tool::Bulldozer),
        ("toolbar_train", Tool::Train),
        ("toolbar_terraform", Tool::Terraforming),
    ];

    for (name, tool) in &tools {
        column(|| {
            let (default_col, hover_col) = if *tool == *uiworld.read::<Tool>() {
                let c = primary().lerp(&Color::WHITE, 0.3);
                (c, c)
            } else {
                (Color::WHITE, Color::WHITE.with_alpha(0.7))
            };
            if image_button(
                uiworld.read::<UiTextures>().get(name),
                Vec2::new(64.0, 64.0),
                default_col,
                hover_col,
                primary(),
                "",
            )
            .clicked
            {
                *uiworld.write::<Tool>() = *tool;
            }

            if *tool == *uiworld.read::<Tool>() {
                select_triangle(uiworld);
            }
        });
    }
}

pub(crate) fn select_triangle(uiworld: &UiWorld) {
    reflow(
        Alignment::CENTER_LEFT,
        Pivot::TOP_LEFT,
        Dim2::pixels(0.0, 32.0),
        || {
            image(
                uiworld.read::<UiTextures>().get("select_triangle_under"),
                Vec2::new(64.0, 10.0),
            );
        },
    );
}

pub fn updown_button(text: &str) -> Button {
    let mut b = icon_button(button_primary(text));
    b.padding = Pad::balanced(5.0, 2.0);
    b.style.text.font_size = 13.0;
    b.down_style.text.font_size = 13.0;
    b.hover_style.text.font_size = 13.0;
    b
}

pub fn updown_value(v: &mut f32, step: f32, suffix: &'static str) -> bool {
    let mut changed = false;
    let mut l = List::column();
    l.cross_axis_alignment = CrossAxisAlignment::Center;
    l.main_axis_size = MainAxisSize::Min;
    l.item_spacing = 3.0;
    l.show(|| {
        if updown_button("caret-up").show().clicked {
            *v += step;
            changed = true;
        }
        round_rect(3.0, primary(), || {
            padxy(5.0, 1.0, || {
                monospace(on_primary(), format!("{:.0}{}", *v, suffix));
            });
        });
        if updown_button("caret-down").show().clicked {
            *v -= step;
            changed = true;
        }
    });
    changed
}
