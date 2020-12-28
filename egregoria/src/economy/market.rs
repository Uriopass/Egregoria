use crate::SoulID;
use geom::Vec2;
use serde::export::PhantomData;
use std::collections::HashMap;

pub struct Market<T> {
    pub capital: HashMap<SoulID, i32>,
    pub buy_orders: HashMap<SoulID, (Vec2, i32)>,
    pub sell_orders: HashMap<SoulID, (Vec2, i32)>,
    _phantom: PhantomData<T>,
}

impl<T> Default for Market<T> {
    fn default() -> Self {
        Self {
            capital: Default::default(),
            buy_orders: Default::default(),
            sell_orders: Default::default(),
            _phantom: Default::default(),
        }
    }
}

pub struct Trade {
    pub buyer: SoulID,
    pub seller: SoulID,
    pub qty: i32,
}

impl<T> Market<T> {
    /// Called when a new agent arrives into this market, for example a new home is built or
    /// a new farm is made.
    /// Must be called before any order happens.
    pub fn add_agent(&mut self, soul: SoulID) {
        self.capital.insert(soul, 0);
    }

    /// Called when an agent tells the world it wants to sell something
    /// If an order is already placed, it will be updated.
    pub fn sell_order(&mut self, soul: SoulID, near: Vec2, qty: i32) {
        self.sell_orders.insert(soul, (near, qty));
    }

    /// Called when an agent tells the world it wants to buy something
    /// If an order is already placed, it will be updated.
    pub fn buy_order(&mut self, soul: SoulID, near: Vec2, qty: i32) {
        self.buy_orders.insert(soul, (near, qty));
    }

    /// Get the capital that this agent owns
    pub fn capital(&self, soul: SoulID) -> i32 {
        *self.capital.get(&soul).expect("forgot to add agent")
    }

    /// Called whenever an agent (like a farm) produces something on it's own
    /// for example wheat is harvested or turned into flour. Returns the new quantity owned.
    pub fn produce(&mut self, soul: SoulID, delta: i32) -> i32 {
        let v = self.capital.get_mut(&soul).expect("forgot to add agent");
        *v += delta;
        *v
    }

    /// Returns a list of buy and sell orders matched together.
    /// A trade updates the buy and sell orders from the market, and the capital of the buyers and sellers.
    /// A trade can only be completed if the seller has enough capital.
    pub fn make_trades(&mut self) -> Vec<Trade> {
        let trades = vec![];

        trades
    }
}
