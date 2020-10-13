use legion::system;
use map_model::Map;

#[system]
pub fn add_trees(#[resource] map: &mut Map) {
    if map.trees.counter > 0 {
        for _ in 0..10 {
            while !map.trees.add_forest() && map.trees.counter > 0 {}
        }
    }
    map.dirty = true;
}
