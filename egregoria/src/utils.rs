use crate::physics::{Collider, CollisionWorld};
use specs::{Entity, World, WorldExt};
macro_rules! unwrap_or {
    ($e: expr, $t: expr) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

pub fn rand_world<T>(world: &mut specs::World) -> T
where
    rand_distr::Standard: rand_distr::Distribution<T>,
{
    world.write_resource::<crate::RandProvider>().random()
}

pub trait Restrict {
    fn restrict(self, min: Self, max: Self) -> Self;
}

impl<T: PartialOrd> Restrict for T {
    fn restrict(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

pub fn delete_entity(world: &mut World, e: Entity) {
    if let Some(&Collider(handle)) = world.read_component::<Collider>().get(e) {
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    match world.delete_entity(e) {
        Ok(()) => {}
        Err(_) => log::warn!("Trying to remove nonexistent entity"),
    }
}
