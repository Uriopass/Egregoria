use specs::World;

pub mod data;
pub mod systems;

pub use data::*;
pub use systems::*;

pub fn setup(_world: &mut World) {}
