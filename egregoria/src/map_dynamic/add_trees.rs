use geom::Camera;
use legion::system;
use map_model::Map;

#[system]
pub fn add_trees(#[resource] map: &mut Map, #[resource] cam: &Camera) {
    map.trees.update(*cam)
}
