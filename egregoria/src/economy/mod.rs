use legion::world::SubWorld;
use legion::{system, EntityStore};

mod market;

use crate::SoulID;
pub use market::*;
use std::collections::HashMap;

pub trait Commodity {}
impl<T> Commodity for T {}

pub trait CommodityList {}

#[derive(Default)]
pub struct Sold(pub Vec<Trade>);

#[derive(Default)]
pub struct Bought(pub HashMap<CommodityKind, Vec<Trade>>);

#[derive(Default)]
pub struct Workers(pub Vec<SoulID>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CommodityKind {
    JobOpening,
    Cereal,
    Flour,
    Bread,
}

impl CommodityKind {
    pub fn values() -> &'static [Self] {
        use CommodityKind::*;
        &[JobOpening, Cereal, Flour, Bread]
    }
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
