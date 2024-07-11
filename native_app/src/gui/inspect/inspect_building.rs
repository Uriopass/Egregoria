use goryak::{
    dragvalue, fixed_spacer, minrow, on_secondary_container, primary, textc, ProgressBar, Window,
};
use prototypes::{ItemID, Recipe};
use simulation::economy::Market;
use simulation::map::{Building, BuildingID, BuildingKind, Zone, MAX_ZONE_AREA};
use simulation::map_dynamic::{BuildingInfos, ElectricityFlow};
use simulation::souls::freight_station::FreightTrainState;
use simulation::world_command::WorldCommand;
use simulation::{Simulation, SoulID};
use std::borrow::Cow;
use yakui::widgets::Pad;
use yakui::Vec2;

use crate::gui::inspect::entity_link;
use crate::gui::item_icon_yakui;
use crate::uiworld::UiWorld;

fn label(x: impl Into<Cow<'static, str>>) {
    textc(on_secondary_container(), x);
}

/// Inspect a specific building, showing useful information about it
pub fn inspect_building(uiworld: &UiWorld, sim: &Simulation, id: BuildingID) -> bool {
    let map = sim.map();
    let Some(building) = map.buildings().get(id) else {
        return false;
    };

    let title: &str = match building.kind {
        BuildingKind::House => "House",
        BuildingKind::GoodsCompany(id) => &id.prototype().name,
        BuildingKind::RailFreightStation(id) => &id.prototype().name,
        BuildingKind::TrainStation => "Train Station",
        BuildingKind::ExternalTrading => "External Trading",
    };

    let mut is_open = true;
    Window {
        title: title.into(),
        pad: Pad::all(10.0),
        radius: 10.0,
        opened: &mut is_open,
        child_spacing: 5.0,
    }
    .show(|| {
        if cfg!(debug_assertions) {
            label(format!("{:?}", building.id));
        }

        match building.kind {
            BuildingKind::House => render_house(uiworld, sim, building),
            BuildingKind::GoodsCompany(_) => {
                render_goodscompany(uiworld, sim, building);
            }
            BuildingKind::RailFreightStation(_) => {
                render_freightstation(uiworld, sim, building);
            }
            BuildingKind::TrainStation => {}
            BuildingKind::ExternalTrading => {}
        };

        if let Some(ref zone) = building.zone {
            let mut cpy = zone.filldir;
            minrow(5.0, || {
                let mut ang = cpy.angle_cossin().to_degrees();

                if dragvalue().min(-180.0).max(180.0).show(&mut ang.0) {
                    cpy = ang.to_radians().vec2();
                    uiworld.commands().push(WorldCommand::UpdateZone {
                        building: id,
                        zone: Zone {
                            filldir: cpy,
                            ..zone.clone()
                        },
                    })
                }

                label("Fill angle");
            });

            ProgressBar {
                value: zone.area / MAX_ZONE_AREA,
                size: Vec2::new(200.0, 25.0),
                color: primary().adjust(0.7),
            }
            .show_children(|| {
                label(format!("area: {}/{}", zone.area, MAX_ZONE_AREA));
            });
        }
    });

    is_open
}

fn render_house(uiworld: &UiWorld, sim: &Simulation, b: &Building) {
    let binfos = sim.read::<BuildingInfos>();
    let Some(info) = binfos.get(b.id) else {
        return;
    };
    let Some(SoulID::Human(owner)) = info.owner else {
        return;
    };

    minrow(5.0, || {
        label("Owner");
        entity_link(uiworld, sim, owner);
    });

    label("Currently in the house:");
    for &soul in info.inside.iter() {
        let SoulID::Human(soul) = soul else {
            continue;
        };
        entity_link(uiworld, sim, soul);
    }
}

fn render_freightstation(uiworld: &UiWorld, sim: &Simulation, b: &Building) {
    let Some(SoulID::FreightStation(owner)) = sim.read::<BuildingInfos>().owner(b.id) else {
        return;
    };
    let Some(freight) = sim.world().get(owner) else {
        return;
    };

    label(format!("Waiting cargo: {}", freight.f.waiting_cargo));
    label(format!("Wanted cargo: {}", freight.f.wanted_cargo));

    fixed_spacer((0.0, 10.0));
    label("Trains:");
    for (tid, state) in &freight.f.trains {
        minrow(5.0, || {
            entity_link(uiworld, sim, *tid);
            match state {
                FreightTrainState::Arriving => {
                    label("Arriving");
                }
                FreightTrainState::Loading => {
                    label("Loading");
                }
                FreightTrainState::Moving => {
                    label("Moving");
                }
            }
        });
    }
}

fn render_goodscompany(uiworld: &UiWorld, sim: &Simulation, b: &Building) {
    let owner = sim.read::<BuildingInfos>().owner(b.id);

    let Some(SoulID::GoodsCompany(c_id)) = owner else {
        return;
    };
    let Some(c) = sim.world().companies.get(c_id) else {
        return;
    };
    let goods = &c.comp;
    let workers = &c.workers;
    let proto = c.comp.proto.prototype();

    let market = &*sim.read::<Market>();
    let map = &*sim.map();
    let elec_flow = &*sim.read::<ElectricityFlow>();

    let max_workers = goods.max_workers;
    ProgressBar {
        value: workers.0.len() as f32 / max_workers as f32,
        size: Vec2::new(200.0, 25.0),
        color: primary().adjust(0.7),
    }
    .show_children(|| {
        label(format!("workers: {}/{}", workers.0.len(), max_workers));
    });

    if let Some(driver) = goods.driver {
        minrow(5.0, || {
            label("Driver is");
            entity_link(uiworld, sim, driver);
        });
    }
    let productivity = c.productivity(proto, b.zone.as_ref(), map, elec_flow);
    if productivity < 1.0 {
        ProgressBar {
            value: productivity,
            size: Vec2::new(200.0, 25.0),
            color: primary().adjust(0.7),
        }
        .show_children(|| {
            label(format!(
                "productivity: {:.0}%",
                (productivity * 100.0).round()
            ));
        });
    }

    if let Some(ref r) = proto.recipe {
        render_recipe(uiworld, r);
    }

    if let Some(net_id) = map.electricity.net_id(b.id) {
        let blackout = elec_flow.blackout(net_id);

        if let Some(power_c) = proto.power_consumption {
            ProgressBar {
                value: productivity,
                size: Vec2::new(200.0, 25.0),
                color: primary().adjust(0.7),
            }
            .show_children(|| {
                label(format!(
                    "power: {}/{}",
                    productivity as f64 * power_c,
                    power_c
                ));
            });
        }

        if let Some(power_prod) = proto.power_production {
            label(format!(
                "producing power: {}",
                power_prod * productivity as f64
            ));

            let stats = elec_flow.network_stats(net_id);

            ProgressBar {
                value: if blackout { 0.0 } else { 1.0 },
                size: Vec2::new(200.0, 25.0),
                color: primary().adjust(0.7),
            }
            .show_children(|| {
                label(format!(
                    "Network health: {}/{}={:.0}%",
                    stats.produced_power,
                    stats.consumed_power,
                    (100 * stats.produced_power.0) / stats.consumed_power.0.max(1)
                ));
            });
        }
    }

    ProgressBar {
        value: goods.progress,
        size: Vec2::new(200.0, 25.0),
        color: primary().adjust(0.7),
    }
    .show_children(|| {
        label(format!("{:.0}%", goods.progress * 100.0));
    });

    fixed_spacer((0.0, 10.0));
    label("Storage");

    let jobopening = ItemID::new("job-opening");
    for (&id, m) in market.iter() {
        let Some(v) = m.capital(c_id.into()) else {
            continue;
        };
        if id == jobopening && v == 0 {
            continue;
        }

        item_icon_yakui(uiworld, id, v);
    }
}

fn render_recipe(uiworld: &UiWorld, recipe: &Recipe) {
    if recipe.consumption.is_empty() {
        label("No Inputs");
    } else {
        label(if recipe.consumption.len() == 1 {
            "Input"
        } else {
            "Inputs"
        });
        minrow(5.0, || {
            for item in recipe.consumption.iter() {
                item_icon_yakui(uiworld, item.id, item.amount);
            }
        });
    }

    if recipe.production.is_empty() {
        label("No Outputs");
    } else {
        label(if recipe.production.len() == 1 {
            "Output"
        } else {
            "Outputs"
        });
        minrow(5.0, || {
            for item in recipe.production.iter() {
                item_icon_yakui(uiworld, item.id, item.amount);
            }
        });
    }
}
