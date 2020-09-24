pub mod desire;
pub mod souls;

use egregoria::engine_interaction::History;
use egregoria::pedestrians::PedestrianID;
use egregoria::Egregoria;
use imgui::Ui;
pub use souls::*;

#[derive(Default)]
pub(crate) struct DebugSoul {
    cur_inspect: Option<PedestrianID>,
    scores: Vec<(&'static str, History)>,
}

pub fn debug_souls(ui: &Ui, goria: &mut Egregoria) {
    let dsoul = goria.read::<DebugSoul>();
    if let Some(v) = dsoul.cur_inspect {
        ui.text(format!("{:?}", v));

        for (name, h) in &dsoul.scores {
            ui.plot_lines(&imgui::im_str!("{}", name), &h.values)
                .build();
        }
    } else {
        ui.text("No pedestrian selected");
    }
}
