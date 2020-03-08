use crate::map_model::traffic_control::TrafficControl;
use specs::World;

mod intersection;
mod lane;
mod map;
mod navmesh;
mod road;
mod road_graph_synchronize;
mod saveload;
mod traffic_control;
mod traversable;
mod turn;

pub use intersection::*;
pub use lane::*;
pub use map::*;
pub use navmesh::*;
pub use road::*;
pub use road_graph_synchronize::*;
pub use saveload::*;
pub use traffic_control::*;
pub use traversable::*;
pub use turn::*;

pub fn setup(world: &mut World) {
    load(world);
}
