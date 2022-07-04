use crate::map::BuildingID;
use crate::vehicles::VehicleID;
use serde::{Deserialize, Serialize};

pub mod data;
pub mod systems;

pub use data::*;
pub use systems::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}

debug_inspect_impl!(Location);
