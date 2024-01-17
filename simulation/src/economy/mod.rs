//! Economy module
//!
//! This module contains all the code related to the economy of the game.
//!
//! The economy is divided in 2 parts:
//!
//! - The market, which is the place where goods are exchanged.
//! - The government, which is the entity representing the player
//!
use crate::utils::resources::Resources;
use crate::SoulID;
use crate::World;
use egui_inspect::Inspect;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;

mod ecostats;
mod government;
mod market;

use crate::map::Map;
use crate::world::HumanID;
pub use ecostats::*;
pub use government::*;
pub use market::*;
use prototypes::{GameTime, ItemID, Money, TICKS_PER_MINUTE};

const WORKER_CONSUMPTION_PER_MINUTE: Money = Money::new_cents(10);

#[derive(Inspect, Default, Serialize, Deserialize)]
pub struct Sold(pub Vec<Trade>);

#[derive(Inspect, Default, Serialize, Deserialize)]
pub struct Bought(pub BTreeMap<ItemID, Vec<Trade>>);

#[derive(Inspect, Debug, Default, Serialize, Deserialize)]
pub struct Workers(pub Vec<HumanID>);

pub fn market_update(world: &mut World, resources: &mut Resources) {
    profiling::scope!("economy::market_update");
    let n_workers = world.humans.len();

    let mut m = resources.write::<Market>();
    let job_opening = ItemID::new("job-opening");
    let mut gvt = resources.write::<Government>();
    let tick = resources.read::<GameTime>().tick;

    if tick.0 % TICKS_PER_MINUTE == 0 {
        gvt.money -= n_workers as i64 * WORKER_CONSUMPTION_PER_MINUTE;
    }

    let freights = &world.freight_stations;

    let map = resources.read::<Map>();
    let trades = m.make_trades(|pos| {
        freights
            .iter()
            .min_by_key(|(_, b)| {
                let Some(b) = map.buildings.get(b.f.building) else {
                    return OrderedFloat(f32::INFINITY);
                };
                OrderedFloat(b.door_pos.xy().distance2(pos))
            })
            .map(|(id, _)| SoulID::FreightStation(id))
    });

    resources.write::<EcoStats>().advance(tick.0, trades);

    for &trade in trades.iter() {
        log::debug!("A trade was made! {:?}", trade);

        if trade.kind == job_opening {
            if let SoulID::GoodsCompany(id) = trade.seller.0 {
                let comp = world.companies.get_mut(id).unwrap();
                comp.workers.0.push(trade.buyer.0.try_into().unwrap())
            }
        }
        gvt.money += trade.money_delta;

        if let SoulID::GoodsCompany(id) = trade.seller.0 {
            if trade.kind != job_opening {
                world.companies.get_mut(id).unwrap().sold.0.push(trade);
            }
        }

        match trade.buyer.0 {
            SoulID::Human(id) => {
                if let Some(h) = world.humans.get_mut(id) {
                    h.bought.0.entry(trade.kind).or_default().push(trade);
                }
            }
            SoulID::GoodsCompany(id) => {
                if let Some(c) = world.companies.get_mut(id) {
                    c.bought.0.entry(trade.kind).or_default().push(trade)
                }
            }
            SoulID::FreightStation(_) => {}
        }
    }
}
