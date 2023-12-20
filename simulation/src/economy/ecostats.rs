use crate::economy::{ItemID, ItemRegistry, Money, Trade, TradeTarget};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::collections::BTreeMap;

pub const HISTORY_SIZE: usize = 128;
/// Tick to wait before the new bin
/// Which can be recovred from FREQ * HISTORY_SIZZ / TICK_RATE
pub const LEVEL_FREQS: [u64; 4] = [250, 1500, 15000, 75000];
pub const LEVEL_NAMES: [&str; 4] = ["10m", "1h", "10h", "50h"];

/// One history of one item at one frequency level
/// The past_ring is controlled by a shared cursor for all items
#[derive(Serialize, Deserialize)]
pub struct ItemHistoryLevel {
    #[serde(with = "BigArray")]
    pub past_ring_items: [i64; HISTORY_SIZE],
    #[serde(with = "BigArray")]
    pub past_ring_money: [Money; HISTORY_SIZE],
}

impl Default for ItemHistoryLevel {
    fn default() -> Self {
        Self {
            past_ring_items: [0; HISTORY_SIZE],
            past_ring_money: [Money::ZERO; HISTORY_SIZE],
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct ItemHistory {
    levels: [ItemHistoryLevel; LEVEL_FREQS.len()],
}

#[derive(Serialize, Deserialize)]
pub struct ItemHistories {
    m: BTreeMap<ItemID, ItemHistory>,
    cursors: [usize; LEVEL_FREQS.len()],
}

#[derive(Serialize, Deserialize)]
pub struct EcoStats {
    pub exports: ItemHistories,
    pub imports: ItemHistories,
    pub internal_trade: ItemHistories,
}

impl ItemHistories {
    pub fn new(registry: &ItemRegistry) -> Self {
        Self {
            m: registry
                .iter()
                .map(|item| (item.id, ItemHistory::default()))
                .collect(),
            cursors: [0; LEVEL_FREQS.len()],
        }
    }

    pub fn cursors(&self) -> &[usize] {
        &self.cursors
    }

    pub fn iter_histories(
        &self,
        level: usize,
    ) -> impl Iterator<Item = (ItemID, &ItemHistoryLevel)> {
        self.m
            .iter()
            .filter_map(move |(id, history)| Some((*id, history.levels.get(level)?)))
    }

    pub fn handle_trade(&mut self, trade: &Trade) {
        if trade.qty <= 0 {
            return;
        }
        let item = trade.kind;

        let h = self.m.get_mut(&item).unwrap();
        for (level, cursor) in h.levels.iter_mut().zip(&self.cursors) {
            // Safety: the cursor is modulo HISTORY_SIZE
            let lvl = unsafe { level.past_ring_items.get_unchecked_mut(*cursor) };
            *lvl = lvl.saturating_add(trade.qty as i64);
            let lvl = unsafe { level.past_ring_money.get_unchecked_mut(*cursor) };
            lvl.0 = lvl.0.saturating_add(trade.money_delta.0);
        }
    }

    pub fn advance(&mut self, tick: u64) {
        for (c_i, (c, freq)) in self.cursors.iter_mut().zip(&LEVEL_FREQS).enumerate() {
            if tick % *freq == 0 {
                *c = (*c + 1) % HISTORY_SIZE;
                self.m.values_mut().for_each(|h| {
                    h.levels[c_i].past_ring_money[*c] = Money::ZERO;
                    h.levels[c_i].past_ring_items[*c] = 0;
                });
            }
        }
    }
}

impl EcoStats {
    pub fn new(registry: &ItemRegistry) -> Self {
        Self {
            exports: ItemHistories::new(registry),
            imports: ItemHistories::new(registry),
            internal_trade: ItemHistories::new(registry),
        }
    }

    pub fn advance(&mut self, tick: u64, trades: &[Trade]) {
        self.exports.advance(tick);
        self.imports.advance(tick);
        self.internal_trade.advance(tick);

        for trade in trades {
            if trade.buyer == TradeTarget::ExternalTrade {
                self.exports.handle_trade(trade);
                continue;
            }
            if trade.seller == TradeTarget::ExternalTrade {
                self.imports.handle_trade(trade);
                continue;
            }
            self.internal_trade.handle_trade(trade);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::economy::HISTORY_SIZE;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn history_is_not_zero() {
        assert!(HISTORY_SIZE > 0);
    }
}
