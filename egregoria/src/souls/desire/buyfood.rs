use crate::economy::{Bought, ItemID, ItemRegistry, Market};
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Destination};
use crate::pedestrians::Location;
use crate::souls::human::HumanDecisionKind;
use crate::utils::time::{GameInstant, GameTime};
use crate::{ParCommandBuffer, SoulID};
use geom::Transform;
use imgui_inspect_derive::Inspect;
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
    last_ate: GameInstant,
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
        cbuf: &ParCommandBuffer,
        binfos: &BuildingInfos,
        time: &GameTime,
        soul: SoulID,
        trans: &Transform,
        loc: &Location,
        bought: &mut Bought,
    ) -> HumanDecisionKind {
        use HumanDecisionKind::*;
        match self.state {
            BuyFoodState::Empty => {
                let pos = trans.position;
                let bread = self.bread;
                cbuf.exec_on(soul.0, move |market: &mut Market| {
                    market.buy(soul, pos.xy(), bread, 1)
                });
                self.state = BuyFoodState::WaitingForTrade;
                Yield
            }
            BuyFoodState::WaitingForTrade => {
                for trade in bought.0.entry(self.bread).or_default().drain(..) {
                    if let Some(b) = binfos.building_owned_by(trade.seller) {
                        self.state = BuyFoodState::BoughtAt(b);
                    }
                }
                Yield
            }
            BuyFoodState::BoughtAt(b) => {
                if loc == &Location::Building(b) {
                    self.state = BuyFoodState::Empty;
                    self.last_ate = time.instant();
                    log::debug!("{:?} ate at {:?}", soul, b);
                    Yield
                } else {
                    GoTo(Destination::Building(b))
                }
            }
        }
    }
}
