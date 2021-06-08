use crate::SoulID;
use common::FastMap;
use legion::world::SubWorld;
use legion::{system, EntityStore};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

mod government;
mod market;

#[derive(Serialize, Deserialize)]
/// Money in cents, can be negative when in debt.
pub struct Money(pub i64);

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

impl Money {
    pub fn cents(cents: i64) -> Self {
        Self(cents)
    }

    pub fn base(base: i64) -> Self {
        Self(base * 100)
    }
}

pub use government::*;
use imgui_inspect::imgui::__core::fmt::Formatter;
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

register_system!(market_update);
#[system]
#[write_component(Sold)]
#[write_component(Bought)]
#[write_component(Workers)]
pub fn market_update(#[resource] m: &mut Market, subworld: &mut SubWorld) {
    for trade in m.make_trades() {
        log::info!("A trade was made! {:?}", trade);

        let mut ent = unwrap_orr!(subworld.entry_mut(trade.seller.0), continue);

        match trade.kind {
            CommodityKind::JobOpening => ent
                .get_component_mut::<Workers>()
                .expect("employer has no component Workers")
                .0
                .push(trade.buyer),
            _ => {
                if let Ok(v) = ent.get_component_mut::<Sold>() {
                    v.0.push(trade)
                }
            }
        }

        if let Ok(v) =
            unwrap_orr!(subworld.entry_mut(trade.buyer.0), continue).get_component_mut::<Bought>()
        {
            v.0.entry(trade.kind).or_default().push(trade);
        }
    }
}
