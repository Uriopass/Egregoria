use legion::world::SubWorld;
use legion::{system, EntityStore};

mod market;

use crate::SoulID;
pub use market::*;

pub trait Commodity {}
impl<T> Commodity for T {}

pub trait CommodityList {}

#[derive(Default)]
pub struct Sold(pub Vec<Trade>);

#[derive(Default)]
pub struct Workers(pub Vec<SoulID>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CommodityKind {
    JobOpening,
    Wheat,
    Flour,
    Bread,
}

impl CommodityKind {
    pub fn values() -> &'static [Self] {
        use CommodityKind::*;
        &[JobOpening, Wheat, Flour, Bread]
    }
}

#[system]
#[write_component(Sold)]
#[write_component(Workers)]
pub fn market_update(#[resource] m: &mut Market, subworld: &mut SubWorld) {
    for trade in m.make_trades() {
        log::info!("A trade was made! {:?}", trade);

        let mut ent = subworld.entry_mut(trade.seller.0).unwrap();

        match trade.kind {
            CommodityKind::JobOpening => ent
                .get_component_mut::<Workers>()
                .expect("seller has no component Workers")
                .0
                .push(trade.buyer),
            _ => ent
                .get_component_mut::<Sold>()
                .expect("seller has no component Sold")
                .0
                .push(trade),
        }
    }
}
