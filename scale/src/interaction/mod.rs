pub use self::follow::*;
pub use self::inspected_aura::*;
pub use self::movable::*;
pub use self::roadbuild::*;
pub use self::selectable::*;

mod follow;
mod inspected_aura;
mod movable;
mod roadbuild;
mod selectable;

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
