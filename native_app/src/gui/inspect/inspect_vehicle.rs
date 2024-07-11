use crate::gui::inspect::{entity_link, follow_button};
use crate::uiworld::UiWorld;
use goryak::{minrow, on_secondary_container, textc, Window};
use simulation::transportation::VehicleState;
use simulation::{Simulation, VehicleID};
use yakui::widgets::Pad;

pub fn inspect_vehicle(uiworld: &UiWorld, sim: &Simulation, id: VehicleID) -> bool {
    let Some(v) = sim.get(id) else {
        return false;
    };

    let name = format!("{:?}", v.vehicle.kind);

    let mut is_open = true;
    Window {
        title: name.into(),
        pad: Pad::all(10.0),
        radius: 10.0,
        opened: &mut is_open,
        child_spacing: 5.0,
    }
    .show(|| {
        if cfg!(debug_assertions) {
            textc(on_secondary_container(), format!("{:?}", id));
        }

        match v.vehicle.state {
            VehicleState::Parked(_) => {
                textc(on_secondary_container(), "Parked");
            }
            VehicleState::Driving => {
                textc(
                    on_secondary_container(),
                    format!("Driving at {:.0}km/h", v.speed.0 * 3.6),
                );
            }
            VehicleState::Panicking(_) => {
                textc(on_secondary_container(), "Panicking");
            }
            VehicleState::RoadToPark(_, _, _) => {
                textc(on_secondary_container(), "Parking");
            }
        }

        for (human_id, human) in &sim.world().humans {
            if human.router.personal_car == Some(id) {
                minrow(5.0, || {
                    textc(on_secondary_container(), "Owned by");
                    entity_link(uiworld, sim, human_id);
                });
            }
        }

        follow_button(uiworld, id);
    });

    is_open
}
