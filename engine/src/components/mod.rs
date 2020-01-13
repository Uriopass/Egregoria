use specs::{Component, NullStorage};

pub use meshrender::*;
pub use physics::*;

mod meshrender;
mod physics;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Movable;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Selectable;
