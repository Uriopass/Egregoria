use crate::interaction::Selectable;
use crate::map_model::{Lane, LaneID, Map, Traversable};
use crate::physics::{add_to_coworld, Collider, Kinematics, PhysicsWorld, Transform};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::{Color, BLACK, GREEN};
use cgmath::{vec2, InnerSpace};
use specs::{Builder, Entity, World, WorldExt};

mod data;
mod saveload;
pub mod systems;
mod transport_component;

pub use data::*;
pub use saveload::*;
pub use transport_component::*;

pub fn spawn_new_car(world: &mut World) {
    let mut pos = Transform::new(vec2(0.0, 0.0));
    let mut obj = TransportObjective::None;

    {
        let map = world.read_resource::<Map>();
        let roads = map.roads();
        let l = roads.len();
        if l > 0 {
            let r = (rand::random::<f32>() * l as f32) as usize;

            let (_, road) = roads.into_iter().nth(r).unwrap();
            let lanes = road
                .lanes_forward
                .iter()
                .chain(road.lanes_backward.iter())
                .collect::<Vec<&LaneID>>();

            if !lanes.is_empty() {
                let r = (rand::random::<f32>() * lanes.len() as f32) as usize;

                let lane: &Lane = &map.lanes()[*lanes[r]];

                let a = lane.points.first().unwrap();
                let b = lane.points.last().unwrap();

                let diff = b - a;
                pos.set_position(a + rand::random::<f32>() * diff);
                pos.set_direction(diff.normalize());
                obj = TransportObjective::Temporary(Traversable::Lane(lane.id));
            }
        }
    }

    let car = TransportComponent::new(obj);

    make_transport_entity(world, pos, car);
}

pub fn make_transport_entity(
    world: &mut World,
    trans: Transform,
    transport: TransportComponent,
) -> Entity {
    let is_tank = false;
    let mut mr = MeshRender::empty(3);

    let c = Color::from_hex(0x25_66_29);
    if is_tank {
        mr.add(RectRender {
            width: 5.0,
            height: 3.0,
            color: GREEN,
            ..Default::default()
        })
        .add(RectRender {
            width: 4.0,
            height: 1.0,
            offset: [2.0, 0.0].into(),
            color: c,
            ..Default::default()
        })
        .add(CircleRender {
            radius: 0.5,
            offset: vec2(4.0, 0.0),
            color: c,
            ..Default::default()
        });
    } else {
        mr.add(RectRender {
            width: CAR_WIDTH,
            height: CAR_HEIGHT,
            color: get_random_car_color(),
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 1.8,
            offset: [-1.7, 0.0].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 1.0,
            height: 1.6,
            offset: [0.8, 0.0].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 2.7,
            height: 0.15,
            offset: [-0.4, 0.85].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 2.7,
            height: 0.15,
            offset: [-0.4, -0.85].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 0.15,
            offset: [2.1, -0.7].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 0.15,
            offset: [2.1, 0.7].into(),
            color: BLACK,
            ..Default::default()
        });
    }

    let e = world
        .create_entity()
        .with(mr)
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(transport)
        //.with(Movable)
        .with(Selectable)
        .build();

    add_to_coworld(world, e);
    e
}

pub fn delete_transport_entity(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<PhysicsWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}

pub fn setup(world: &mut World) {
    load(world);
}
