use crate::api::Location;
use crate::engine_interaction::{Movable, Selectable};
use crate::map_dynamic::{BuildingInfos, Itinerary};
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::Color;
use crate::Egregoria;
use geom::{vec2, Transform, Vec2};
use imgui_inspect_derive::*;
use legion::Entity;
use map_model::BuildingID;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PedestrianID(pub Entity);

#[derive(Serialize, Deserialize, Inspect)]
pub struct Pedestrian {
    pub walking_speed: f32,
    pub walk_anim: f32,
}

const PED_SIZE: f32 = 0.5;

pub fn spawn_pedestrian(goria: &mut Egregoria, house: BuildingID) -> PedestrianID {
    let color = random_pedestrian_shirt_color();

    let e = PedestrianID(goria.world.push((
        Transform::new(Vec2::ZERO),
        Location::Building(house),
        Pedestrian::default(),
        Itinerary::none(),
        Kinematics::from_mass(80.0),
        Movable,
        {
            MeshRender::empty(0.35)
                .add(RectRender {
                    // Arm 1
                    height: 0.14,
                    width: PED_SIZE * 0.4,
                    offset: vec2(0.0, PED_SIZE * 0.6),
                    color: Color::from_hex(0xFFCCA8), // Skin color (beige)
                })
                .add(RectRender {
                    // Arm 2
                    height: 0.14,
                    width: PED_SIZE * 0.4,
                    offset: vec2(0.0, -PED_SIZE * 0.6),
                    color: Color::from_hex(0xFFCCA8),
                })
                .add(RectRender {
                    // Body
                    height: PED_SIZE,
                    width: PED_SIZE * 0.5,
                    color,
                    ..Default::default()
                })
                .add(CircleRender {
                    // Head
                    radius: 0.16,
                    color: Color::BLACK,
                    ..Default::default()
                })
                .hidden()
                .build()
        },
        Selectable::new(0.5),
    )));

    goria.write::<BuildingInfos>().get_in(house, e);
    e
}

pub fn put_pedestrian_in_coworld(goria: &mut Egregoria, pos: Vec2) -> Collider {
    Collider(goria.write::<CollisionWorld>().insert(
        pos,
        PhysicsObject {
            radius: PED_SIZE * 0.6,
            group: PhysicsGroup::Pedestrians,
            ..Default::default()
        },
    ))
}

impl Default for Pedestrian {
    fn default() -> Self {
        Self {
            walking_speed: rand_distr::Normal::new(1.34f32, 0.26) // https://arxiv.org/pdf/cond-mat/9805244.pdf
                .unwrap() // Unwrap ok: it is a normal distribution
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
