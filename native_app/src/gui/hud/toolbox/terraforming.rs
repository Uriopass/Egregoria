use yakui::widgets::List;
use yakui::{column, CrossAxisAlignment, MainAxisAlignment, Vec2};

use goryak::{fixed_spacer, padxy, primary_image_button};
use simulation::map::TerraformKind;

use crate::gui::hud::toolbox::{select_triangle, updown_value};
use crate::gui::terraforming::TerraformingResource;
use crate::gui::textures::UiTextures;
use crate::uiworld::UiWorld;

pub fn terraform_properties(uiw: &UiWorld) {
    let state = &mut *uiw.write::<TerraformingResource>();

    padxy(0.0, 10.0, || {
        let mut l = List::row();
        l.main_axis_alignment = MainAxisAlignment::Center;
        l.cross_axis_alignment = CrossAxisAlignment::Center;
        l.item_spacing = 10.0;
        l.show(|| {
            let texs = uiw.read::<UiTextures>();

            let terraform_choices = &[
                (
                    TerraformKind::Elevation,
                    "Elevation (up/down)",
                    "terraforming_raise_lower",
                ),
                (TerraformKind::Smooth, "Smooth", "terraforming_smooth"),
                (TerraformKind::Level, "Level", "terraforming_level"),
                (TerraformKind::Slope, "Slope", "terraforming_slope"),
                (TerraformKind::Erode, "Erode", "terraforming_erode"),
            ];

            for (kind, label, icon) in terraform_choices {
                column(|| {
                    let enabled = state.kind == *kind;
                    if primary_image_button(texs.get(icon), Vec2::new(64.0, 64.0), enabled, *label)
                        .clicked
                    {
                        state.kind = *kind;
                    }

                    if enabled {
                        select_triangle(uiw);
                    }
                });
            }

            fixed_spacer((30.0, 0.0));

            let radius_choices = &[
                (200.0, "200m", "terraforming_radius_small"),
                (400.0, "400m", "terraforming_radius_medium"),
                (700.0, "700m", "terraforming_radius_large"),
            ];

            for (radius, label, icon) in radius_choices {
                column(|| {
                    let enabled = state.radius == *radius;
                    if primary_image_button(texs.get(icon), Vec2::new(64.0, 64.0), enabled, *label)
                        .clicked
                    {
                        state.radius = *radius;
                    }

                    if enabled {
                        select_triangle(uiw);
                    }
                });
            }

            let step = if state.radius < 150.0 {
                10.0
            } else if state.radius < 500.0 {
                50.0
            } else {
                100.0
            };

            updown_value(&mut state.radius, step, "m");

            fixed_spacer((30.0, 0.0));

            let amount_choices = &[
                (100.0, "Small", "terraforming_speed_low"),
                (300.0, "Medium", "terraforming_speed_medium"),
                (500.0, "Large", "terraforming_speed_large"),
            ];

            for (amount, label, icon) in amount_choices {
                column(|| {
                    let enabled = state.amount == *amount;
                    if primary_image_button(texs.get(icon), Vec2::new(64.0, 64.0), enabled, *label)
                        .clicked
                    {
                        state.amount = *amount;
                    }

                    if enabled {
                        select_triangle(uiw);
                    }
                });
            }

            updown_value(&mut state.amount, 100.0, "");
        });
    });
}
