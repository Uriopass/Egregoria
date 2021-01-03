use legion::world::SubWorld;
use legion::{system, EntityStore};

mod market;

pub use market::*;

pub trait Commodity {}
impl<T> Commodity for T {}

pub trait CommodityList {}

pub struct Sold<T: Commodity>(pub Vec<Trade<T>>);

impl<T> Default for Sold<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

pub type Workers = Sold<JobApplication>;

pub struct Wheat;
pub struct Flour;
pub struct Bread;

pub struct JobApplication;

macro_rules! markets_system {
    ($($n:ident; $t: ty),+) => {
        #[system]
        $(
        #[write_component(Sold<$t>)]
        )+
        pub fn markets_update($(#[resource] $n: &mut Market<$t>,)+ subworld: &mut SubWorld) {
            $(
            for trade in $n.make_trades() {
                subworld
                    .entry_mut(trade.seller.0)
                    .unwrap()
                    .get_component_mut::<Sold<$t>>()
                    .expect("seller has no component Sold")
                    .0
                    .push(trade)
            }
            )+
        }
    }
}

markets_system!(a;JobApplication, b;Wheat);
