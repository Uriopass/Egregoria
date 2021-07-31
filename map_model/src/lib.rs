#![allow(
    clippy::too_many_arguments,
    clippy::many_single_char_names,
    clippy::upper_case_acronyms,
    clippy::collapsible_else_if,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![deny(
    clippy::indexing_slicing,
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates
)]

#[macro_use]
extern crate common;

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

pub mod procgen {
    mod building;
    pub mod heightmap;
    mod presets;

    pub use building::*;
    pub use presets::*;
}

mod light_policy;
mod map;
mod pathfinding;
mod serializing;
mod spatial_map;
mod terrain;
mod traffic_control;
mod traversable;
mod turn_policy;

// Use self or else it would be ambiguous with "pathfinding" crate
pub use self::pathfinding::*;
pub use light_policy::*;
pub use map::*;
pub use spatial_map::*;
pub use terrain::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn_policy::*;

pub use ::pathfinding as pathfinding_crate;

pub const CROSSWALK_WIDTH: f32 = 2.0;
