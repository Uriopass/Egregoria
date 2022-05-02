use crate::network::NetworkState;
use crate::uiworld::UiWorld;
use egregoria::pedestrians::Pedestrian;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use geom::Camera;
use imgui::Ui;

register_resource_noserialize!(TestFieldProperties);

#[derive(Clone)]
struct TestFieldProperties {
    size: u32,
    spacing: f32,
}

pub fn map(
    window: imgui::Window<'_, &'static str>,
    ui: &Ui<'_>,
    uiworld: &mut UiWorld,
    goria: &Egregoria,
) {
    window.build(ui, || {
        if ui.small_button("load Paris map") {
            uiworld.commands().map_load_paris();
        }
        ui.separator();
        let mut state = uiworld.write::<TestFieldProperties>();

        imgui::Drag::new("size")
            .range(2, 100)
            .build(ui, &mut state.size);

        imgui::Drag::new("spacing")
            .range(30.0, 1000.0)
            .display_format("%.0f")
            .build(ui, &mut state.spacing);

        if ui.small_button("load test field") {
            uiworld.commands().map_load_testfield(
                uiworld.read::<Camera>().pos.xy(),
                state.size,
                state.spacing,
            );
        }

        if matches!(
            *uiworld.read::<NetworkState>(),
            NetworkState::Singleplayer { .. }
        ) && ui.small_button("reset the save")
        {
            uiworld.commands().reset_save();
        }

        ui.text(format!(
            "{} pedestrians",
            goria.world().query::<&Pedestrian>().iter().count()
        ));
        ui.text(format!(
            "{} vehicles",
            goria.world().query::<&Vehicle>().iter().count()
        ));
    });
}

impl Default for TestFieldProperties {
    fn default() -> Self {
        Self {
            size: 10,
            spacing: 150.0,
        }
    }
}
