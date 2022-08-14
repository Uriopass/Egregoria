use crate::gui::follow::FollowEntity;
use crate::uiworld::UiWorld;
use egregoria::economy::{ItemRegistry, Market, Workers};
use egregoria::map_dynamic::{Itinerary, Router};
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::physics::{Collider, CollisionWorld, Kinematics, PhysicsObject};
use egregoria::souls::desire::{BuyFood, Home, Work};
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::souls::human::HumanDecision;
use egregoria::vehicles::{Vehicle, VehicleID};
use egregoria::{Egregoria, SoulID};

use egregoria::vehicles::trains::{Locomotive, LocomotiveReservation};
use geom::{Transform, Vec2};
use hecs::{Component, Entity};
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};

pub struct InspectRenderer {
    pub entity: Entity,
}

impl InspectRenderer {
    fn inspect_component<T: Component + InspectRenderDefault<T>>(
        &self,
        goria: &Egregoria,
        ui: &Ui<'_>,
    ) {
        let c = goria.comp::<T>(self.entity);
        if let Some(x) = c {
            <T as InspectRenderDefault<T>>::render(
                &[&*x],
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                ui,
                &InspectArgsDefault::default(),
            )
        }
    }

    fn inspect_transform(&self, goria: &Egregoria, uiw: &mut UiWorld, ui: &Ui<'_>) {
        let c = goria.comp(self.entity);
        if let Some(x) = c {
            let mut t = *x;
            if <Transform as InspectRenderDefault<Transform>>::render_mut(
                &mut [&mut t],
                "Transform",
                ui,
                &InspectArgsDefault::default(),
            ) {
                uiw.commands().update_transform(self.entity, t);
            }
        }
    }

    pub fn render(&mut self, uiworld: &mut UiWorld, goria: &Egregoria, ui: &Ui<'_>) {
        let mut custom_ent = self.entity.id() as i32;
        if ui.input_int("enter id directly", &mut custom_ent).build() {
            if let Some(ent) = Entity::from_bits(1 << 32 | custom_ent as u64) {
                if goria.world().contains(ent) {
                    self.entity = ent;
                }
            }
        }

        ui.text(format!("{:?}", self.entity));
        self.inspect_transform(goria, uiworld, ui);
        self.inspect_component::<Vehicle>(goria, ui);
        self.inspect_component::<Pedestrian>(goria, ui);
        self.inspect_component::<Location>(goria, ui);
        self.inspect_component::<Kinematics>(goria, ui);
        self.inspect_component::<Itinerary>(goria, ui);
        self.inspect_component::<Router>(goria, ui);
        self.inspect_component::<HumanDecision>(goria, ui);
        self.inspect_component::<Workers>(goria, ui);
        self.inspect_component::<Work>(goria, ui);
        self.inspect_component::<Home>(goria, ui);
        self.inspect_component::<BuyFood>(goria, ui);
        self.inspect_component::<GoodsCompany>(goria, ui);
        self.inspect_component::<Locomotive>(goria, ui);
        self.inspect_component::<LocomotiveReservation>(goria, ui);

        if goria.comp::<Vehicle>(self.entity).is_some() {
            for (e, loc) in goria.world().query::<&Location>().iter() {
                let loc: &Location = loc;
                if loc == &Location::Vehicle(VehicleID(self.entity))
                    && ui.small_button(&*format!("inspect inside vehicle: {:?}", e))
                {
                    self.entity = e;
                    return;
                }
            }
        }

        if let Some(coll) = goria.comp::<Collider>(self.entity) {
            if let Some((pos, po)) = goria.read::<CollisionWorld>().get(coll.0) {
                if imgui::CollapsingHeader::new("Physics Object").build(ui) {
                    <Vec2 as InspectRenderDefault<Vec2>>::render(
                        &[&pos],
                        "pos",
                        ui,
                        &InspectArgsDefault::default(),
                    );
                    <PhysicsObject as InspectRenderDefault<PhysicsObject>>::render(
                        &[po],
                        "aaaa",
                        ui,
                        &InspectArgsDefault {
                            header: Some(false),
                            indent_children: Some(false),
                            min_value: None,
                            max_value: None,
                            step: None,
                        },
                    )
                }
            } else {
                ui.text_colored([1.0, 0.0, 0.0, 1.0], "Invalid coll handle!");
            }
        }

        if goria.comp::<Kinematics>(self.entity).is_some() {
            let follow = &mut uiworld.write::<FollowEntity>().0;
            if follow.is_none() {
                if ui.small_button("Follow") {
                    follow.replace(self.entity);
                }
            } else if ui.small_button("Unfollow") {
                follow.take();
            }
        }

        let market = goria.read::<Market>();
        let registry = goria.read::<ItemRegistry>();
        let mut capitals = vec![];
        for (kind, market) in market.inner() {
            let cap = unwrap_or!(market.capital(SoulID(self.entity)), continue);
            capitals.push((kind, cap));
        }

        if capitals.is_empty() {
            return;
        }

        if imgui::CollapsingHeader::new("Capital").build(ui) {
            ui.indent();
            ui.columns(2, "markett", false);

            for (kind, cap) in capitals {
                ui.text(&registry[*kind].label);
                ui.next_column();
                ui.text(format!("{}", cap));
                ui.next_column();
            }
        }
    }
}
