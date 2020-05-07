pub use self::follow::*;
pub use self::movable::*;
pub use self::roadbuild::*;
pub use self::selectable::*;
pub use self::selectable_aura::*;

mod follow;
mod movable;
mod roadbuild;
mod selectable;
mod selectable_aura;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tool {
    Hand,
    Roadbuild,
    Bulldozer,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
