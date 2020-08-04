use specs::World;

mod data;
mod saveload;
pub mod systems;

pub use data::*;
pub use saveload::*;

pub fn setup(world: &mut World) {
    load(world);
}
