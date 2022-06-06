use crate::engine_interaction::Selectable;
use crate::map_dynamic::{BuildingInfos, Itinerary};
use crate::pedestrians::Location;
use crate::physics::{Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject};
use crate::utils::rand_provider::RandProvider;
use crate::{Egregoria, SoulID};
use geom::Transform;
use geom::{Color, Vec3};
use hecs::Entity;
use imgui_inspect_derive::Inspect;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Inspect)]
pub struct Pedestrian {
    pub walking_speed: f32,
    pub walk_anim: f32,
}

const PED_SIZE: f32 = 0.5;

pub fn spawn_pedestrian(goria: &mut Egregoria, house: BuildingID) -> Option<Entity> {
    let _color = random_pedestrian_shirt_color(&mut *goria.write::<RandProvider>());

    let hpos = goria.map().buildings().get(house)?.door_pos;
    let p = Pedestrian::new(&mut *goria.write::<RandProvider>());
    let e = goria.world.spawn((
        Transform::new(hpos),
        Location::Building(house),
        p,
        Itinerary::NONE,
        Kinematics::default(),
        Selectable::new(3.0),
    ));

    goria.write::<BuildingInfos>().get_in(house, SoulID(e));
    Some(e)
}

pub fn put_pedestrian_in_coworld(coworld: &mut CollisionWorld, pos: Vec3) -> Collider {
    Collider(coworld.insert(
        pos.xy(),
        PhysicsObject {
            radius: PED_SIZE * 0.6,
            group: PhysicsGroup::Pedestrians,
            ..Default::default()
        },
    ))
}

impl Pedestrian {
    fn new(r: &mut RandProvider) -> Self {
        Self {
            walking_speed: (0.8 + r.next_f32() * 0.8),
            walk_anim: 0.0,
        }
    }
}

pub fn random_pedestrian_shirt_color(r: &mut RandProvider) -> Color {
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

    let r = r.next_f32() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}
