use crate::interaction::Selectable;
use crate::map_model::{Lane, LaneID, Map, Traversable};
use crate::physics::{add_transport_to_coworld, Collider, CollisionWorld, Kinematics, Transform};
use crate::rendering::meshrender_component::MeshRender;
use cgmath::{vec2, InnerSpace};
use rand::random;
use specs::{Builder, Entity, World, WorldExt};

mod data;
mod saveload;
pub mod systems;

pub use data::*;
pub use saveload::*;

pub fn spawn_new_transport(world: &mut World) {
    let mut pos = Transform::new(vec2(0.0, 0.0));
    let mut obj = TransportObjective::None;

    let kind = TransportKind::Car;

    {
        let map = world.read_resource::<Map>();
        let roads = map.roads();
        let l = roads.len();
        if l > 0 {
            let r = (random::<f32>() * l as f32) as usize;

            let (_, road) = roads.into_iter().nth(r).unwrap();
            let lanes = road
                .lanes_iter()
                .filter(|x| map.lanes()[**x].kind.vehicles())
                .collect::<Vec<&LaneID>>();

            if !lanes.is_empty() {
                let r = (random::<f32>() * lanes.len() as f32) as usize;

                let lane: &Lane = &map.lanes()[*lanes[r]];
                if let [a, .., b] = lane.points.as_slice() {
                    let diff = b - a;
                    pos.set_position(*a + random::<f32>() * diff);
                    pos.set_direction(diff.normalize());
                    obj = TransportObjective::Temporary(Traversable::Lane(lane.id));
                }
            }
        }
    }

    make_transport_entity(world, pos, TransportComponent::new(obj, kind));
}

pub fn make_transport_entity(
    world: &mut World,
    trans: Transform,
    transport: TransportComponent,
) -> Entity {
    let mut mr = MeshRender::empty(3);

    transport.kind.build_mr(&mut mr);

    let e = world
        .create_entity()
        .with(mr)
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(transport)
        //.with(Movable)
        .with(Selectable::default())
        .build();

    add_transport_to_coworld(world, e);
    e
}

pub fn delete_transport_entity(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}

pub fn setup(world: &mut World) {
    load(world);
}
