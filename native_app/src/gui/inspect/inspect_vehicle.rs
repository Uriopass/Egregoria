use crate::gui::inspect::{entity_link, follow_button};
use crate::uiworld::UiWorld;
use egui::Context;
use simulation::transportation::VehicleState;
use simulation::{Simulation, VehicleID};

pub fn inspect_vehicle(
    uiworld: &mut UiWorld,
    sim: &Simulation,
    ui: &Context,
    id: VehicleID,
) -> bool {
    let Some(v) = sim.get(id) else {
        return false;
    };

    let name = format!("{:?}", v.vehicle.kind);

    let mut is_open = true;
    egui::Window::new(name)
        .resizable(false)
        .auto_sized()
        .open(&mut is_open)
        .show(ui, |ui| {
            if cfg!(debug_assertions) {
                ui.label(format!("{:?}", id));
            }

            match v.vehicle.state {
                VehicleState::Parked(_) => {
                    ui.label("Parked");
                }
                VehicleState::Driving => {
                    ui.label(format!("Driving at {:.0}km/h", v.speed.0 * 3.6));
                }
                VehicleState::Panicking(_) => {
                    ui.label("Panicking");
                }
                VehicleState::RoadToPark(_, _, _) => {
                    ui.label("Parking");
                }
            }

            for (human_id, human) in &sim.world().humans {
                if human.router.personal_car == Some(id) {
                    ui.horizontal(|ui| {
                        ui.label("Owned by");
                        entity_link(uiworld, sim, ui, human_id);
                    });
                }
            }

            follow_button(uiworld, ui, id);
        });

    is_open
}
