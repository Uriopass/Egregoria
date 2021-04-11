use crate::map_dynamic::{Destination, Router};
use crate::souls::desire::Desire;
use crate::utils::time::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use crate::vehicles::VehicleID;
use imgui_inspect_derive::*;
use legion::system;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DriverState {
    GoingToWork,
    WaitingForDelivery,
    Delivering(BuildingID),
    DeliveryBack,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WorkKind {
    Driver {
        state: DriverState,
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
}

register_system!(desire_work);
#[system(par_for_each)]
pub fn desire_work(#[resource] time: &GameTime, router: &mut Router, d: &mut Desire<Work>) {
    d.score_and_apply(
        |work| {
            if work.on_mission || work.work_inter.dist_until(time.daytime) == 0 {
                0.5
            } else {
                0.0
            }
        },
        |work| match work.kind {
            WorkKind::Worker => {
                router.go_to(Destination::Building(work.workplace));
            }
            WorkKind::Driver {
                ref mut state,
                truck,
            } => match *state {
                DriverState::GoingToWork => {
                    router.use_vehicle(router.personal_car);
                    if router.go_to(Destination::Building(work.workplace)) {
                        log::info!(
                            "hello I'm a driver and I arrived at {:?}. Ready to serve!",
                            work.workplace
                        );
                        *state = DriverState::WaitingForDelivery;
                    }
                }
                DriverState::WaitingForDelivery => {}
                DriverState::Delivering(b) => {
                    router.use_vehicle(Some(truck));
                    if router.go_to(Destination::Building(b)) {
                        log::info!("finished delivering to {:?} from {:?}", b, work.workplace);
                        *state = DriverState::DeliveryBack
                    }
                }
                DriverState::DeliveryBack => {
                    router.use_vehicle(Some(truck));
                    if router.go_to(Destination::Building(work.workplace)) {
                        *state = DriverState::WaitingForDelivery;
                    }
                }
            },
        },
    )
}
