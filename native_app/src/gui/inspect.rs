use crate::gui::follow::FollowEntity;
use crate::uiworld::UiWorld;
use egregoria::economy::Market;
use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::physics::{Collider, Kinematics};
use egregoria::rendering::assets::AssetRender;
use egregoria::rendering::meshrender_component::MeshRender;
use egregoria::souls::desire::{BuyFood, Desire, Home, Work};
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::vehicles::Vehicle;
use egregoria::{Egregoria, SoulID};
use geom::Transform;
use imgui::im_str;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use legion::storage::Component;
use legion::Entity;

pub struct InspectRenderer {
    pub entity: Entity,
}

impl InspectRenderer {
    fn inspect_component<T: Component + InspectRenderDefault<T>>(
        &self,
        goria: &Egregoria,
        ui: &Ui,
    ) {
        let c: Option<&T> = goria.comp::<T>(self.entity);
        if let Some(x) = c {
            <T as InspectRenderDefault<T>>::render(
                &[x],
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                ui,
                &InspectArgsDefault::default(),
            )
        }
    }

    pub fn render(&self, uiworld: &mut UiWorld, goria: &Egregoria, ui: &Ui) {
        ui.text(im_str!("{:?}", self.entity));
        self.inspect_component::<Transform>(goria, ui);
        self.inspect_component::<Vehicle>(goria, ui);
        self.inspect_component::<Pedestrian>(goria, ui);
        self.inspect_component::<Location>(goria, ui);
        self.inspect_component::<AssetRender>(goria, ui);
        self.inspect_component::<MeshRender>(goria, ui);
        self.inspect_component::<Kinematics>(goria, ui);
        self.inspect_component::<Collider>(goria, ui);
        self.inspect_component::<Itinerary>(goria, ui);
        self.inspect_component::<Desire<Work>>(goria, ui);
        self.inspect_component::<Desire<Home>>(goria, ui);
        self.inspect_component::<Desire<BuyFood>>(goria, ui);
        self.inspect_component::<GoodsCompany>(goria, ui);

        if goria.comp::<Kinematics>(self.entity).is_some() {
            let follow = &mut uiworld.write::<FollowEntity>().0;
            if follow.is_none() {
                if ui.small_button(im_str!("Follow")) {
                    follow.replace(self.entity);
                }
            } else if ui.small_button(im_str!("Unfollow")) {
                follow.take();
            }
        }

        let market = goria.read::<Market>();
        let mut capitals = vec![];
        for (kind, market) in market.inner() {
            let cap = unwrap_or!(market.capital(SoulID(self.entity)), continue);
            capitals.push((kind, cap));
        }

        if capitals.is_empty() {
            return;
        }

        if imgui::CollapsingHeader::new(im_str!("Capital")).build(ui) {
            ui.indent();
            ui.columns(2, im_str!("markett"), false);

            for (kind, cap) in capitals {
                ui.text(im_str!("{}", kind));
                ui.next_column();
                ui.text(im_str!("{}", cap));
                ui.next_column();
            }
        }
    }
}
