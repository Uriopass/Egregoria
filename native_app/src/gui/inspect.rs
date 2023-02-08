use crate::gui::follow::FollowEntity;
use crate::uiworld::UiWorld;
use egregoria::economy::{ItemRegistry, Market, Workers};
use egregoria::map_dynamic::{DispatchKind, Itinerary, Router};
use egregoria::physics::{Collider, CollisionWorld, PhysicsObject, Speed};
use egregoria::souls::desire::{BuyFood, Home, Work};
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::souls::human::HumanDecision;
use egregoria::transportation::{Location, Pedestrian, Vehicle, VehicleID};
use egregoria::{Egregoria, SoulID};

use egregoria::souls::freight_station::FreightStation;
use egregoria::transportation::train::{Locomotive, LocomotiveReservation};
use egui::{Color32, RichText, Ui};
use egui_inspect::{Inspect, InspectArgs};
use geom::{Transform, Vec2};
use hecs::{Component, Entity};

pub(crate) struct InspectRenderer {
    pub(crate) entity: Entity,
}

impl InspectRenderer {
    fn inspect_component<T: Component + Inspect<T>>(&self, goria: &Egregoria, ui: &mut Ui) {
        let c = goria.comp::<T>(self.entity);
        if let Some(x) = c {
            <T as Inspect<T>>::render(
                &x,
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                ui,
                &InspectArgs::default(),
            )
        }
    }

    fn inspect_transform(&self, goria: &Egregoria, uiw: &mut UiWorld, ui: &mut Ui) {
        let c = goria.comp(self.entity);
        if let Some(x) = c {
            let mut t = *x;
            if <Transform as Inspect<Transform>>::render_mut(
                &mut t,
                "Transform",
                ui,
                &InspectArgs::default(),
            ) {
                uiw.commands().update_transform(self.entity, t);
            }
        }
    }

    pub(crate) fn render(&mut self, uiworld: &mut UiWorld, goria: &Egregoria, ui: &mut Ui) {
        let mut custom_ent = self.entity.id() as i32;

        ui.horizontal(|ui| {
            if ui.add(egui::DragValue::new(&mut custom_ent)).changed() {
                if let Some(ent) = Entity::from_bits(1 << 32 | custom_ent as u64) {
                    if goria.world().contains(ent) {
                        self.entity = ent;
                    }
                }
            }
            ui.label("Entity ID");
        });

        ui.label(format!("{:?}", self.entity));
        self.inspect_transform(goria, uiworld, ui);
        self.inspect_component::<Vehicle>(goria, ui);
        self.inspect_component::<Pedestrian>(goria, ui);
        self.inspect_component::<Location>(goria, ui);
        self.inspect_component::<Speed>(goria, ui);
        self.inspect_component::<Itinerary>(goria, ui);
        self.inspect_component::<Router>(goria, ui);
        self.inspect_component::<HumanDecision>(goria, ui);
        self.inspect_component::<FreightStation>(goria, ui);
        self.inspect_component::<Workers>(goria, ui);
        self.inspect_component::<Work>(goria, ui);
        self.inspect_component::<Home>(goria, ui);
        self.inspect_component::<BuyFood>(goria, ui);
        self.inspect_component::<GoodsCompany>(goria, ui);
        self.inspect_component::<Locomotive>(goria, ui);
        self.inspect_component::<LocomotiveReservation>(goria, ui);
        self.inspect_component::<DispatchKind>(goria, ui);

        if goria.comp::<Vehicle>(self.entity).is_some() {
            for (e, loc) in goria.world().query::<&Location>().iter() {
                let loc: &Location = loc;
                if loc == &Location::Vehicle(VehicleID(self.entity))
                    && ui
                        .small_button(&*format!("inspect inside vehicle: {e:?}"))
                        .clicked()
                {
                    self.entity = e;
                    return;
                }
            }
        }

        if let Some(coll) = goria.comp::<Collider>(self.entity) {
            if let Some((pos, po)) = goria.read::<CollisionWorld>().get(coll.0) {
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
        }

        if goria.comp::<Speed>(self.entity).is_some() {
            let follow = &mut uiworld.write::<FollowEntity>().0;
            if follow.is_none() {
                if ui.small_button("Follow").clicked() {
                    follow.replace(self.entity);
                }
            } else if ui.small_button("Unfollow").clicked() {
                follow.take();
            }
        }

        let market = goria.read::<Market>();
        let registry = goria.read::<ItemRegistry>();
        let mut capitals = vec![];
        let mut borders = vec![];
        let mut sellorders = vec![];
        for (kind, market) in market.inner() {
            let cap = unwrap_or!(market.capital(SoulID(self.entity)), continue);
            capitals.push((kind, cap));
            if let Some(b) = market.buy_order(SoulID(self.entity)) {
                borders.push((kind, b));
            }
            if let Some(s) = market.sell_order(SoulID(self.entity)) {
                sellorders.push((kind, s));
            }
        }

        if !capitals.is_empty() {
            egui::CollapsingHeader::new("Capital").show(ui, |ui| {
                ui.columns(2, |ui| {
                    for (kind, cap) in capitals {
                        ui[0].label(&registry[*kind].label);
                        ui[1].label(format!("{cap}"));
                    }
                });
            });
        }

        if !borders.is_empty() {
            egui::CollapsingHeader::new("Buy orders").show(ui, |ui| {
                ui.columns(2, |ui| {
                    for (kind, b) in borders {
                        ui[0].label(&registry[*kind].label);
                        ui[1].label(format!("{b:#?}"));
                    }
                });
            });
        }

        if !sellorders.is_empty() {
            egui::CollapsingHeader::new("Sell orders").show(ui, |ui| {
                ui.columns(2, |ui| {
                    for (kind, b) in sellorders {
                        ui[0].label(&registry[*kind].label);
                        ui[1].label(format!("{b:#?}"));
                    }
                });
            });
        }
    }
}
