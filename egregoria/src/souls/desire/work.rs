use crate::map::BuildingID;
use crate::map_dynamic::{Destination, Router};
use crate::pedestrians::Location;
use crate::souls::human::HumanDecisionKind;
use crate::utils::time::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use crate::vehicles::VehicleID;
use imgui_inspect_derive::Inspect;
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

#[derive(Inspect, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Work {
    workplace: BuildingID,
    work_inter: RecTimeInterval,
    pub kind: WorkKind,
    on_mission: bool,
}

impl Work {
    pub fn new(workplace: BuildingID, kind: WorkKind, offset: f32) -> Self {
        Work {
            workplace,
            work_inter: RecTimeInterval::new(
                (8, (offset * SECONDS_PER_HOUR as f32) as i32),
                (18, (offset * SECONDS_PER_HOUR as f32) as i32),
            ),
            kind,
            on_mission: false,
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
        if self.on_mission || self.work_inter.dist_until(time.daytime) == 0 {
            0.5
        } else {
            0.0
        }
    }
}
