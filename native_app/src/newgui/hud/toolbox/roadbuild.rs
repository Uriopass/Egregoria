use crate::gui::UiTextures;
use crate::newgui::roadbuild::RoadBuildResource;
use crate::uiworld::UiWorld;
use goryak::{
    button_primary, icon_button, image_button, monospace, on_primary, padxy, primary, round_rect,
};
use simulation::map::LanePatternBuilder;
use yakui::widgets::{Button, List, Pad};
use yakui::{
    image, reflow, Alignment, Color, CrossAxisAlignment, Dim2, MainAxisAlignment, MainAxisSize,
    Vec2,
};

pub fn roadbuild_properties(uiw: &UiWorld) {
    let mut state = uiw.write::<RoadBuildResource>();

    padxy(0.0, 10.0, || {
        let mut l = List::row();
        l.main_axis_alignment = MainAxisAlignment::Center;
        l.cross_axis_alignment = CrossAxisAlignment::Center;
        l.item_spacing = 10.0;
        l.show(|| {
            // Snap to grid
            let (default_col, hover_col) = if state.snap_to_grid {
                let c = primary().lerp(&Color::WHITE, 0.3);
                (c, c.with_alpha(0.7))
            } else {
                (Color::WHITE.with_alpha(0.3), Color::WHITE.with_alpha(0.5))
            };
            if image_button(
                uiw.read::<UiTextures>().get_yakui("snap_grid"),
                Vec2::new(32.0, 32.0),
                default_col,
                hover_col,
                primary(),
                "snap to grid",
            )
            .clicked
            {
                state.snap_to_grid = !state.snap_to_grid;
            }

            // Road elevation
            let mut l = List::column();
            l.cross_axis_alignment = CrossAxisAlignment::Center;
            l.main_axis_size = MainAxisSize::Min;
            l.item_spacing = 3.0;
            l.show(|| {
                if updown_button("caret-up").show().clicked {
                    state.height_offset += 2.0;
                }
                round_rect(3.0, primary(), || {
                    padxy(5.0, 2.0, || {
                        monospace(on_primary(), format!("{:.0}m", state.height_offset));
                    });
                });
                if updown_button("caret-down").show().clicked {
                    state.height_offset -= 2.0;
                }
            });

            // image name, label, builder
            let builders: &[(&str, &str, LanePatternBuilder)] = &[
                ("roadtypes_street", "Street", LanePatternBuilder::new()),
                (
                    "roadtypes_street_1way",
                    "Street one-way",
                    LanePatternBuilder::new().one_way(true),
                ),
                (
                    "roadtypes_avenue",
                    "Avenue",
                    LanePatternBuilder::new().n_lanes(2).speed_limit(13.0),
                ),
                (
                    "roadtypes_avenue_1way",
                    "Avenue one-way",
                    LanePatternBuilder::new()
                        .n_lanes(2)
                        .one_way(true)
                        .speed_limit(13.0),
                ),
                (
                    "roadtypes_drive",
                    "Drive",
                    LanePatternBuilder::new()
                        .parking(false)
                        .sidewalks(false)
                        .speed_limit(13.0),
                ),
                (
                    "roadtypes_drive_1way",
                    "Drive one-way",
                    LanePatternBuilder::new()
                        .parking(false)
                        .sidewalks(false)
                        .one_way(true)
                        .speed_limit(13.0),
                ),
                (
                    "roadtypes_highway",
                    "Highway",
                    LanePatternBuilder::new()
                        .n_lanes(3)
                        .speed_limit(25.0)
                        .parking(false)
                        .sidewalks(false),
                ),
                (
                    "roadtypes_highway_1way",
                    "Highway one-way",
                    LanePatternBuilder::new()
                        .n_lanes(3)
                        .speed_limit(25.0)
                        .parking(false)
                        .sidewalks(false)
                        .one_way(true),
                ),
                (
                    "roadtypes_rail",
                    "Rail",
                    LanePatternBuilder::new().rail(true),
                ),
                (
                    "roadtypes_rail_1way",
                    "Rail one-way",
                    LanePatternBuilder::new().rail(true).one_way(true),
                ),
            ];

            for (icon, label, builder) in builders {
                let mut l = List::column();
                l.main_axis_size = MainAxisSize::Min;
                l.show(|| {
                    let is_active = &state.pattern_builder == builder;
                    let (default_col, hover_col) = if is_active {
                        let c = Color::WHITE.adjust(0.5);
                        (c, c)
                    } else {
                        (Color::WHITE, Color::WHITE.with_alpha(0.7))
                    };
                    if image_button(
                        uiw.read::<UiTextures>().get_yakui(icon),
                        Vec2::new(64.0, 64.0),
                        default_col,
                        hover_col,
                        primary(),
                        label,
                    )
                    .clicked
                    {
                        state.pattern_builder = *builder;
                    }

                    if is_active {
                        reflow(Alignment::CENTER_LEFT, Dim2::pixels(0.0, 32.0), || {
                            image(
                                uiw.read::<UiTextures>().get_yakui("select_triangle_under"),
                                Vec2::new(64.0, 10.0),
                            );
                        });
                    }
                });
            }
        });
    });
}

pub fn updown_button(text: &str) -> Button {
    let mut b = icon_button(button_primary(text));
    b.padding = Pad::balanced(5.0, 3.0);
    b.style.text.font_size = 13.0;
    b.down_style.text.font_size = 13.0;
    b.hover_style.text.font_size = 13.0;
    b
}
