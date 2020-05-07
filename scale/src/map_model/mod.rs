use crate::map_model::traffic_control::TrafficControl;
use specs::World;

mod intersection;
mod itinerary;
mod lane;
mod light_policy;
mod map;
mod pathfinding;
mod road;
mod saveload;
mod traffic_control;
mod traversable;
mod turn;
mod turn_policy;

pub use self::pathfinding::*;
pub use intersection::*;
pub use itinerary::*;
pub use lane::*;
pub use light_policy::*;
pub use map::*;
pub use road::*;
pub use saveload::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn::*;
pub use turn_policy::*;

pub fn setup(world: &mut World) {
    load(world);
}
