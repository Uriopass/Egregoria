use crate::economy::{EconomicAgent, Transaction};
use egregoria::SoulID;
use std::collections::HashMap;

#[derive(Default)]
pub struct Market {
    pub for_sale: HashMap<SoulID, Vec<Transaction>>,
}

impl Market {
    pub fn propose(&mut self, soul: SoulID, transactions: Vec<Transaction>) {
        self.for_sale.insert(soul, transactions);
    }

    pub fn apply(
        seller: &mut EconomicAgent,
        buyer: &mut EconomicAgent,
        transaction: Transaction,
    ) -> bool {
        if buyer.money < transaction.cost {
            return false;
        }

        if !transaction.delta.is_smaller(&seller.goods) {
            return false;
        }

        seller.money += transaction.cost;
        buyer.money -= transaction.cost;

        seller.goods -= transaction.delta;
        buyer.goods += transaction.delta;

        true
    }
}
