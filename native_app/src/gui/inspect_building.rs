use crate::uiworld::UiWorld;
use egregoria::economy::{ItemRegistry, Market, Workers};
use egregoria::engine_interaction::WorldCommand;
use egregoria::Egregoria;
use egui::{Context, Ui, Widget};

use crate::gui::{item_icon, InspectedEntity};
use egregoria::map::{Building, BuildingID, BuildingKind, Zone, MAX_ZONE_AREA};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::souls::freight_station::{FreightStation, FreightTrainState};
use egregoria::souls::goods_company::{GoodsCompany, GoodsCompanyRegistry, Recipe};
use egui_inspect::{Inspect, InspectArgs, InspectVec2Rotation};

/// Inspect a specific building, showing useful information about it
pub(crate) fn inspect_building(
    uiworld: &mut UiWorld,
    goria: &Egregoria,
    ui: &Context,
    id: BuildingID,
) {
    let map = goria.map();
    let Some(building) = map.buildings().get(id) else { return; };
    let gregistry = goria.read::<GoodsCompanyRegistry>();

    let title: &str = match building.kind {
        BuildingKind::House => "House",
        BuildingKind::GoodsCompany(id) => &gregistry.descriptions[id].name,
        BuildingKind::RailFreightStation => "Rail Freight Station",
        BuildingKind::TrainStation => "Train Station",
        BuildingKind::ExternalTrading => "External Trading",
    };

    egui::Window::new(title)
        .resizable(false)
        .auto_sized()
        .show(ui, |ui| {
            if cfg!(debug_assertions) {
                ui.label(format!("{:?}", building.id));
            }

            match building.kind {
                BuildingKind::House => render_house(ui, uiworld, goria, building),
                BuildingKind::GoodsCompany(_) => {
                    render_goodscompany(ui, uiworld, goria, building);
                }
                BuildingKind::RailFreightStation => {
                    render_freightstation(ui, uiworld, goria, building);
                }
                BuildingKind::TrainStation => {}
                BuildingKind::ExternalTrading => {}
            };

            if let Some(ref zone) = building.zone {
                let mut cpy = zone.filldir;
                if InspectVec2Rotation::render_mut(
                    &mut cpy,
                    "fill angle",
                    ui,
                    &InspectArgs::default(),
                ) {
                    uiworld.commands().push(WorldCommand::UpdateZone {
                        building: id,
                        zone: Zone {
                            filldir: cpy,
                            ..zone.clone()
                        },
                    })
                }
                egui::ProgressBar::new(zone.area / MAX_ZONE_AREA)
                    .text(format!("area: {}/{}", zone.area, MAX_ZONE_AREA))
                    .desired_width(200.0)
                    .ui(ui);
            }
        });
}

fn render_house(ui: &mut Ui, uiworld: &mut UiWorld, goria: &Egregoria, b: &Building) {
    let binfos = goria.read::<BuildingInfos>();
    let Some(info) = binfos.get(b.id) else { return; };
    let Some(owner) = info.owner else { return; };

    let mut inspected = uiworld.write::<InspectedEntity>();

    if ui.button(format!("Owner: {owner:?}")).clicked() {
        inspected.e = Some(owner.0);
    }

    ui.label("Currently in the house:");
    for &soul in info.inside.iter() {
        if ui.button(format!("{soul:?}")).clicked() {
            inspected.e = Some(soul.0);
        }
    }
}

fn render_freightstation(ui: &mut Ui, _uiworld: &mut UiWorld, goria: &Egregoria, b: &Building) {
    let Some(owner) = goria.read::<BuildingInfos>().owner(b.id) else { return; };

    let Some(freight) = goria.comp::<FreightStation>(owner.0) else { return; };

    ui.label(format!("Waiting cargo: {}", freight.waiting_cargo));
    ui.label(format!("Wanted cargo: {}", freight.wanted_cargo));

    ui.add_space(10.0);
    ui.label("Trains:");
    for (tid, state) in &freight.trains {
        ui.horizontal(|ui| {
            ui.label(format!("{tid:?} "));
            match state {
                FreightTrainState::Arriving => {
                    ui.label("Arriving");
                }
                FreightTrainState::Loading => {
                    ui.label("Loading");
                }
                FreightTrainState::Moving => {
                    ui.label("Moving");
                }
            }
        });
    }
}

fn render_goodscompany(ui: &mut Ui, uiworld: &mut UiWorld, goria: &Egregoria, b: &Building) {
    let owner = goria.read::<BuildingInfos>().owner(b.id);

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
    let productivity = goods.productivity(workers.0.len(), b.zone.as_ref());
    let productivity = (productivity * 100.0).round();
    if productivity < 100.0 {
        egui::ProgressBar::new(productivity)
            .text(format!("productivity: {productivity:.0}%"))
            .desired_width(200.0)
            .ui(ui);
    }

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
