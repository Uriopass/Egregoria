use crate::economy::CommodityKind;
use crate::SoulID;
use geom::Vec2;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Serialize, Deserialize)]
pub struct SingleMarket {
    // todo: change i32 to Quantity
    capital: BTreeMap<SoulID, i32>,
    buy_orders: BTreeMap<SoulID, (Vec2, i32)>,
    sell_orders: BTreeMap<SoulID, (Vec2, i32)>,
    ext_buy: i32,
    ext_sell: i32,
}

impl SingleMarket {
    pub fn capital(&self, soul: SoulID) -> Option<i32> {
        self.capital.get(&soul).copied()
    }

    pub fn capital_map(&self) -> &BTreeMap<SoulID, i32> {
        &self.capital
    }
    pub fn buy_orders(&self) -> &BTreeMap<SoulID, (Vec2, i32)> {
        &self.buy_orders
    }
    pub fn sell_orders(&self) -> &BTreeMap<SoulID, (Vec2, i32)> {
        &self.sell_orders
    }
}

#[derive(Serialize, Deserialize)]
pub struct Market {
    markets: BTreeMap<CommodityKind, SingleMarket>,
}

impl Default for Market {
    fn default() -> Self {
        Self {
            markets: CommodityKind::values()
                .iter()
                .map(|&v| (v, SingleMarket::default()))
                .collect(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    pub buyer: SoulID,
    pub seller: SoulID,
    pub qty: i32,
    pub sell_pos: Vec2,
    pub buy_pos: Vec2,
    pub kind: CommodityKind,
}

impl Market {
    fn m(&mut self, kind: CommodityKind) -> &mut SingleMarket {
        self.markets.get_mut(&kind).unwrap()
    }

    /// Called when an agent tells the world it wants to sell something
    /// If an order is already placed, it will be updated.
    /// Beware that you need capital to sell anything, using produce.
    pub fn sell(&mut self, soul: SoulID, near: Vec2, kind: CommodityKind, qty: i32) {
        log::debug!("{:?} sell {:?} {:?} near {:?}", soul, qty, kind, near);
        self.m(kind).sell_orders.insert(soul, (near, qty));
    }

    pub fn sell_all(&mut self, soul: SoulID, near: Vec2, kind: CommodityKind) {
        let c = self.capital(soul, kind);
        if c == 0 {
            return;
        }
        self.sell(soul, near, kind, c);
    }

    /// Called when an agent tells the world it wants to buy something
    /// If an order is already placed, it will be updated.
    pub fn buy(&mut self, soul: SoulID, near: Vec2, kind: CommodityKind, qty: i32) {
        log::debug!("{:?} buy {:?} {:?} near {:?}", soul, qty, kind, near);

        self.m(kind).buy_orders.insert(soul, (near, qty));
    }

    pub fn buy_until(&mut self, soul: SoulID, near: Vec2, kind: CommodityKind, qty: i32) {
        let c = self.capital(soul, kind);
        if c >= qty {
            return;
        }
        self.buy(soul, near, kind, qty - c);
    }

    /// Get the capital that this agent owns
    pub fn capital(&self, soul: SoulID, kind: CommodityKind) -> i32 {
        self.markets.get(&kind).unwrap().capital(soul).unwrap_or(0)
    }

    /// Registers a soul to the market, not obligatory
    pub fn register(&mut self, soul: SoulID, kind: CommodityKind) {
        self.m(kind).capital.entry(soul).or_default();
    }

    /// Called whenever an agent (like a farm) produces something on it's own
    /// for example wheat is harvested or turned into flour. Returns the new quantity owned.
    pub fn produce(&mut self, soul: SoulID, kind: CommodityKind, delta: i32) -> i32 {
        log::debug!("{:?} produced {:?} {:?}", soul, delta, kind);

        let v = self.m(kind).capital.entry(soul).or_default();
        *v += delta;
        *v
    }

    /// Returns a list of buy and sell orders matched together.
    /// A trade updates the buy and sell orders from the market, and the capital of the buyers and sellers.
    /// A trade can only be completed if the seller has enough capital.
    pub fn make_trades(&mut self) -> impl Iterator<Item = Trade> + '_ {
        let mut all_trades = vec![];
        let mut potential = vec![];

        for (&kind, market) in &mut self.markets {
            // Naive O(n²) alg
            for (&seller, &(sell_pos, qty_sell)) in &market.sell_orders {
                let capital_sell = unwrap_or!(market.capital(seller), continue);
                if qty_sell > capital_sell {
                    continue;
                }
                for (&buyer, &(buy_pos, qty_buy)) in &market.buy_orders {
                    if seller == buyer {
                        log::warn!(
                            "{:?} is both selling and buying same commodity: {:?}",
                            seller,
                            kind
                        );
                        continue;
                    }
                    if qty_buy <= qty_sell {
                        let score = sell_pos.distance2(buy_pos);
                        potential.push((
                            score,
                            Trade {
                                buyer,
                                seller,
                                qty: qty_buy,
                                sell_pos,
                                buy_pos,
                                kind,
                            },
                            qty_buy == qty_sell,
                        ))
                    }
                }
            }
            potential.sort_unstable_by_key(|(x, _, _)| OrderedFloat(*x));
            let mut already_sold = BTreeSet::default();
            let SingleMarket {
                buy_orders,
                sell_orders,
                capital, 
                ..
            } = market;

            all_trades.extend(
                potential
                    .drain(..)
                    .filter(move |(_, trade, complete)| {
                        let ok =
                            already_sold.insert(trade.buyer) && already_sold.insert(trade.seller);
                        if !ok {
                            return false;
                        }
                        buy_orders.remove(&trade.buyer);
                        if *complete {
                            sell_orders.remove(&trade.seller);
                        } else if let Some((_, qty)) = sell_orders.get_mut(&trade.seller) {
                            *qty -= trade.qty
                        }

                        *capital.entry(trade.buyer).or_default() += trade.qty;
                        *capital.entry(trade.seller).or_default() -= trade.qty;

                        true
                    })
                    .map(|(_, x, _)| x),
            )
        }

        all_trades.into_iter()
    }

    pub fn inner(&self) -> &BTreeMap<CommodityKind, SingleMarket> {
        &self.markets
    }
}

#[cfg(test)]
mod tests {
    use super::Market;
    use crate::economy::CommodityKind;
    use crate::SoulID;
    use geom::{vec2, Vec2};
    use hecs::Entity;

    fn mk_ent(id: u64) -> Entity {
        unsafe { std::mem::transmute(id) }
    }

    #[test]
    fn test_match_orders() {
        let seller = SoulID(mk_ent(1));
        let seller_far = SoulID(mk_ent(2));
        let buyer = SoulID(mk_ent(3));

        let mut m = Market::default();

        m.produce(seller, CommodityKind::Cereal, 3);
        m.produce(seller_far, CommodityKind::Cereal, 3);

        m.buy(buyer, Vec2::ZERO, CommodityKind::Cereal, 2);
        m.sell(seller, Vec2::X, CommodityKind::Cereal, 3);
        m.sell(seller_far, vec2(10.0, 10.0), CommodityKind::Cereal, 3);

        let trades = m.make_trades().collect::<Vec<_>>();

        assert_eq!(trades.len(), 1);
        let t0 = trades[0];
        assert_eq!(t0.seller, seller);
        assert_eq!(t0.buyer, buyer);
        assert_eq!(t0.qty, 2);
    }
}
