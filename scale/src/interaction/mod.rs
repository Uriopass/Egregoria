pub use self::follow::*;
pub use self::inspected_aura::*;
pub use self::movable::*;
pub use self::roadbuild::*;
pub use self::selectable::*;
pub use roadeditor::*;

mod follow;
mod inspected_aura;
mod movable;
mod roadbuild;
mod roadeditor;
mod selectable;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tool {
    Hand,
    Roadbuild,
    RoadEditor,
    Bulldozer,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
