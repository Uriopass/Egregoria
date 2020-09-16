use crate::Action;
use egregoria::Egregoria;

mod home;
mod work;

pub use home::*;
pub use work::*;

pub trait Desire: Send + Sync {
    fn score(&self, goria: &Egregoria) -> f32;
    fn apply(&self, goria: &Egregoria) -> Action;
}
