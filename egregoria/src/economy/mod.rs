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
use crate::World;
use crate::{GoodsCompanyRegistry, SoulID};
use egui_inspect::Inspect;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, SubAssign};

mod ecostats;
mod government;
mod item;
mod market;

use crate::utils::time::{Tick, TICKS_PER_SECOND};
use crate::world::HumanID;
pub use ecostats::*;
pub use government::*;
pub use item::*;
pub use market::*;

const WORKER_CONSUMPTION_PER_SECOND: Money = Money::new_cents(1);

/// Money in cents, can be negative when expressing debt.
#[derive(Default, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Money(i64);

debug_inspect_impl!(Money);

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&(self.bucks()), f)?;
        let cent = (self.0 % 10000) / 100;
        if cent > 0 {
            f.write_str(".")?;
            if cent < 10 {
                f.write_str("0")?;
            }
            Display::fmt(&cent, f)?;
        }
        f.write_str("$")
    }
}

impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        Money(-self.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}

impl Debug for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::ops::Sub for Money {
    type Output = Money;

    fn sub(self, other: Money) -> Money {
        Money(self.0 - other.0)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Money) {
        self.0 -= other.0;
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        Money(self.0 + other.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Money) {
        self.0 += other.0;
    }
}

impl Mul<i64> for Money {
    type Output = Money;

    fn mul(self, rhs: i64) -> Self::Output {
        Money(self.0 * rhs)
    }
}

impl Mul<Money> for i64 {
    type Output = Money;

    fn mul(self, rhs: Money) -> Self::Output {
        Money(self * rhs.0)
    }
}

impl Div<i64> for Money {
    type Output = Money;

    fn div(self, rhs: i64) -> Self::Output {
        Money(self.0 / rhs)
    }
}

impl Money {
    pub const ZERO: Money = Money(0);
    pub const MAX: Money = Money(i64::MAX);

    pub const fn new_inner(inner: i64) -> Self {
        Self(inner)
    }

    pub const fn new_cents(cents: i64) -> Self {
        Self(cents * 100)
    }

    pub const fn new_bucks(base: i64) -> Self {
        Self(base * 10000)
    }

    pub fn inner(&self) -> i64 {
        self.0
    }

    pub fn cents(&self) -> i64 {
        self.0 / 100
    }

    pub fn bucks(&self) -> i64 {
        self.0 / 10000
    }
}

#[derive(Inspect, Default, Serialize, Deserialize)]
pub struct Sold(pub Vec<Trade>);

#[derive(Inspect, Default, Serialize, Deserialize)]
pub struct Bought(pub BTreeMap<ItemID, Vec<Trade>>);

#[derive(Inspect, Debug, Default, Serialize, Deserialize)]
pub struct Workers(pub Vec<HumanID>);

#[cfg(not(test))]
const ITEMS_PATH: &str = "assets/items.json";
#[cfg(not(test))]
const COMPANIES_PATH: &str = "assets/companies.json";

#[cfg(test)]
const ITEMS_PATH: &str = "../assets/items.json";
#[cfg(test)]
const COMPANIES_PATH: &str = "../assets/companies.json";

pub fn init_market(_: &mut World, res: &mut Resources) {
    res.get_mut::<ItemRegistry>()
        .unwrap()
        .load_item_definitions(&common::saveload::load_string(ITEMS_PATH).unwrap());

    res.get_mut::<GoodsCompanyRegistry>().unwrap().load(
        &common::saveload::load_string(COMPANIES_PATH).unwrap(),
        &res.get::<ItemRegistry>().unwrap(),
    );

    let market = Market::new(
        &res.get::<ItemRegistry>().unwrap(),
        &res.get::<GoodsCompanyRegistry>().unwrap(),
    );
    res.insert(market);
    let stats = EcoStats::new(&res.get::<ItemRegistry>().unwrap());
    res.insert(stats);
}

#[profiling::function]
pub fn market_update(world: &mut World, resources: &mut Resources) {
    let n_workers = world.humans.len();

    let mut m = resources.get_mut::<Market>().unwrap();
    let job_opening = resources.get::<ItemRegistry>().unwrap().id("job-opening");
    let mut gvt = resources.get_mut::<Government>().unwrap();
    let tick = resources.get::<Tick>().unwrap().0;

    if tick % TICKS_PER_SECOND == 0 {
        gvt.money -= n_workers as i64 * WORKER_CONSUMPTION_PER_SECOND;
    }

    let trades = m.make_trades();

    resources
        .get_mut::<EcoStats>()
        .unwrap()
        .advance(tick, trades);

    for &trade in trades.iter() {
        log::debug!("A trade was made! {:?}", trade);

        if trade.kind == job_opening {
            if let SoulID::GoodsCompany(id) = trade.seller.soul() {
                let comp = world.companies.get_mut(id).unwrap();
                comp.workers.0.push(trade.buyer.soul().try_into().unwrap())
            }
        }
        gvt.money += trade.money_delta;

        match trade.seller {
            TradeTarget::Soul(id) => {
                if trade.kind != job_opening {
                    if let SoulID::GoodsCompany(id) = id {
                        world.companies.get_mut(id).unwrap().sold.0.push(trade);
                    }
                }
            }
            TradeTarget::ExternalTrade => {}
        }

        match trade.buyer {
            TradeTarget::Soul(SoulID::Human(id)) => {
                if let Some(h) = world.humans.get_mut(id) {
                    h.bought.0.entry(trade.kind).or_default().push(trade);
                }
            }
            TradeTarget::Soul(SoulID::GoodsCompany(id)) => {
                if let Some(c) = world.companies.get_mut(id) {
                    c.bought.0.entry(trade.kind).or_default().push(trade)
                }
            }
            TradeTarget::Soul(SoulID::FreightStation(_)) => {}
            TradeTarget::ExternalTrade => {}
        }
    }
}
