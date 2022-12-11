use crate::map::BuildingID;
use serde::{Deserialize, Serialize};

pub mod pedestrian;
pub mod road;
pub mod train;
mod vehicle;

pub use pedestrian::*;
pub use vehicle::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}
debug_inspect_impl!(Location);
