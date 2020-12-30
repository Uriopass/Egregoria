use crate::economy::Commodity;
use crate::SoulID;
use geom::Vec2;
use ordered_float::OrderedFloat;
use serde::export::PhantomData;
use std::collections::{HashMap, HashSet};

pub struct Market<T: Commodity> {
    pub capital: HashMap<SoulID, i32>,
    pub buy_orders: HashMap<SoulID, (Vec2, i32)>,
    pub sell_orders: HashMap<SoulID, (Vec2, i32)>,
    potential: Vec<(f32, Trade, bool)>,
    _phantom: PhantomData<T>,
}

impl<T: Commodity> Default for Market<T> {
    fn default() -> Self {
        Self {
            capital: Default::default(),
            buy_orders: Default::default(),
            sell_orders: Default::default(),
            potential: Default::default(),
            _phantom: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Trade {
    pub buyer: SoulID,
    pub seller: SoulID,
    pub qty: i32,
}

impl<T: Commodity> Market<T> {
    /// Called when a new agent arrives into this market, for example a new home is built or
    /// a new farm is made.
    /// Must be called before any order happens.
    pub fn add_agent(&mut self, soul: SoulID) {
        self.capital.insert(soul, 0);
    }

    /// Called when an agent tells the world it wants to sell something
    /// If an order is already placed, it will be updated.
    pub fn sell(&mut self, soul: SoulID, near: Vec2, qty: i32) {
        self.sell_orders.insert(soul, (near, qty));
    }

    /// Called when an agent tells the world it wants to buy something
    /// If an order is already placed, it will be updated.
    pub fn buy(&mut self, soul: SoulID, near: Vec2, qty: i32) {
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
    pub fn make_trades(&mut self) -> impl Iterator<Item = Trade> + '_ {
        // Naive O(n²) alg
        self.potential.clear();
        for (&seller, &(pos_sell, qty_sell)) in &self.sell_orders {
            let capital_sell = self.capital[&seller];
            if qty_sell > capital_sell {
                continue;
            }
            for (&buyer, &(pos_buy, qty_buy)) in &self.buy_orders {
                if seller == buyer {
                    log::warn!(
                        "{:?} is both selling and buying same commodity: {:?}",
                        seller,
                        std::any::type_name::<T>()
                    );
                    continue;
                }
                if qty_buy <= qty_sell {
                    let score = pos_sell.distance2(pos_buy);
                    self.potential.push((
                        score,
                        Trade {
                            buyer,
                            seller,
                            qty: qty_buy,
                        },
                        qty_buy == qty_sell,
                    ))
                }
            }
        }
        self.potential
            .sort_unstable_by_key(|(x, _, _)| OrderedFloat(*x));
        let mut already_sold = HashSet::new();
        let Self {
            buy_orders,
            sell_orders,
            capital,
            ..
        } = self;

        self.potential
            .iter()
            .filter(move |(_, trade, complete)| {
                let ok = already_sold.insert(trade.buyer) && already_sold.insert(trade.seller);
                if !ok {
                    return false;
                }
                buy_orders.remove(&trade.buyer);
                if *complete {
                    sell_orders.remove(&trade.seller);
                } else if let Some((_, qty)) = sell_orders.get_mut(&trade.seller) {
                    *qty -= trade.qty
                }

                *capital.get_mut(&trade.buyer).expect("add_agent not called") += trade.qty;
                *capital
                    .get_mut(&trade.seller)
                    .expect("add_agent not called") -= trade.qty;

                true
            })
            .map(|&(_, x, _)| x)
    }
}

#[cfg(test)]
mod tests {
    use super::Market;
    use crate::SoulID;
    use geom::{vec2, Vec2};
    use legion::Entity;

    fn mk_ent(id: u64) -> Entity {
        unsafe { std::mem::transmute(id) }
    }

    #[test]
    fn test_match_orders() {
        let seller = SoulID(mk_ent(1));
        let seller_far = SoulID(mk_ent(2));
        let buyer = SoulID(mk_ent(3));

        let mut m = Market::<()>::default();

        m.add_agent(seller);
        m.add_agent(seller_far);
        m.add_agent(buyer);

        m.produce(seller, 3);
        m.produce(seller_far, 3);

        m.buy(buyer, Vec2::ZERO, 2);
        m.sell(seller, Vec2::UNIT_X, 3);
        m.sell(seller_far, vec2(10.0, 10.0), 3);

        let trades = m.make_trades().collect::<Vec<_>>();

        assert_eq!(trades.len(), 1);
        let t0 = trades[0];
        assert_eq!(t0.seller, seller);
        assert_eq!(t0.buyer, buyer);
        assert_eq!(t0.qty, 2);
    }
}
