use crate::gui::inspect::follow_button;
use crate::uiworld::UiWorld;
use goryak::{on_secondary_container, textc, Window};
use simulation::{Simulation, TrainID};
use yakui::widgets::Pad;

pub fn inspect_train(uiworld: &UiWorld, sim: &Simulation, id: TrainID) -> bool {
    let Some(t) = sim.get(id) else {
        return false;
    };

    let mut is_open = true;

    Window {
        title: "Train".into(),
        pad: Pad::all(10.0),
        radius: 10.0,
        opened: &mut is_open,
        child_spacing: 5.0,
    }
    .show(|| {
        if cfg!(debug_assertions) {
            textc(on_secondary_container(), format!("{:?}", id));
        }

        textc(
            on_secondary_container(),
            format!("Going at {:.0}km/h", t.speed.0),
        );

        follow_button(uiworld, id);
    });

    is_open
}
