use crate::map_dynamic::Itinerary;
use crate::physics::Speed;
use crate::transportation::bird_mob::BirdMob;
use crate::utils::rand_provider::RandProvider;
use crate::Simulation;
use crate::{BirdEnt, BirdID};
use geom::{Transform, Vec3};

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
