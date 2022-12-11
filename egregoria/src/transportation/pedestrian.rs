use crate::engine_interaction::Selectable;
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Itinerary};
use crate::physics::{Collider, CollisionWorld, PhysicsGroup, PhysicsObject, Speed};
use crate::transportation::Location;
use crate::utils::rand_provider::RandProvider;
use crate::utils::time::GameTime;
use crate::{Egregoria, SoulID};
use egui_inspect::Inspect;
use geom::{angle_lerpxy, Color, Transform, Vec3};
use hecs::Entity;
use hecs::World;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Inspect)]
pub struct Pedestrian {
    pub walking_speed: f32,
    pub walk_anim: f32,
}

const PED_SIZE: f32 = 0.5;

pub fn spawn_pedestrian(goria: &mut Egregoria, house: BuildingID) -> Option<Entity> {
    let _color = random_pedestrian_shirt_color(&mut goria.write::<RandProvider>());

    let hpos = goria.map().buildings().get(house)?.door_pos;
    let p = Pedestrian::new(&mut goria.write::<RandProvider>());
    let e = goria.world.spawn((
        Transform::new(hpos),
        Location::Building(house),
        p,
        Itinerary::NONE,
        Speed::default(),
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

#[profiling::function]
pub fn pedestrian_decision_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    world
        .query::<(&mut Itinerary, &mut Transform, &mut Speed, &mut Pedestrian)>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| batch.for_each(|(_, (a, b, c, d))| pedestrian_decision(ra, a, b, c, d)))
}

pub fn pedestrian_decision(
    time: &GameTime,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Speed,
    pedestrian: &mut Pedestrian,
) {
    let (desired_v, desired_dir) = calc_decision(pedestrian, trans, it);

    pedestrian.walk_anim += 7.0 * kin.speed * time.delta / pedestrian.walking_speed;
    pedestrian.walk_anim %= 2.0 * std::f32::consts::PI;
    physics(kin, trans, time, desired_v, desired_dir);
}

const PEDESTRIAN_ACC: f32 = 1.5;

pub fn physics(
    kin: &mut Speed,
    trans: &mut Transform,
    time: &GameTime,
    desired_velocity: f32,
    desired_dir: Vec3,
) {
    let diff = desired_velocity - kin.speed;
    let mag = diff.min(time.delta * PEDESTRIAN_ACC);
    if mag > 0.0 {
        kin.speed += mag;
    }
    const ANG_VEL: f32 = 1.0;
    trans.dir = angle_lerpxy(trans.dir, desired_dir, ANG_VEL * time.delta);
}

pub fn calc_decision(
    pedestrian: &mut Pedestrian,
    trans: &Transform,
    it: &Itinerary,
) -> (f32, Vec3) {
    let objective = match it.get_point() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let position = trans.position;

    let delta_pos: Vec3 = objective - position;
    let dir_to_pos = match delta_pos.try_normalize() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let desired_dir = dir_to_pos.normalize();
    (pedestrian.walking_speed, desired_dir)
}
