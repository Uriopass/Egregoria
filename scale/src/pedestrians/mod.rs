use crate::interaction::Selectable;
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use specs::{Builder, World, WorldExt};
pub mod data;
pub mod systems;
use crate::map_model::{Itinerary, LaneKind, Map, Traversable, TraverseDirection, TraverseKind};
pub use data::*;
pub use systems::*;

pub fn setup(world: &mut World) {
    for _ in 0..3000 {
        spawn_pedestrian(world);
    }
}

pub fn spawn_pedestrian(world: &mut World) {
    let map = world.read_resource::<Map>();

    let lane = unwrap_ret!(map.get_random_lane(LaneKind::Walking));

    let pos = if let [a, b, ..] = lane.points.as_slice() {
        a + (b - a) * rand::random()
    } else {
        return;
    };

    let mut itinerary = Itinerary::default();
    itinerary.set_simple(
        Traversable::new(TraverseKind::Lane(lane.id), TraverseDirection::Forward),
        &map,
    );
    itinerary.advance(&map);
    drop(map);

    let h = world.get_mut::<CollisionWorld>().unwrap().insert(
        pos,
        PhysicsObject {
            radius: 0.3,
            group: PhysicsGroup::Pedestrians,
            ..Default::default()
        },
    );
    world
        .create_entity()
        .with(Transform::new(pos))
        .with(PedestrianComponent {
            itinerary,
            ..Default::default()
        })
        .with(Kinematics::from_mass(80.0))
        .with(MeshRender::simple(
            CircleRender {
                radius: 0.3,
                ..Default::default()
            },
            3,
        ))
        .with(Collider(h))
        .with(Selectable::new(0.5))
        .build();
}
