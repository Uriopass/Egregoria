use egregoria::api::{Action, Router};
use egregoria::Egregoria;

mod home;
mod work;

pub use home::*;
pub use work::*;

pub trait Desire<T>: Send + Sync {
    fn score(&self, goria: &Egregoria, soul: &T) -> f32;
    fn apply(&mut self, goria: &Egregoria, soul: &mut T) -> Action;
}

pub trait Routed {
    fn router_mut(&mut self) -> &mut Router;
}
