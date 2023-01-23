use crate::uiworld::UiWorld;
use egregoria::economy::{ItemRegistry, Market, Workers};
use egregoria::engine_interaction::WorldCommand;
use egregoria::Egregoria;
use egui::{Context, Ui, Widget};

use crate::gui::item_icon;
use egregoria::map::{BuildingID, BuildingKind, Zone};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::souls::goods_company::{GoodsCompany, GoodsCompanyRegistry, Recipe};
use egui_inspect::{Inspect, InspectArgs, InspectVec2Rotation};

pub(crate) fn inspect_building(
    uiworld: &mut UiWorld,
    goria: &Egregoria,
    ui: &Context,
    id: BuildingID,
) {
    let map = goria.map();
    let Some(building) = map.buildings().get(id) else { return; };

    let owner = goria.read::<BuildingInfos>().owner(building.id);
    let goodcompregistry = goria.read::<GoodsCompanyRegistry>();

    egui::Window::new("Building").show(ui, |ui| {
        ui.label(format!("{:?}", building.id));

        match building.kind {
            BuildingKind::House => ui.label("House"),
            BuildingKind::GoodsCompany(id) => {
                let descr = &goodcompregistry.descriptions[id];
                ui.label(&descr.name)
            }
            BuildingKind::RailFretStation => ui.label("Rail Fret Station"),
            BuildingKind::TrainStation => ui.label("Train Station"),
            BuildingKind::ExternalTrading => ui.label("External Trading"),
        };

        if let Some(ref zone) = building.zone {
            let mut cpy = zone.filldir;
            if InspectVec2Rotation::render_mut(&mut cpy, "fill angle", ui, &InspectArgs::default())
            {
                uiworld.commands().push(WorldCommand::UpdateZone {
                    building: id,
                    zone: Zone {
                        filldir: cpy,
                        ..zone.clone()
                    },
                })
            }
        }

        let Some(soul) = owner else { return; };
        let Some(goods) = goria.comp::<GoodsCompany>(soul.0) else { return; };
        let Some(workers) = goria.comp::<Workers>(soul.0) else { return; };

        let market = goria.read::<Market>();
        let itemregistry = goria.read::<ItemRegistry>();
        let max_workers = goods.max_workers;
        egui::ProgressBar::new(workers.0.len() as f32 / max_workers as f32)
            .text(format!("workers: {}/{}", workers.0.len(), max_workers))
            .desired_width(200.0)
            .ui(ui);

        render_recipe(ui, uiworld, goria, &goods.recipe);

        egui::ProgressBar::new(goods.progress)
            .show_percentage()
            .desired_width(200.0)
            .ui(ui);

        ui.add_space(10.0);
        ui.label("Storage");

        let jobopening = itemregistry.id("job-opening");
        for (&id, m) in market.iter() {
            let Some(v) = m.capital(soul) else { continue };
            if id == jobopening && v == 0 {
                continue;
            }
            let Some(item) = itemregistry.get(id) else { continue };

            item_icon(ui, uiworld, item, v);
        }
    });
}

fn render_recipe(ui: &mut Ui, uiworld: &UiWorld, goria: &Egregoria, recipe: &Recipe) {
    let registry = goria.read::<ItemRegistry>();

    if recipe.consumption.is_empty() {
        ui.label("No Inputs");
    } else {
        ui.label(if recipe.consumption.len() == 1 {
            "Input"
        } else {
            "Inputs"
        });
        ui.horizontal(|ui| {
            for &(good, amount) in recipe.consumption.iter() {
                let Some(item) = registry.get(good) else { continue };
                item_icon(ui, uiworld, item, amount);
            }
        });
    }

    if recipe.production.is_empty() {
        ui.label("No Outputs");
    } else {
        ui.label(if recipe.production.len() == 1 {
            "Output"
        } else {
            "Outputs"
        });
        ui.horizontal(|ui| {
            for &(good, amount) in recipe.production.iter() {
                let Some(item) = registry.get(good) else { continue };
                item_icon(ui, uiworld, item, amount);
            }
        });
    }
}
