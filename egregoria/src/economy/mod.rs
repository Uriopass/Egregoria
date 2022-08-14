use crate::{GoodsCompanyRegistry, SoulID};
use common::FastMap;
use hecs::World;
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{AddAssign, Mul, SubAssign};

mod government;
mod item;
mod market;

pub use government::*;
pub use item::*;
pub use market::*;

/// Money in cents, can be negative when expressing debt.
#[derive(Default, Copy, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(i64);

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&(self.0 / 100), f)?;
        let cent = self.0 % 100;
        if cent > 0 {
            f.write_str(".")?;
            if cent < 10 {
                f.write_str("0")?;
            }
            Display::fmt(&cent, f)?;
        }
        f.write_str("¢")
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

impl Money {
    pub const ZERO: Money = Money(0);

    pub fn new_cents(cents: i64) -> Self {
        Self(cents)
    }

    pub fn new_base(base: i64) -> Self {
        Self(base * 100)
    }

    pub fn cents(&self) -> i64 {
        self.0
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Sold(pub Vec<Trade>);

#[derive(Default, Serialize, Deserialize)]
pub struct Bought(pub FastMap<ItemID, Vec<Trade>>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Workers(pub Vec<SoulID>);

debug_inspect_impl!(Workers);

pub fn init_market(_: &mut World, res: &mut Resources) {
    res.get_mut::<ItemRegistry>()
        .unwrap()
        .load_item_definitions(&*std::fs::read_to_string("assets/items.json").unwrap());

    res.get_mut::<GoodsCompanyRegistry>().unwrap().load(
        &*std::fs::read_to_string("assets/companies.json").unwrap(),
        &*res.get::<ItemRegistry>().unwrap(),
    );

    let market = Market::new(&*res.get::<ItemRegistry>().unwrap());
    res.insert(market);
}

#[profiling::function]
pub fn market_update(world: &mut World, resources: &mut Resources) {
    let mut m = resources.get_mut::<Market>().unwrap();
    let job_opening = resources.get::<ItemRegistry>().unwrap().id("job-opening");
    let mut gvt = resources.get_mut::<Government>().unwrap();

    for trade in m.make_trades() {
        log::debug!("A trade was made! {:?}", trade);

        if trade.kind == job_opening {
            // Jobs are guaranteed to not be external
            world
                .get::<&mut Workers>(trade.seller.soul().0)
                .expect("employer has no component Workers")
                .0
                .push(trade.buyer.soul());
        }

        match trade.buyer {
            TradeTarget::Soul(id) => {
                if trade.kind != job_opening {
                    if let Ok(mut v) = world.get::<&mut Sold>(id.0) {
                        v.0.push(trade)
                    }
                }
            }
            TradeTarget::ExternalTrade => {
                let singlem = m.m(trade.kind);
                gvt.money += (singlem.ext_value - singlem.transport_cost) * trade.qty as i64;
            }
        }

        match trade.seller {
            TradeTarget::Soul(id) => {
                if let Ok(mut v) = world.get::<&mut Bought>(id.0) {
                    v.0.entry(trade.kind).or_default().push(trade);
                }
            }
            TradeTarget::ExternalTrade => {}
        }
    }
}
