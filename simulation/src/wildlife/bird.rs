use crate::map::Map;
use crate::map_dynamic::Itinerary;
use crate::physics::Speed;
use crate::utils::rand_provider::RandProvider;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::Simulation;
use crate::World;
use crate::{BirdEnt, BirdID};
use common::rand::rand3;
use egui_inspect::Inspect;
use geom::angle_lerpxy;
use geom::{Transform, Vec3};
use serde::{Deserialize, Serialize};

pub fn spawn_bird(sim: &mut Simulation, home_pos: Vec3) -> Option<BirdID> {
    profiling::scope!("spawn_bird");

    log::info!("added bird at {}", home_pos);

    let a = BirdMob::new(&mut sim.write::<RandProvider>());

    let id = sim.world.insert(BirdEnt {
        trans: Transform::new(home_pos),
        bird_mob: a,
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
}

impl BirdMob {
    pub(crate) fn new(r: &mut RandProvider) -> Self {
        Self {
            flying_speed: (10.0 + r.next_f32() * 0.4),
            fly_anim: 0.0,
        }
    }
}

pub fn bird_decision_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("transportation::animal_decision_system");
    let ra = &*resources.read();
    let map = &*resources.read::<Map>();
    let next_dests: Vec<Vec3> = map
        .buildings()
        .values()
        .map(|building| building.door_pos.up(-building.height))
        // .chain(
        //     map.terrain
        //         .trees
        //         .objects()
        //         .map(|tree| vec3(tree.1.pos.x, tree.1.pos.y, tree.1.size)),
        // )
        .collect();
    world.birds
        .values_mut()
        //.par_bridge()
        .for_each(|human| bird_decision(ra, &mut human.it, &mut human.trans, &mut human.speed, &mut human.bird_mob, &next_dests))
}

const BIRD_WAIT_TIME: f64 = 100.0;

pub fn bird_decision(
    time: &GameTime,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Speed,
    bird_mob: &mut BirdMob,
    next_dests: &Vec<Vec3>,
) {
    let (desired_v, desired_dir) = calc_decision(bird_mob, trans, it);

    bird_mob.fly_anim += 2.0 * kin.0 * time.realdelta / bird_mob.flying_speed;
    bird_mob.fly_anim %= 2.0 * std::f32::consts::PI;
    physics(kin, trans, time, desired_v, desired_dir);

    let get_new_itinerary = || {
        let n2 = rand3(trans.position.y, trans.position.x, time.timestamp as f32);
        let n1 = rand3(trans.position.x, trans.position.y, time.timestamp as f32);
        if n1 > 0.5 {
            Itinerary::simple(vec![next_dests[(next_dests.len() as f32 * n2) as usize]])
        } else {
            Itinerary::wait_until(time.timestamp + BIRD_WAIT_TIME)
        }
    };

    // choose a random new destination if the current one has been reached
    if it.has_ended(time.timestamp) {
        *it = get_new_itinerary();
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
    trans.dir = angle_lerpxy(trans.dir, desired_dir, ANG_VEL * time.realdelta);
}

pub fn calc_decision(bird_mob: &mut BirdMob, trans: &Transform, it: &Itinerary) -> (f32, Vec3) {
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
    (bird_mob.flying_speed, desired_dir)
}
