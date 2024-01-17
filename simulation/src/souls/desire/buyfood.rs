use serde::{Deserialize, Serialize};

use egui_inspect::Inspect;
use geom::Transform;
use prototypes::{GameInstant, GameTime, ItemID};

use crate::economy::{find_trade_place, Bought, Market};
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Destination};
use crate::souls::human::HumanDecisionKind;
use crate::transportation::Location;
use crate::world::{HumanEnt, HumanID};
use crate::{ParCommandBuffer, SoulID};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum BuyFoodState {
    Empty,
    WaitingForTrade,
    BoughtAt(BuildingID),
}

debug_inspect_impl!(BuyFoodState);

#[derive(Inspect, Clone, Serialize, Deserialize, Debug)]
pub struct BuyFood {
    pub last_ate: GameInstant,
    state: BuyFoodState,
    pub last_score: f32,
}

impl BuyFood {
    pub fn new(start: GameInstant) -> Self {
        BuyFood {
            last_ate: start,
            state: BuyFoodState::Empty,
            last_score: 0.0,
        }
    }

    pub fn score(&self, time: &GameTime, loc: &Location, bought: &Bought) -> f32 {
        if matches!(self.state, BuyFoodState::WaitingForTrade)
            && bought
                .0
                .get(&ItemID::new("bread"))
                .map(Vec::is_empty)
                .unwrap_or(false)
        {
            return 0.0;
        }
        if let BuyFoodState::BoughtAt(id) = self.state {
            if loc == &Location::Building(id) {
                return 1.0;
            }
        }
        self.last_ate.elapsed(time).seconds() as f32 / GameTime::DAY as f32 - 1.0
    }

    pub fn apply(
        &mut self,
        cbuf: &ParCommandBuffer<HumanEnt>,
        binfos: &BuildingInfos,
        time: &GameTime,
        id: HumanID,
        trans: &Transform,
        loc: &Location,
        bought: &mut Bought,
    ) -> HumanDecisionKind {
        use HumanDecisionKind::*;
        match self.state {
            BuyFoodState::Empty => {
                let pos = trans.pos;
                cbuf.exec_on(id, move |market: &mut Market| {
                    market.buy(SoulID::Human(id), pos.xy(), ItemID::new("bread"), 1)
                });
                self.state = BuyFoodState::WaitingForTrade;
                Yield
            }
            BuyFoodState::WaitingForTrade => {
                for trade in bought.0.entry(ItemID::new("bread")).or_default().drain(..) {
                    if let Some(b) = find_trade_place(trade.seller, binfos) {
                        self.state = BuyFoodState::BoughtAt(b);
                    }
                }
                Yield
            }
            BuyFoodState::BoughtAt(b) => {
                if loc == &Location::Building(b) {
                    self.state = BuyFoodState::Empty;
                    self.last_ate = time.instant();
                    log::debug!("{:?} ate at {:?}", id, b);
                    Yield
                } else {
                    GoTo(Destination::Building(b))
                }
            }
        }
    }
}
