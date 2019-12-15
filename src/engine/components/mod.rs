use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, NullStorage, VecStorage};

pub use meshrender::*;
pub use physics::*;

mod meshrender;
mod physics;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Movable;
