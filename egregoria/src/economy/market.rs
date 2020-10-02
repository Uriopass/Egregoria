use crate::economy::{EconomicAgent, Goods, Transaction};
use crate::SoulID;
use std::collections::HashMap;

#[derive(Default)]
pub struct Market {
    pub agents: HashMap<SoulID, EconomicAgent>,
    pub for_sale: HashMap<SoulID, Vec<Transaction>>,
}

impl Market {
    pub fn propose(&mut self, soul: SoulID, transactions: Vec<Transaction>) {
        self.for_sale.insert(soul, transactions);
    }

    pub fn want(&self, seller: SoulID, goods: Goods) -> Option<Transaction> {
        self.for_sale
            .get(&seller)?
            .iter()
            .copied()
            .filter(|trans| trans.delta.is_smaller(&goods))
            .min_by_key(|trans| trans.cost)
    }

    pub fn apply(&mut self, buyer_id: SoulID, seller_id: SoulID, transaction: Transaction) -> bool {
        if buyer_id == seller_id {
            log::warn!(
                "Trying to sell {:?} to itself ({:?})",
                transaction,
                buyer_id
            );
            return false;
        }

        let (buyer, seller) = match common::get_mut_pair(&mut self.agents, &buyer_id, &seller_id) {
            Some(x) => x,
            None => {
                log::warn!(
                    "Trying to apply transaction to non existing agents: {:?} and/or {:?}",
                    buyer_id,
                    seller_id
                );
                return false;
            }
        };

        if buyer.money < transaction.cost {
            log::warn!(
                "Buyer {:?} doesnt have enough {:?} to fullfill {:?}",
                buyer.id,
                buyer.money,
                transaction
            );
            return false;
        }

        if !transaction.delta.is_smaller(&seller.goods) {
            log::warn!(
                "Seller {:?} doesn't have enough {:?} to fullfill {:?}",
                seller.id,
                seller.goods,
                transaction
            );
            return false;
        }

        seller.money += transaction.cost;
        seller.goods -= transaction.delta;

        buyer.money -= transaction.cost;
        buyer.goods += transaction.delta;

        true
    }
}
