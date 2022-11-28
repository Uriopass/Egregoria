use crate::economy::{Item, ItemID, ItemRegistry, Money, WORKER_CONSUMPTION_PER_SECOND};
use crate::map::BuildingID;
use crate::map_dynamic::BuildingInfos;
use crate::souls::goods_company::GoodsCompanyID;
use crate::{BuildingKind, GoodsCompanyRegistry, Map, SoulID};
use geom::Vec2;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SellOrder {
    pub pos: Vec2,
    pub qty: u32,
    /// When selling less than stock, should not enable external trading
    pub stock: u32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BuyOrder {
    pub pos: Vec2,
    pub qty: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SingleMarket {
    // todo: change i32 to Quantity
    capital: BTreeMap<SoulID, i32>,
    buy_orders: BTreeMap<SoulID, BuyOrder>,
    sell_orders: BTreeMap<SoulID, SellOrder>,
    pub ext_value: Money,
    pub transport_cost: Money,
    optout_exttrade: bool,
}

impl SingleMarket {
    pub fn new(ext_value: Money, transport_cost: Money, optout_exttrade: bool) -> Self {
        Self {
            capital: Default::default(),
            buy_orders: Default::default(),
            sell_orders: Default::default(),
            ext_value,
            transport_cost,
            optout_exttrade,
        }
    }

    pub fn capital(&self, soul: SoulID) -> Option<i32> {
        self.capital.get(&soul).copied()
    }
    pub fn buy_order(&self, soul: SoulID) -> Option<&BuyOrder> {
        self.buy_orders.get(&soul)
    }
    pub fn sell_order(&self, soul: SoulID) -> Option<&SellOrder> {
        self.sell_orders.get(&soul)
    }

    pub fn capital_map(&self) -> &BTreeMap<SoulID, i32> {
        &self.capital
    }
}

#[derive(Serialize, Deserialize)]
pub struct Market {
    markets: BTreeMap<ItemID, SingleMarket>,
    // reuse the trade vec to avoid allocations
    #[serde(skip)]
    all_trades: Arc<Vec<Trade>>,
    // reuse the potential vec to avoid allocations
    #[serde(skip)]
    potential: Vec<(Trade, f32)>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TradeTarget {
    Soul(SoulID),
    ExternalTrade,
}

impl TradeTarget {
    pub(crate) fn soul(self) -> SoulID {
        match self {
            TradeTarget::Soul(soul) => soul,
            TradeTarget::ExternalTrade => panic!("Cannot get soul from external trade"),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    pub buyer: TradeTarget,
    pub seller: TradeTarget,
    pub qty: i32,
    pub kind: ItemID,
}

pub fn find_trade_place(
    target: TradeTarget,
    pos: Vec2,
    binfos: &BuildingInfos,
    map: &Map,
) -> Option<BuildingID> {
    match target {
        TradeTarget::Soul(id) => binfos.building_owned_by(id),
        TradeTarget::ExternalTrade => {
            map.bkinds
                .get(&BuildingKind::RailFretStation)
                .and_then(|b| {
                    b.iter()
                        .filter_map(|&bid| map.buildings.get(bid))
                        .min_by_key(|&b| OrderedFloat(b.door_pos.xy().distance2(pos)))
                        .map(|x| x.id)
                })
        }
    }
}

impl Market {
    pub fn new(registry: &ItemRegistry, companies: &GoodsCompanyRegistry) -> Self {
        let prices = calculate_prices(registry, companies);
        Self {
            markets: registry
                .iter()
                .map(|v| {
                    (
                        v.id,
                        SingleMarket::new(prices[&v.id], Money::new_base(10), v.optout_exttrade),
                    )
                })
                .collect(),
            all_trades: Default::default(),
            potential: Default::default(),
        }
    }

    pub fn m(&mut self, kind: ItemID) -> &mut SingleMarket {
        self.markets.get_mut(&kind).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ItemID, &SingleMarket)> {
        self.markets.iter()
    }

    /// Called when an agent tells the world it wants to sell something
    /// If an order is already placed, it will be updated.
    /// Beware that you need capital to sell anything, using produce.
    pub fn sell(&mut self, soul: SoulID, near: Vec2, kind: ItemID, qty: u32, stock: u32) {
        log::info!("{:?} sell {:?} {:?} near {:?}", soul, qty, kind, near);
        self.m(kind).sell_orders.insert(
            soul,
            SellOrder {
                pos: near,
                qty,
                stock,
            },
        );
    }

    pub fn sell_all(&mut self, soul: SoulID, near: Vec2, kind: ItemID, stock: u32) {
        let c = self.capital(soul, kind);
        if c <= 0 {
            return;
        }
        self.sell(soul, near, kind, c as u32, stock);
    }

    /// An agent was removed from the world, we need to clean after him
    pub fn remove(&mut self, soul: SoulID) {
        for market in self.markets.values_mut() {
            market.sell_orders.remove(&soul);
            market.buy_orders.remove(&soul);
            market.capital.remove(&soul);
        }
    }

    /// Called when an agent tells the world it wants to buy something
    /// If an order is already placed, it will be updated.
    pub fn buy(&mut self, soul: SoulID, near: Vec2, kind: ItemID, qty: u32) {
        log::debug!("{:?} buy {:?} {:?} near {:?}", soul, qty, kind, near);

        self.m(kind)
            .buy_orders
            .insert(soul, BuyOrder { pos: near, qty });
    }

    pub fn buy_until(&mut self, soul: SoulID, near: Vec2, kind: ItemID, qty: u32) {
        let c = self.capital(soul, kind);
        if c >= qty as i32 {
            return;
        }
        self.buy(soul, near, kind, qty - c as u32);
    }

    /// Get the capital that this agent owns
    pub fn capital(&self, soul: SoulID, kind: ItemID) -> i32 {
        self.markets.get(&kind).unwrap().capital(soul).unwrap_or(0)
    }

    /// Registers a soul to the market, not obligatory
    pub fn register(&mut self, soul: SoulID, kind: ItemID) {
        self.m(kind).capital.entry(soul).or_default();
    }

    /// Called whenever an agent (like a farm) produces something on it's own
    /// for example wheat is harvested or turned into flour. Returns the new quantity owned.
    pub fn produce(&mut self, soul: SoulID, kind: ItemID, delta: i32) -> i32 {
        log::debug!("{:?} produced {:?} {:?}", soul, delta, kind);

        let v = self.m(kind).capital.entry(soul).or_default();
        *v += delta;
        *v
    }

    /// Returns a list of buy and sell orders matched together.
    /// A trade updates the buy and sell orders from the market, and the capital of the buyers and sellers.
    /// A trade can only be completed if the seller has enough capital.
    /// Please do not keep the trades around much, it needs to be destroyed by the next time you call this function.
    pub fn make_trades(&mut self) -> Arc<Vec<Trade>> {
        let all_trades: &mut Vec<Trade> = match Arc::get_mut(&mut self.all_trades) {
            None => {
                log::warn!("Market trades not dropped in time");
                self.all_trades = Default::default();
                Arc::get_mut(&mut self.all_trades).unwrap()
            }
            Some(x) => x,
        };
        all_trades.clear();
        let potential = &mut self.potential;

        for (&kind, market) in &mut self.markets {
            // Naive O(n²) alg
            // We don't immediatly apply the trades, because we want to find the nearest-positioned trades
            for (&seller, sorder) in &market.sell_orders {
                let qty_sell = sorder.qty as i32;

                let capital_sell = unwrap_or!(market.capital(seller), continue);
                if qty_sell > capital_sell {
                    continue;
                }
                for (&buyer, &border) in &market.buy_orders {
                    if seller == buyer {
                        log::warn!(
                            "{:?} is both selling and buying same commodity: {:?}",
                            seller,
                            kind
                        );
                        continue;
                    }
                    let qty_buy = border.qty as i32;
                    if qty_buy > qty_sell {
                        continue;
                    }
                    let score = sorder.pos.distance2(border.pos);
                    potential.push((
                        Trade {
                            buyer: TradeTarget::Soul(buyer),
                            seller: TradeTarget::Soul(seller),
                            qty: qty_buy,
                            kind,
                        },
                        score,
                    ))
                }
            }
            potential.sort_unstable_by_key(|(_, x)| OrderedFloat(*x));
            let SingleMarket {
                buy_orders,
                sell_orders,
                capital,
                optout_exttrade,
                ..
            } = market;

            all_trades.extend(potential.drain(..).filter_map(|(trade, _)| {
                let buyer = trade.buyer.soul();
                let seller = trade.seller.soul();

                let cap_seller = capital.entry(seller).or_default() as *mut _;
                // Safety: buyer cannot be the same as seller
                if unsafe { *cap_seller } < trade.qty {
                    return None;
                }

                let cap_buyer = capital.entry(buyer).or_default();
                let border = buy_orders.entry(buyer);

                match border {
                    Entry::Vacant(_) => return None,
                    Entry::Occupied(o) => o.remove(),
                };

                let sorderent = sell_orders.entry(seller);

                let mut sorderocc = match sorderent {
                    Entry::Vacant(_) => return None,
                    Entry::Occupied(o) => o,
                };

                let sorder = sorderocc.get_mut();

                if sorder.qty < trade.qty as u32 {
                    return None;
                }

                sorder.qty -= trade.qty as u32;

                if sorder.qty == 0 {
                    sorderocc.remove();
                }

                // Safety: buyer cannot be the same as seller
                *cap_buyer += trade.qty;
                unsafe { *cap_seller -= trade.qty };

                Some(trade)
            }));

            // External trading
            if !*optout_exttrade {
                // All buyers can fullfil since they can buy externally
                let btaken = std::mem::take(buy_orders);
                all_trades.reserve(btaken.len());
                for (buyer, order) in btaken {
                    let qty_buy = order.qty as i32;
                    *capital.entry(buyer).or_default() += qty_buy;

                    all_trades.push(Trade {
                        buyer: TradeTarget::Soul(buyer),
                        seller: TradeTarget::ExternalTrade,
                        qty: qty_buy,
                        kind,
                    });
                }

                // Seller surplus goes to external trading
                for (&seller, order) in sell_orders.iter_mut() {
                    let qty_sell = order.qty as i32 - order.stock as i32;
                    if qty_sell <= 0 {
                        continue;
                    }
                    let cap = capital.entry(seller).or_default();
                    if *cap < qty_sell {
                        log::warn!("{:?} is selling more than it has: {:?}", &seller, qty_sell);
                        continue;
                    }
                    *cap -= qty_sell;
                    order.qty -= qty_sell as u32;

                    all_trades.push(Trade {
                        buyer: TradeTarget::ExternalTrade,
                        seller: TradeTarget::Soul(seller),
                        qty: qty_sell,
                        kind,
                    });
                }
            }
        }

        self.all_trades.clone()
    }

    pub fn inner(&self) -> &BTreeMap<ItemID, SingleMarket> {
        &self.markets
    }
}

fn calculate_prices(
    registry: &ItemRegistry,
    companies: &GoodsCompanyRegistry,
) -> BTreeMap<ItemID, Money> {
    let mut item_graph: BTreeMap<ItemID, Vec<GoodsCompanyID>> = BTreeMap::new();
    for (id, company) in companies.descriptions.iter() {
        for (itemid, _) in &company.recipe.production {
            item_graph.entry(*itemid).or_default().push(id);
        }
    }

    let mut prices = BTreeMap::new();
    fn calculate_price_inner(
        registry: &ItemRegistry,
        companies: &GoodsCompanyRegistry,
        item_graph: &BTreeMap<ItemID, Vec<GoodsCompanyID>>,
        item: &Item,
        prices: &mut BTreeMap<ItemID, Money>,
    ) {
        if prices.contains_key(&item.id) {
            return;
        }

        let mut minprice = None;
        for &comp in item_graph.get(&item.id).unwrap_or(&vec![]) {
            let company = &companies.descriptions[comp];
            let mut price_consumption = Money::ZERO;
            for &(itemid, qty) in &company.recipe.consumption {
                calculate_price_inner(registry, companies, item_graph, &registry[itemid], prices);
                price_consumption += prices[&itemid] * qty as i64;
            }
            let qty = company
                .recipe
                .production
                .iter()
                .find_map(|x| (x.0 == item.id).then(|| x.1))
                .unwrap_or(0) as i64;
            let price_workers = company.recipe.complexity as i64
                * company.n_workers as i64
                * WORKER_CONSUMPTION_PER_SECOND;

            let newprice = (price_consumption + price_workers) / qty;

            minprice = minprice.map(|x: Money| x.min(newprice)).or(Some(newprice));
        }

        prices.insert(item.id, minprice.unwrap_or(Money::ZERO));
    }

    for item in registry.iter() {
        calculate_price_inner(registry, companies, &item_graph, item, &mut prices);
    }

    prices
}

#[cfg(test)]
mod tests {
    use super::Market;
    use crate::economy::{ItemRegistry, WORKER_CONSUMPTION_PER_SECOND};
    use crate::souls::goods_company::{CompanyKind, GoodsCompanyDescription, Recipe};
    use crate::{map::BuildingGen, GoodsCompanyRegistry, SoulID};
    use geom::{vec2, Vec2};
    use hecs::Entity;

    fn mk_ent(id: u64) -> Entity {
        Entity::from_bits(id).unwrap()
    }

    #[test]
    fn test_match_orders() {
        let seller = SoulID(mk_ent((1 << 32) | 1));
        let seller_far = SoulID(mk_ent((1 << 32) | 2));
        let buyer = SoulID(mk_ent((1 << 32) | 3));

        let mut registry = ItemRegistry::default();

        registry.load_item_definitions(
            r#"
          [{
            "name": "cereal",
            "label": "Cereal"
          },
          {
            "name": "wheat",
            "label": "Wheat"
          }]
        "#,
        );

        let g = GoodsCompanyRegistry::default();

        let mut m = Market::new(&registry, &g);

        let cereal = registry.id("cereal");

        m.produce(seller, cereal, 3);
        m.produce(seller_far, cereal, 3);

        m.buy(buyer, Vec2::ZERO, cereal, 2);
        m.sell(seller, Vec2::X, cereal, 3, 5);
        m.sell(seller_far, vec2(10.0, 10.0), cereal, 3, 5);

        let trades = m.make_trades();

        assert_eq!(trades.len(), 1);
        let t0 = trades[0];
        assert_eq!(t0.seller.soul(), seller);
        assert_eq!(t0.buyer.soul(), buyer);
        assert_eq!(t0.qty, 2);
    }

    #[test]
    fn calculate_prices() {
        let mut registry = ItemRegistry::default();

        registry.load_item_definitions(
            r#"
          [{
            "name": "cereal",
            "label": "Cereal"
          },
          {
            "name": "wheat",
            "label": "Wheat"
          }]
        "#,
        );

        let cereal = registry.id("cereal");
        let wheat = registry.id("wheat");

        let mut companies = GoodsCompanyRegistry::default();

        companies
            .descriptions
            .insert_with_key(|id| GoodsCompanyDescription {
                id,
                name: "Cereal farm".to_string(),
                bgen: BuildingGen::House,
                kind: CompanyKind::Store,
                recipe: Recipe {
                    production: vec![(cereal, 3)],
                    complexity: 3,
                    consumption: vec![],
                    storage_multiplier: 5,
                },
                n_workers: 2,
                size: 0.0,
                asset_location: "".to_string(),
                price: 0,
            });

        companies
            .descriptions
            .insert_with_key(|id| GoodsCompanyDescription {
                id,
                name: "Wheat factory".to_string(),
                bgen: BuildingGen::House,
                kind: CompanyKind::Store,
                recipe: Recipe {
                    production: vec![(wheat, 2)],
                    complexity: 10,
                    consumption: vec![(cereal, 2)],
                    storage_multiplier: 5,
                },
                n_workers: 5,
                size: 0.0,
                asset_location: "".to_string(),
                price: 0,
            });

        let prices = super::calculate_prices(&registry, &companies);

        assert_eq!(prices.len(), 2);
        let price_cereal = 2 * WORKER_CONSUMPTION_PER_SECOND;
        assert_eq!(prices[&cereal], price_cereal);
        assert_eq!(
            prices[&wheat],
            (price_cereal * 2 + 5 * WORKER_CONSUMPTION_PER_SECOND * 10) / 2
        );
    }
}
