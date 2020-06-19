use specs::World;

mod data;
pub mod parking;
mod saveload;
pub mod systems;

pub use data::*;
pub use saveload::*;

pub fn setup(world: &mut World) {
    load(world);
}
