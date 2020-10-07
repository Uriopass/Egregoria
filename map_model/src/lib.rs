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

mod objects {
    mod building;
    mod intersection;
    mod lane;
    mod lot;
    mod parking;
    mod road;
    mod turn;

    pub use building::*;
    pub use intersection::*;
    pub use lane::*;
    pub use lot::*;
    pub use parking::*;
    pub use road::*;
    pub use turn::*;
}

pub use objects::*;

mod procgen {
    pub mod heightmap;
    mod presets;

    pub use presets::*;
}

pub use procgen::*;

mod light_policy;
mod map;
mod pathfinding;
mod serializing;
mod spatial_map;
mod traffic_control;
mod traversable;
mod turn_policy;

// Use self or else it would be ambiguous with "pathfinding" crate
pub use self::pathfinding::*;
pub use light_policy::*;
pub use map::*;
pub use serializing::*;
pub use spatial_map::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn_policy::*;

pub const CROSSWALK_WIDTH: f32 = 4.0;
