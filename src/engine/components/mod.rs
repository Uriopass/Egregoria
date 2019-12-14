use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, NullStorage, VecStorage};

pub use location::*;
pub use meshrender::*;

mod location;
mod meshrender;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Movable;
