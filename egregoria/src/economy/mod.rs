use crate::SoulID;
use common::FastMap;
use hecs::World;
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::SubAssign;

mod government;
mod item;
mod market;

pub use government::*;
pub use item::*;
pub use market::*;

/// Money in cents, can be negative when expressing debt.
#[derive(Copy, Clone, Serialize, Deserialize)]
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

#[profiling::function]
pub fn market_update(world: &mut World, resources: &mut Resources) {
    let mut m = resources.get_mut::<Market>().unwrap();
    let job_opening = resources.get::<ItemRegistry>().unwrap().id("job-opening");
    for trade in m.make_trades() {
        log::debug!("A trade was made! {:?}", trade);

        if trade.kind == job_opening {
            world
                .get_mut::<Workers>(trade.seller.0)
                .expect("employer has no component Workers")
                .0
                .push(trade.buyer);
        } else if let Ok(mut v) = world.get_mut::<Sold>(trade.seller.0) {
            v.0.push(trade)
        }

        if let Ok(mut v) = world.get_mut::<Bought>(trade.buyer.0) {
            v.0.entry(trade.kind).or_default().push(trade);
        }
    }
}
