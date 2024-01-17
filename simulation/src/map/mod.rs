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

mod change_detection;
mod electricity_cache;
mod height_override;
mod light_policy;
#[allow(clippy::module_inception)]
mod map;
mod pathfinding;
mod serializing;
mod spatial_map;
pub mod terrain;
mod traffic_control;
mod traversable;
mod turn_policy;

// Use self or else it would be ambiguous with "pathfinding" crate
pub use self::pathfinding::*;
pub use change_detection::*;
pub use electricity_cache::*;
pub use light_policy::*;
pub use map::*;
pub use spatial_map::*;
pub use terrain::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn_policy::*;

pub use ::pathfinding as pathfinding_crate;

pub const CROSSWALK_WIDTH: f32 = 2.0;
pub const ROAD_Z_OFFSET: f32 = 0.3;
pub const MAX_SLOPE: f32 = 0.25; // 25% grade
