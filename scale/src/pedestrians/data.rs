use crate::interaction::{Movable, Selectable};
use crate::map_model::{Itinerary, LaneKind, Map, Traversable, TraverseDirection, TraverseKind};
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::Color;
use crate::utils::rand_normal;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Builder, Entity, World, WorldExt};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Serialize, Deserialize, Component, Inspect)]
pub struct PedestrianComponent {
    pub itinerary: Itinerary,
    pub walking_speed: f32,
    pub walk_anim: f32,
}

pub fn delete_pedestrian(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}

pub fn spawn_pedestrian(world: &mut World) {
    let map = world.read_resource::<Map>();

    let lane = unwrap_or!(map.get_random_lane(LaneKind::Walking), return);

    let pos = if let [a, b, ..] = lane.points.as_slice() {
        a + (b - a) * crate::utils::rand_det()
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

    let size = 0.5;

    let h = world.get_mut::<CollisionWorld>().unwrap().insert(
        pos,
        PhysicsObject {
            radius: size * 0.6,
            group: PhysicsGroup::Pedestrians,
            ..Default::default()
        },
    );
    let color = random_pedestrian_shirt_color();

    world
        .create_entity()
        .with(Transform::new(pos))
        .with(PedestrianComponent {
            itinerary,
            ..Default::default()
        })
        .with(Kinematics::from_mass(80.0))
        .with(Movable)
        .with({
            MeshRender::empty(0.35)
                .add(RectRender {
                    height: 0.14,
                    width: size * 0.4,
                    offset: vec2!(0.0, size * 0.6),
                    color: Color::from_hex(0xFFCCA8),
                })
                .add(RectRender {
                    height: 0.14,
                    width: size * 0.4,
                    offset: vec2!(0.0, -size * 0.6),
                    color: Color::from_hex(0xFFCCA8),
                })
                .add(RectRender {
                    height: size,
                    width: size * 0.5,
                    color,
                    ..Default::default()
                })
                .add(CircleRender {
                    radius: size * 0.25,
                    color,
                    offset: vec2!(0.0, size * 0.5),
                })
                .add(CircleRender {
                    radius: size * 0.25,
                    color,
                    offset: vec2!(0.0, -size * 0.5),
                })
                .add(CircleRender {
                    radius: 0.16,
                    color: Color::BLACK,
                    ..Default::default()
                })
                .build()
        })
        .with(Collider(h))
        .with(Selectable::new(0.5))
        .build();
}

impl Default for PedestrianComponent {
    fn default() -> Self {
        Self {
            itinerary: Itinerary::default(),
            walking_speed: rand_normal(1.34f32, 0.26).max(0.5), // https://arxiv.org/pdf/cond-mat/9805244.pdf
            walk_anim: 0.0,
        }
    }
}

pub fn random_pedestrian_shirt_color() -> Color {
    let car_colors: [(Color, f32); 7] = [
        (Color::from_hex(0xff_ff_ff), 0.1),  // White
        (Color::from_hex(0x66_66_66), 0.1),  // Gray
        (Color::from_hex(0x1a_3c_70), 0.1),  // Blue
        (Color::from_hex(0xd8_22_00), 0.1),  // Red
        (Color::from_hex(0x7c_4b_24), 0.04), // Brown
        (Color::from_hex(0xd4_c6_78), 0.04), // Gold
        (Color::from_hex(0x72_cb_19), 0.02), // Green
    ];

    let total: f32 = car_colors.iter().map(|x| x.1).sum();

    let r = crate::utils::rand_det::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}
