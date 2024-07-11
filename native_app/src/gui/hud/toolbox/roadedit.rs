use yakui::widgets::List;
use yakui::{
    column, image, reflow, Alignment, CrossAxisAlignment, Dim2, MainAxisAlignment, Pivot, Vec2,
};

use goryak::{padxy, primary_image_button};
use simulation::map::LightPolicy;

use crate::gui::hud::toolbox;
use crate::gui::hud::toolbox::select_triangle;
use crate::gui::roadeditor::RoadEditorResource;
use crate::gui::textures::UiTextures;
use crate::uiworld::UiWorld;

pub fn roadedit_properties(uiw: &UiWorld) {
    let state = &mut *uiw.write::<RoadEditorResource>();
    let Some(ref mut v) = state.inspect else {
        return;
    };

    padxy(0.0, 10.0, || {
        let mut l = List::row();
        l.main_axis_alignment = MainAxisAlignment::Center;
        l.cross_axis_alignment = CrossAxisAlignment::Center;
        l.item_spacing = 10.0;
        l.show(|| {
            let texs = uiw.read::<UiTextures>();

            let light_policy_choices = &[
                (LightPolicy::NoLights, "No lights", "roadedit_no_light"),
                (LightPolicy::Lights, "Traffic lights", "roadedit_light"),
                (LightPolicy::StopSigns, "Stop signs", "roadedit_stop_sign"),
                (LightPolicy::Auto, "Auto", "roadedit_auto"),
            ];

            for (policy, label, icon) in light_policy_choices {
                column(|| {
                    let enabled = v.light_policy == *policy;
                    if primary_image_button(texs.get(icon), Vec2::new(64.0, 64.0), enabled, *label)
                        .clicked
                    {
                        v.light_policy = *policy;
                        state.dirty = true;
                    }

                    if enabled {
                        select_triangle(uiw);
                    }
                });
            }

            let mut has_roundabout = v.turn_policy.roundabout.is_some();

            let turn_policies = [
                (
                    &mut v.turn_policy.left_turns,
                    "Left turns",
                    "roadedit_left_turn",
                ),
                (
                    &mut v.turn_policy.back_turns,
                    "Back turns",
                    "roadedit_back_turn",
                ),
                (
                    &mut v.turn_policy.crosswalks,
                    "Crosswalks",
                    "roadedit_crosswalk",
                ),
                (&mut has_roundabout, "Roundabout", "roadedit_roundabout"),
            ];

            for (enabled, label, icon) in turn_policies {
                column(|| {
                    if primary_image_button(texs.get(icon), Vec2::new(64.0, 64.0), *enabled, label)
                        .clicked
                    {
                        *enabled = !*enabled;
                        state.dirty = true;
                    }

                    if !*enabled {
                        reflow(
                            Alignment::TOP_LEFT,
                            Pivot::TOP_LEFT,
                            Dim2::pixels(0.0, 0.0),
                            || {
                                image(texs.get("roadedit_forbidden"), Vec2::new(64.0, 64.0));
                            },
                        );
                    }
                });
            }

            if has_roundabout != v.turn_policy.roundabout.is_some() {
                v.turn_policy.roundabout = if has_roundabout {
                    Some(Default::default())
                } else {
                    None
                };
                state.dirty = true;
            }

            if let Some(ref mut roundabout) = v.turn_policy.roundabout {
                state.dirty |= toolbox::updown_value(&mut roundabout.radius, 2.0, "m");
            }
        });
    });
}
