pub mod desire;
pub mod souls;

use egregoria::api::Router;
use egregoria::engine_interaction::History;
use egregoria::pedestrians::PedestrianID;
use egregoria::Egregoria;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
pub use souls::*;

#[derive(Default)]
pub(crate) struct DebugSoul {
    cur_inspect: Option<PedestrianID>,
    scores: Vec<(&'static str, History)>,
    router: Option<Router>,
}

pub fn debug_souls(ui: &Ui, goria: &mut Egregoria) {
    let mut dsoul = goria.write::<DebugSoul>();
    if let Some(v) = dsoul.cur_inspect {
        ui.text(format!("{:?}", v));

        for (name, h) in &dsoul.scores {
            ui.plot_lines(&imgui::im_str!("{}", name), &h.values)
                .build();
        }

        if let Some(router) = dsoul.router.as_mut() {
            <Router as InspectRenderDefault<Router>>::render_mut(
                &mut [router],
                "router",
                ui,
                &InspectArgsDefault::default(),
            );
        }
    } else {
        ui.text("No pedestrian selected");
    }
}
