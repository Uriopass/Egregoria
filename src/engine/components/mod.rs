use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, NullStorage, VecStorage};

pub use physics::*;
pub use meshrender::*;

mod physics;
mod meshrender;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Movable;
