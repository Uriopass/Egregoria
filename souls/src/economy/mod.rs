use derive_more::{Add, AddAssign, Sub, SubAssign};
use egregoria::SoulID;

mod market;

pub use market::*;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct Money(pub i32);

#[derive(Copy, Clone, Default, Add, Sub, AddAssign, SubAssign)]
pub struct Goods {
    food: i32,
}

impl Goods {
    pub fn is_smaller(&self, delta: &Goods) -> bool {
        self.food < delta.food
    }
}

#[derive(Clone)]
pub struct Transaction {
    cost: Money,
    delta: Goods,
}

pub struct EconomicAgent {
    id: SoulID,
    money: Money,
    goods: Goods,
}
