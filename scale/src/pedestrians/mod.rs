use specs::World;

pub mod data;
pub mod systems;

pub use data::*;
pub use systems::*;

pub fn setup(world: &mut World) {
    for _ in 0..3000 {
        spawn_pedestrian(world);
    }
}
