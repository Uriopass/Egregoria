use egui::Ui;
use egui::{Context, Window};

use crate::debug_gui::debug_window::DebugState;
use egui_inspect::{Inspect, InspectArgs};
use simulation::economy::Market;
use simulation::transportation::Location;
use simulation::{
    AnyEntity, CompanyEnt, FreightStationEnt, HumanEnt, Simulation, SoulID, TrainEnt, VehicleEnt,
    WagonEnt,
};

use crate::gui::follow::FollowEntity;
use crate::gui::InspectedEntity;
use crate::uiworld::UiWorld;

pub fn debug_inspector(ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
    profiling::scope!("hud::inspector");
    let e = unwrap_or!(uiworld.read::<InspectedEntity>().e, return);

    let force_debug_inspect = uiworld.read::<DebugState>().debug_inspector;

    let mut is_open = true;
    if force_debug_inspect {
        Window::new("Inspect")
            .default_size([400.0, 500.0])
            .default_pos([30.0, 160.0])
            .resizable(true)
            .open(&mut is_open)
            .show(ui, |ui| {
                let mut ins = InspectRenderer { entity: e };
                ins.render(uiworld, sim, ui);
                uiworld.write::<InspectedEntity>().e = Some(ins.entity);
            });
    }

    if !is_open {
        uiworld.write::<InspectedEntity>().e = None;
    }
}

/// Inspect window
/// Allows to debug_inspect an entity
pub struct InspectRenderer {
    pub entity: AnyEntity,
}

impl InspectRenderer {
    pub fn render(&mut self, uiworld: &UiWorld, sim: &Simulation, ui: &mut Ui) {
        let entity = self.entity;
        ui.label(format!("{:?}", self.entity));

        let args = InspectArgs {
            indent_children: Some(false),
            ..Default::default()
        };

        match entity {
            AnyEntity::VehicleID(x) => {
                <VehicleEnt as Inspect<VehicleEnt>>::render(sim.get(x).unwrap(), "", ui, &args)
            }
            AnyEntity::TrainID(x) => {
                <TrainEnt as Inspect<TrainEnt>>::render(sim.get(x).unwrap(), "", ui, &args)
            }
            AnyEntity::WagonID(x) => {
                <WagonEnt as Inspect<WagonEnt>>::render(sim.get(x).unwrap(), "", ui, &args)
            }
            AnyEntity::FreightStationID(x) => {
                <FreightStationEnt as Inspect<FreightStationEnt>>::render(
                    sim.get(x).unwrap(),
                    "",
                    ui,
                    &args,
                )
            }
            AnyEntity::CompanyID(x) => {
                <CompanyEnt as Inspect<CompanyEnt>>::render(sim.get(x).unwrap(), "", ui, &args)
            }
            AnyEntity::HumanID(x) => {
                <HumanEnt as Inspect<HumanEnt>>::render(sim.get(x).unwrap(), "", ui, &args)
            }
        }

        if let AnyEntity::VehicleID(id) = entity {
            for (hid, h) in sim.world().humans.iter() {
                if h.location == Location::Vehicle(id)
                    && ui
                        .small_button(&*format!("debug_inspect inside vehicle: {hid:?}"))
                        .clicked()
                {
                    self.entity = hid.into();
                    return;
                }
            }
        }

        /*
        if let Some(coll) = sim.comp::<Collider>(self.entity) {
            if let Some((pos, po)) = sim.read::<CollisionWorld>().get(coll.0) {
                egui::CollapsingHeader::new("Physics Object").show(ui, |ui| {
                    <Vec2 as Inspect<Vec2>>::render(&pos, "pos", ui, &InspectArgs::default());
                    <PhysicsObject as Inspect<PhysicsObject>>::render(
                        po,
                        "aaaa",
                        ui,
                        &InspectArgs {
                            header: Some(false),
                            indent_children: Some(false),
                            min_value: None,
                            max_value: None,
                            step: None,
                        },
                    )
                });
            } else {
                ui.label(
                    RichText::new("Invalid coll handle")
                        .color(Color32::from_rgba_unmultiplied(255, 0, 0, 255)),
                );
            }
        }*/

        {
            let mut follow = uiworld.write::<FollowEntity>();
            if ui.small_button("Follow").clicked() {
                follow.0.replace(entity);
            }
        }

        if let Ok(soul) = SoulID::try_from(entity) {
            let market = sim.read::<Market>();
            let mut capitals = vec![];
            let mut borders = vec![];
            let mut sellorders = vec![];
            for (kind, market) in market.inner() {
                let cap = unwrap_or!(market.capital(soul), continue);
                capitals.push((kind, cap));
                if let Some(b) = market.buy_order(soul) {
                    borders.push((kind, b));
                }
                if let Some(s) = market.sell_order(soul) {
                    sellorders.push((kind, s));
                }
            }

            if !capitals.is_empty() {
                egui::CollapsingHeader::new("Capital").show(ui, |ui| {
                    ui.columns(2, |ui| {
                        for (kind, cap) in capitals {
                            ui[0].label(&kind.prototype().label);
                            ui[1].label(format!("{cap}"));
                        }
                    });
                });
            }

            if !borders.is_empty() {
                egui::CollapsingHeader::new("Buy orders").show(ui, |ui| {
                    ui.columns(2, |ui| {
                        for (kind, b) in borders {
                            ui[0].label(&kind.prototype().label);
                            ui[1].label(format!("{b:#?}"));
                        }
                    });
                });
            }

            if !sellorders.is_empty() {
                egui::CollapsingHeader::new("Sell orders").show(ui, |ui| {
                    ui.columns(2, |ui| {
                        for (kind, b) in sellorders {
                            ui[0].label(&kind.prototype().label);
                            ui[1].label(format!("{b:#?}"));
                        }
                    });
                });
            }
        }
    }
}
