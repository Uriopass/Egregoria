use crate::map::BuildingID;
use serde::{Deserialize, Serialize};

pub mod pedestrian;
pub mod road;
pub mod testing_vehicles;
pub mod train;
mod vehicle;

use crate::world::VehicleID;
pub use pedestrian::*;
pub use vehicle::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}
debug_inspect_impl!(Location);
