use crate::economy::{EconomicAgent, Goods, Market, Money, Transaction};
use crate::map_dynamic::BuildingInfos;
use crate::souls::Soul;
use crate::{Egregoria, SoulID};
use map_model::BuildingID;

pub type SupermarketSoul = Soul<Supermarket, ()>;

pub struct Supermarket {
    pub id: SoulID,
}

impl Supermarket {
    pub fn soul(goria: &mut Egregoria, id: SoulID, build: BuildingID) -> SupermarketSoul {
        let agent = EconomicAgent::new(id, Money(10000), Goods { food: 1000 });

        let market: &mut Market = &mut *goria.write::<Market>();
        market.agents.insert(id, agent);
        market.for_sale.insert(
            id,
            vec![Transaction {
                cost: Money(1),
                delta: Goods { food: 1 },
            }],
        );

        goria.write::<BuildingInfos>().set_owner(build, id);

        let supermarket = Supermarket { id };

        Soul {
            desires: (),
            extra: supermarket,
        }
    }
}
