use crate::map::Map;
use crate::map_dynamic::Itinerary;
use crate::physics::Speed;
use crate::utils::rand_provider::RandProvider;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::Simulation;
use crate::World;
use crate::{BirdEnt, BirdID};
use common::rand::rand2;
use egui_inspect::Inspect;
use geom::angle_lerpxy;
use geom::AABB;
use geom::{Transform, Vec3};
use serde::{Deserialize, Serialize};

pub fn spawn_bird(sim: &mut Simulation, home_pos: Vec3) -> Option<BirdID> {
    profiling::scope!("spawn_bird");

    // log::info!("added bird at {}", home_pos);

    let mob = BirdMob::new(&mut sim.write::<RandProvider>());

    let id = sim.world.insert(BirdEnt {
        trans: Transform::new(home_pos),
        bird_mob: mob,
        it: Itinerary::simple(vec![Vec3 {
            x: home_pos.x,
            y: home_pos.y,
            z: home_pos.z,
        }]),
        speed: Speed::default(),
    });

    Some(id)
}

#[derive(Serialize, Deserialize, Inspect)]
pub struct BirdMob {
    pub flying_speed: f32,
    pub fly_anim: f32,
    pub flock_id: u32,
}

pub const NUM_FLOCKS: u32 = 10;

impl BirdMob {
    pub(crate) fn new(r: &mut RandProvider) -> Self {
        Self {
            flying_speed: (10.0 + r.next_f32() * 0.4),
            fly_anim: 0.0,
            flock_id: (NUM_FLOCKS as f32 * r.next_f32()) as u32,
        }
    }
}

pub fn bird_decision_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("wildlife::animal_decision_system");
    let ra = &*resources.read::<GameTime>();
    let map = &*resources.read::<Map>();

    let aabb = map.terrain.bounds();
    let r = &mut RandProvider::new(ra.timestamp as u64);

    world.flocks.values().for_each(|flock| {
        let flock_physics: Vec<(Transform, Speed)> = flock
            .bird_ids
            .iter()
            .map(|bird_id| match world.birds.get_mut(*bird_id) {
                Some(bird_ent) => (bird_ent.trans, bird_ent.speed.clone()),
                None => unreachable!(),
            })
            .collect();

        flock
            .bird_ids
            .iter()
            .for_each(|bird_id| match world.birds.get_mut(*bird_id) {
                Some(bird_ent) => bird_decision(
                    ra,
                    &mut bird_ent.it,
                    &mut bird_ent.trans,
                    &mut bird_ent.speed,
                    &mut bird_ent.bird_mob,
                    aabb,
                    r,
                    &flock_physics,
                ),
                None => unreachable!(),
            })
    });

    // world.birds
    //     .values_mut()
    //     //.par_bridge()
    //     .for_each(|bird| bird_decision(ra, &mut bird.it, &mut bird.trans, &mut bird.speed, &mut bird.bird_mob, aabb, r))
}

const BIRD_WAIT_TIME: f64 = 100.0;

pub fn get_random_bird_pos(aabb: AABB, r1: f32, r2: f32, r3: f32) -> Vec3 {
    let AABB { ll, ur } = aabb;
    Vec3 {
        x: ll.x + (ur.x - ll.x) * r1,
        y: ll.y + (ur.y - ll.y) * r2,
        z: 2.0 + 10.0 * r3,
    }
}

pub fn get_random_pos_from_center(center: Vec3, radius: f32, r1: f32, r2: f32, r3: f32) -> Vec3 {
    Vec3 {
        x: center.x + radius * (r1 - 0.5),
        y: center.y + radius * (r2 - 0.5),
        z: 2.0 + 10.0 * r3,
    }
}

// amount of time per each flock itinerary
const FLOCK_IT_PERIOD: f32 = 20000.0;
// likelihood the bird will stray from the flock itinerary
const DISTRACTED_PROB: f32 = 0.5;

pub fn bird_decision(
    time: &GameTime,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Speed,
    bird_mob: &mut BirdMob,
    aabb: AABB,
    r: &mut RandProvider,
    flock_physics: &Vec<(Transform, Speed)>,
) {
    let (desired_v, desired_dir) = calc_decision(bird_mob, trans, it);

    bird_mob.fly_anim += 2.0 * kin.0 * time.realdelta / bird_mob.flying_speed;
    bird_mob.fly_anim %= 2.0 * std::f32::consts::PI;
    physics(kin, trans, time, desired_v, desired_dir);

    // a random nearby location to hang around
    let random_itinerary = Itinerary::simple(vec![get_random_pos_from_center(
        trans.position,
        100.0,
        r.next_f32(),
        r.next_f32(),
        r.next_f32(),
    )]);

    // every bird in the flock should have the same itinerary during the same flock period
    let flock_itinerary = Itinerary::simple(vec![get_random_bird_pos(
        aabb,
        rand2(
            bird_mob.flock_id as f32,
            (time.timestamp as f32 / FLOCK_IT_PERIOD).floor(),
        ),
        rand2(
            (time.timestamp as f32 / FLOCK_IT_PERIOD).floor(),
            bird_mob.flock_id as f32,
        ),
        r.next_f32(),
    )]);

    // choose a random new destination if the current one has been reached
    if it.has_ended(time.timestamp) {
        if it.is_none_or_wait() {
            *it = if r.next_f32() < DISTRACTED_PROB {
                random_itinerary
            } else {
                flock_itinerary
            };
        } else {
            // wait a bit before continuing
            *it = Itinerary::wait_until(time.timestamp + BIRD_WAIT_TIME);
        }
    }
}

const BIRD_ACC: f32 = 1.5;

pub fn physics(
    kin: &mut Speed,
    trans: &mut Transform,
    time: &GameTime,
    desired_velocity: f32,
    desired_dir: Vec3,
) {
    let diff = desired_velocity - kin.0;
    let mag = diff.min(time.realdelta * BIRD_ACC);
    if mag > 0.0 {
        kin.0 += mag;
    }
    const ANG_VEL: f32 = 1.0;
    // log::info!("{} {}", trans.dir, desired_dir);
    assert!(
        desired_dir.xy().mag() != 0.0,
        "desired_dir.xy() had 0 magnitude"
    );
    trans.dir = angle_lerpxy(trans.dir, desired_dir, ANG_VEL * time.realdelta);
}

pub fn calc_decision(bird_mob: &mut BirdMob, trans: &Transform, it: &Itinerary) -> (f32, Vec3) {
    assert!(
        trans.dir.xy().mag() != 0.0,
        "trans.dir.xy() had 0 magnitude"
    );
    let objective = match it.get_point() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let position = trans.position;

    let delta_pos: Vec3 = objective - position;
    let dir_to_pos = match delta_pos.xy().try_normalize() {
        Some(x) => x.z(trans.dir.z),
        None => return (0.0, trans.dir),
    };

    // log::info!("calc_decision trans.dir {}", trans.dir);
    // log::info!("calc_decision dir_to_pos {}", dir_to_pos);
    let desired_dir = dir_to_pos.normalize();
    assert!(
        desired_dir.xy().mag() != 0.0,
        "desired_dir.xy() had 0 magnitude in calc_decision"
    );
    (bird_mob.flying_speed, desired_dir)
}
