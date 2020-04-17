use crate::interaction::Selectable;
use crate::map_model::{Itinerary, LaneKind, Map, Traversable, TraverseDirection, TraverseKind};
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::Color;
use cgmath::vec2;
use imgui_inspect_derive::*;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use specs::{Builder, World, WorldExt};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Serialize, Deserialize, Component, Inspect)]
pub struct PedestrianComponent {
    pub itinerary: Itinerary,
    pub walking_speed: f32,
    pub walk_anim: f32,
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
    let color = random_pedestrian_shirt_color();

    let is_tpose = rand::random::<bool>();

    world
        .create_entity()
        .with(Transform::new(pos))
        .with(PedestrianComponent {
            itinerary,
            ..Default::default()
        })
        .with(Kinematics::from_mass(80.0))
        .with({
            let mut lol = MeshRender::empty(3);
            if is_tpose {
                lol.add(RectRender {
                    height: 0.3,
                    width: 0.1,
                    offset: vec2(0.0, 0.35),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                })
                .add(RectRender {
                    height: 0.3,
                    width: 0.1,
                    offset: vec2(0.0, -0.35),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                })
                .add(CircleRender {
                    radius: 0.05,
                    offset: vec2(0.12, 0.06),
                    color: Color::from_hex(0xDDBB98),
                    ..Default::default()
                })
                .add(CircleRender {
                    radius: 0.05,
                    offset: vec2(0.12, -0.06),
                    color: Color::from_hex(0xDDBB98),
                    ..Default::default()
                })
                .add(RectRender {
                    height: 0.1,
                    width: 0.3,
                    offset: vec2(0.1, 0.0),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                })
                .add(CircleRender {
                    radius: 0.05,
                    offset: vec2(0.25, 0.0),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                });
            } else {
                lol.add(RectRender {
                    height: 0.12,
                    width: 0.15,
                    offset: vec2(0.0, 0.225),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                })
                .add(RectRender {
                    height: 0.12,
                    width: 0.15,
                    offset: vec2(0.0, -0.225),
                    color: Color::from_hex(0xFFCCA8),
                    ..Default::default()
                });
            }
            lol.add(RectRender {
                height: 0.4,
                width: 0.2,
                color,
                ..Default::default()
            })
            .add(CircleRender {
                radius: 0.1,
                color,
                offset: vec2(0.0, 0.2),
                ..Default::default()
            })
            .add(CircleRender {
                radius: 0.1,
                color,
                offset: vec2(0.0, -0.2),
                ..Default::default()
            })
            .add(CircleRender {
                radius: 0.125,
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
            walking_speed: rand_distr::Normal::new(1.34f32, 0.26) // https://arxiv.org/pdf/cond-mat/9805244.pdf
                .unwrap()
                .sample(&mut rand::thread_rng())
                .max(0.5),
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

    let r = rand::random::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}
