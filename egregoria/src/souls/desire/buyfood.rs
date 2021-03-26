use crate::economy::{Bought, CommodityKind, Market};
use crate::map_dynamic::{BuildingInfos, Destination, Router};
use crate::souls::desire::Desire;
use crate::{ParCommandBuffer, SoulID};
use common::{GameInstant, GameTime};
use geom::Transform;
use legion::{system, Entity};
use map_model::BuildingID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum BuyFoodState {
    Empty,
    WaitingForTrade,
    BoughtAt(BuildingID),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BuyFood {
    last_ate: GameInstant,
    state: BuyFoodState,
}

debug_inspect_impl!(BuyFood);

impl BuyFood {
    pub fn new(start: GameInstant) -> Self {
        BuyFood {
            last_ate: start,
            state: BuyFoodState::Empty,
        }
    }
}

register_system!(desire_buy_food);
#[system(par_for_each)]
pub fn desire_buy_food(
    #[resource] cbuf: &ParCommandBuffer,
    #[resource] binfos: &BuildingInfos,
    #[resource] time: &GameTime,
    me: &Entity,
    trans: &Transform,
    router: &mut Router,
    d: &mut Desire<BuyFood>,
    bought: &mut Bought,
) {
    let soul = SoulID(*me);
    let pos = trans.position();
    let buy_food = &mut d.v;
    if d.was_max {
        match buy_food.state {
            BuyFoodState::Empty => {
                cbuf.exec_on(move |market: &mut Market| {
                    market.buy(soul, pos, CommodityKind::Bread, 1)
                });
                buy_food.state = BuyFoodState::WaitingForTrade;
            }
            BuyFoodState::WaitingForTrade => {
                for trade in bought.0.entry(CommodityKind::Bread).or_default().drain(..) {
                    if let Some(b) = binfos.building_owned_by(trade.seller) {
                        buy_food.state = BuyFoodState::BoughtAt(b);
                    }
                }
            }
            BuyFoodState::BoughtAt(b) => {
                if router.go_to(Destination::Building(b)) {
                    buy_food.state = BuyFoodState::Empty;
                    buy_food.last_ate = time.instant();
                }
            }
        }
    }
    if matches!(buy_food.state, BuyFoodState::WaitingForTrade)
        && bought
            .0
            .get(&CommodityKind::Bread)
            .map(|x| x.is_empty())
            .unwrap_or(true)
    {
        d.score = 0.0;
        return;
    }
    d.score = buy_food.last_ate.elapsed(time) as f32 * 0.001 - 1.0
}
