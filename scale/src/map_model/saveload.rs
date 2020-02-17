use crate::map_model::Map;
use specs::{LazyUpdate, World, WorldExt};

const GRAPH_FILENAME: &str = "world/graph";

pub fn save(_world: &mut World) {
    //world.read_resource::<NavMesh>().save(GRAPH_FILENAME);
}

pub fn load(world: &mut World) {
    let map = Map::empty();
    world.insert(map);
    /*
    let navmesh = NavMesh::from_file(GRAPH_FILENAME).unwrap_or_else(NavMesh::empty);

        for (inter_id, inter) in navmesh.intersections() {
        make_inter_entity(
            *inter_id,
            inter.pos,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
        );
    }
    world.insert(navmesh);
    */
}
