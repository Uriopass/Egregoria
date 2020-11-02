use common::inspect::InspectedEntity;
use egregoria::Egregoria;
use map_model::LotKind;
use movable::MovableSystem;
use roadbuild::RoadBuildResource;
use roadeditor::RoadEditorResource;

mod bulldozer;
mod follow;
mod inspect;
mod inspected_aura;
mod lotbrush;
mod movable;
mod roadbuild;
mod roadeditor;
mod selectable;
mod topgui;
pub mod windows;

pub use follow::FollowEntity;

pub use inspect::*;
pub use topgui::*;

pub fn setup_gui(goria: &mut Egregoria) {
    goria
        .schedule
        .add_system(selectable::selectable_select_system())
        .add_system(selectable::selectable_cleanup_system())
        .add_system(roadbuild::roadbuild_system())
        .add_system(roadeditor::roadeditor_system())
        .add_system(bulldozer::bulldozer_system())
        .add_system(lotbrush::lotbrush_system())
        .add_system(inspected_aura::inspected_aura_system())
        .add_system(movable::movable_system(MovableSystem::default()));

    goria.insert(InspectedEntity::default());
    goria.insert(FollowEntity::default());
    goria.insert(Tool::default());

    goria.insert(RoadBuildResource::default());
    goria.insert(RoadEditorResource::default());
}

#[derive(Copy, Clone)]
pub enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush(LotKind),
}

const Z_TOOL: f32 = 0.9;

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
