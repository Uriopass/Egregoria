use crate::economy::{find_trade_place, Bought, ItemID, ItemRegistry, Market};
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Destination};
use crate::souls::human::HumanDecisionKind;
use crate::transportation::Location;
use crate::utils::time::{GameInstant, GameTime};
use crate::world::{HumanEnt, HumanID};
use crate::{Map, ParCommandBuffer, SoulID};
use egui_inspect::Inspect;
use geom::Transform;
use serde::{Deserialize, Serialize};

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
    bread: ItemID,
}

impl BuyFood {
    pub fn new(start: GameInstant, registry: &ItemRegistry) -> Self {
        BuyFood {
            last_ate: start,
            state: BuyFoodState::Empty,
            bread: registry.id("bread"),
        }
    }

    pub fn score(&self, time: &GameTime, loc: &Location, bought: &Bought) -> f32 {
        if matches!(self.state, BuyFoodState::WaitingForTrade)
            && bought
                .0
                .get(&self.bread)
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
        self.last_ate.elapsed(time) as f32 / GameTime::DAY as f32 - 1.0
    }

    pub fn apply(
        &mut self,
        cbuf: &ParCommandBuffer<HumanEnt>,
        binfos: &BuildingInfos,
        map: &Map,
        time: &GameTime,
        id: HumanID,
        trans: &Transform,
        loc: &Location,
        bought: &mut Bought,
    ) -> HumanDecisionKind {
        use HumanDecisionKind::*;
        match self.state {
            BuyFoodState::Empty => {
                let pos = trans.position;
                let bread = self.bread;
                cbuf.exec_on(id, move |market: &mut Market| {
                    market.buy(SoulID::Human(id), pos.xy(), bread, 1)
                });
                self.state = BuyFoodState::WaitingForTrade;
                Yield
            }
            BuyFoodState::WaitingForTrade => {
                for trade in bought.0.entry(self.bread).or_default().drain(..) {
                    if let Some(b) =
                        find_trade_place(trade.seller, trans.position.xy(), binfos, map)
                    {
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
