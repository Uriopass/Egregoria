use crate::map::{make_inter_entity, RoadGraph};
use specs::{LazyUpdate, World, WorldExt};

const GRAPH_FILENAME: &str = "world/graph";

pub fn save(world: &mut World) {
    world.read_resource::<RoadGraph>().save(GRAPH_FILENAME);
}

pub fn load(world: &mut World) {
    let rg = RoadGraph::from_file(GRAPH_FILENAME).unwrap_or_else(RoadGraph::empty);
    for (inter_id, inter) in rg.intersections() {
        make_inter_entity(
            *inter_id,
            inter.pos,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
        );
    }
    world.insert(rg);
}
