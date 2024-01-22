use crate::map::BuildingID;
use crate::map_dynamic::{Destination, Router};
use crate::souls::human::HumanDecisionKind;
use crate::transportation::Location;
use crate::world::VehicleID;
use egui_inspect::Inspect;
use prototypes::{GameTime, RecTimeInterval, MINUTES_PER_HOUR};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WorkKind {
    Driver {
        deliver_order: Option<BuildingID>,
        truck: VehicleID,
    },
    Worker,
}
debug_inspect_impl!(WorkKind);

#[derive(Inspect, Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub workplace: BuildingID,
    pub work_inter: RecTimeInterval,
    pub kind: WorkKind,
    pub last_score: f32,
}

impl Work {
    pub fn new(workplace: BuildingID, kind: WorkKind, offset: f32) -> Self {
        Work {
            workplace,
            work_inter: RecTimeInterval::new(
                (8, (offset * MINUTES_PER_HOUR as f32) as i32),
                (18, (offset * MINUTES_PER_HOUR as f32) as i32),
            ),
            kind,
            last_score: 0.0,
        }
    }

    pub fn apply(&mut self, loc: &Location, router: &Router) -> HumanDecisionKind {
        use HumanDecisionKind::*;
        match self.kind {
            WorkKind::Worker => GoTo(Destination::Building(self.workplace)),
            WorkKind::Driver {
                deliver_order,
                truck,
            } => {
                if &Location::Building(self.workplace) != loc {
                    MultiStack(vec![
                        GoTo(Destination::Building(self.workplace)),
                        SetVehicle(router.personal_car),
                    ])
                } else if let Some(b) = deliver_order {
                    MultiStack(vec![
                        SetVehicle(router.personal_car),
                        GoTo(Destination::Building(self.workplace)),
                        DeliverAtBuilding(b),
                        GoTo(Destination::Building(b)),
                        SetVehicle(Some(truck)),
                    ])
                } else {
                    Yield
                }
            }
        }
    }

    pub fn score(&self, time: &GameTime) -> f32 {
        if self.work_inter.dist_start(&time.daytime) == 0 {
            0.5
        } else {
            0.0
        }
    }
}
