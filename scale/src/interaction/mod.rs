pub use self::bulldozer::*;
pub use self::follow::*;
pub use self::inspected_aura::*;
pub use self::movable::*;
pub use self::roadbuild::*;
pub use self::roadeditor::*;
pub use self::selectable::*;

mod bulldozer;
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

const Z_TOOL: f32 = 0.9;

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
