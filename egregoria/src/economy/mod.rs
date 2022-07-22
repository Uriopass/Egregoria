use crate::SoulID;
use common::FastMap;
use hecs::World;
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::SubAssign;

mod government;
mod market;

#[derive(Serialize, Deserialize)]
/// Money in cents, can be negative when in debt.
pub struct Money(i64);

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.0 / 100).fmt(f)?;
        let cent = self.0 % 100;
        if cent > 0 {
            f.write_str(".")?;
            if cent < 10 {
                f.write_str("0")?;
            }
            cent.fmt(f)?;
        }
        f.write_str("¢")
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

pub use government::*;
pub use market::*;

#[derive(Default, Serialize, Deserialize)]
pub struct Sold(pub Vec<Trade>);

#[derive(Default, Serialize, Deserialize)]
pub struct Bought(pub FastMap<CommodityKind, Vec<Trade>>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Workers(pub Vec<SoulID>);

debug_inspect_impl!(Workers);

macro_rules! commodity {
    {$($member:tt => $display:literal),*,} => {
        #[derive(Copy, Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash, Serialize, Deserialize)]
        pub enum CommodityKind {
            $($member),*
        }
        impl CommodityKind {
            pub fn values() -> &'static [Self] {
                use CommodityKind::*;
                &[$($member),*]
            }
        }
        impl Display for CommodityKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$member => f.write_str($display)),*
                }
            }
        }
    };
}

debug_inspect_impl!(CommodityKind);

commodity! {
    JobOpening => "Job opening",
    Cereal => "Cereal",
    Flour => "Flour",
    Bread => "Bread",
    Vegetable => "Vegetables",
    Carcass => "Carcass",
    RawMeat => "Raw meat",
    Meat => "Meat",
    TreeLog => "Tree Log",
    WoodPlank => "Wood Planks",
    IronOre => "Iron Ore",
    Metal => "Metal",
    RareMetal => "Rare Metal",
    HighTechProduct => "High Tech Product",
    Furniture => "Furniture",
    Flower => "Flower",
    Wool => "Wool",
    Textile => "Textile",
    Cloth => "Cloth",
    Oil => "Oil",
    Coal => "Coal",
    Electricity => "Electricity",
    Polyester => "Polyester",
    Petrol => "Petrol",
}

#[profiling::function]
pub fn market_update(world: &mut World, resources: &mut Resources) {
    let mut m = resources.get_mut::<Market>().unwrap();
    for trade in m.make_trades() {
        log::debug!("A trade was made! {:?}", trade);

        match trade.kind {
            CommodityKind::JobOpening => world
                .get_mut::<Workers>(trade.seller.0)
                .expect("employer has no component Workers")
                .0
                .push(trade.buyer),
            _ => {
                if let Ok(mut v) = world.get_mut::<Sold>(trade.seller.0) {
                    v.0.push(trade)
                }
            }
        }

        if let Ok(mut v) = world.get_mut::<Bought>(trade.buyer.0) {
            v.0.entry(trade.kind).or_default().push(trade);
        }
    }
}
