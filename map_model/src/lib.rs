#![allow(clippy::too_many_arguments)]

macro_rules! unwrap_or {
    ($e: expr, $t: expr) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

#[macro_use]
extern crate log;

mod building;
mod intersection;
mod lane;
mod light_policy;
mod lot;
mod map;
mod mapgen;
mod parking;
mod pathfinding;
mod road;
mod serializing;
mod spatial_map;
mod traffic_control;
mod traversable;
mod turn;
mod turn_policy;

// Use self or else it would be ambiguous with "pathfinding" crate
pub use self::pathfinding::*;
pub use building::*;
pub use intersection::*;
pub use lane::*;
pub use light_policy::*;
pub use lot::*;
pub use map::*;
pub use mapgen::*;
pub use parking::*;
pub use road::*;
pub use serializing::*;
pub use spatial_map::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn::*;
pub use turn_policy::*;

pub const CROSSWALK_WIDTH: f32 = 4.0;
