use crate::network::NetworkState;
use crate::uiworld::UiWorld;
use egregoria::pedestrians::Pedestrian;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use geom::Camera;
use imgui::{im_str, Ui};
use legion::IntoQuery;

register_resource_noserialize!(TestFieldProperties);

#[derive(Clone)]
struct TestFieldProperties {
    size: u32,
    spacing: f32,
}

pub fn map(window: imgui::Window, ui: &Ui, uiworld: &mut UiWorld, goria: &Egregoria) {
    window.build(ui, || {
        if ui.small_button(im_str!("load Paris map")) {
            uiworld.commands().map_load_paris();
        }
        ui.separator();
        let mut state = uiworld.write::<TestFieldProperties>();

        imgui::Drag::new(im_str!("size"))
            .range(2..=100)
            .build(ui, &mut state.size);

        imgui::Drag::new(im_str!("spacing"))
            .range(30.0..=1000.0)
            .display_format(im_str!("%.0f"))
            .build(ui, &mut state.spacing);

        if ui.small_button(im_str!("load test field")) {
            uiworld.commands().map_load_testfield(
                uiworld.read::<Camera>().position.xy(),
                state.size,
                state.spacing,
            );
        }

        if matches!(
            *uiworld.read::<NetworkState>(),
            NetworkState::Singleplayer { .. }
        ) && ui.small_button(im_str!("reset the save"))
        {
            uiworld.commands().reset_save();
        }

        ui.text(im_str!(
            "{} pedestrians",
            <&Pedestrian>::query().iter(goria.world()).count()
        ));
        ui.text(im_str!(
            "{} vehicles",
            <&Vehicle>::query().iter(goria.world()).count()
        ));
    })
}

impl Default for TestFieldProperties {
    fn default() -> Self {
        Self {
            size: 10,
            spacing: 150.0,
        }
    }
}
