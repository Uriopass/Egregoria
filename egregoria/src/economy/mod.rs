use crate::SoulID;
use derive_more::{Add, AddAssign, Sub, SubAssign};

mod market;

pub use market::*;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct Money(pub i32);

#[derive(Copy, Clone, Debug, Default, Add, Sub, AddAssign, SubAssign)]
pub struct Goods {
    pub food: i32,
}

impl Goods {
    pub fn is_smaller(&self, delta: &Goods) -> bool {
        self.food <= delta.food
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transaction {
    pub cost: Money,
    pub delta: Goods,
}

pub struct EconomicAgent {
    pub id: SoulID,
    pub money: Money,
    pub goods: Goods,
}

impl EconomicAgent {
    pub fn new(id: SoulID, money: Money, goods: Goods) -> Self {
        EconomicAgent { id, money, goods }
    }
}
