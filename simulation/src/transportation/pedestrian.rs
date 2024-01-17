use crate::map_dynamic::Itinerary;
use crate::transportation::{
    Speed, TransportGrid, TransportState, TransportationGroup, Transporter,
};
use crate::utils::rand_provider::RandProvider;
use crate::utils::resources::Resources;
use crate::World;
use egui_inspect::Inspect;
use geom::{angle_lerpxy, Color, Transform, Vec3};
use prototypes::DELTA;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Inspect)]
pub struct Pedestrian {
    pub walking_speed: f32,
    pub walk_anim: f32,
}

const PED_SIZE: f32 = 0.5;

pub fn put_pedestrian_in_transport_grid(
    transport_grid: &mut TransportGrid,
    pos: Vec3,
) -> Transporter {
    Transporter(transport_grid.insert(
        pos.xy(),
        TransportState {
            radius: PED_SIZE * 0.6,
            group: TransportationGroup::Pedestrians,
            ..Default::default()
        },
    ))
}

impl Pedestrian {
    pub(crate) fn new(r: &mut RandProvider) -> Self {
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

pub fn pedestrian_decision_system(world: &mut World, _resources: &mut Resources) {
    profiling::scope!("transportation::pedestrian_decision_system");
    world.humans
        .values_mut()
        //.par_bridge()
        .for_each(|human| pedestrian_decision(&mut human.it, &mut human.trans, &mut human.speed, &mut human.pedestrian))
}

pub fn pedestrian_decision(
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Speed,
    pedestrian: &mut Pedestrian,
) {
    let (desired_v, desired_dir) = calc_decision(pedestrian, trans, it);

    pedestrian.walk_anim += 7.0 * kin.0 * DELTA / pedestrian.walking_speed;
    pedestrian.walk_anim %= 2.0 * std::f32::consts::PI;
    physics(kin, trans, desired_v, desired_dir);
}

const PEDESTRIAN_ACC: f32 = 1.5;

pub fn physics(kin: &mut Speed, trans: &mut Transform, desired_velocity: f32, desired_dir: Vec3) {
    let diff = desired_velocity - kin.0;
    let mag = diff.min(DELTA * PEDESTRIAN_ACC);
    if mag > 0.0 {
        kin.0 += mag;
    }
    const ANG_VEL: f32 = 1.0;
    trans.dir = angle_lerpxy(trans.dir, desired_dir, ANG_VEL * DELTA);
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

    let position = trans.pos;

    let delta_pos: Vec3 = objective - position;
    let dir_to_pos = match delta_pos.try_normalize() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let desired_dir = dir_to_pos.normalize();
    (pedestrian.walking_speed, desired_dir)
}
