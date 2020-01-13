use engine::components::Transform;
use engine::specs::prelude::*;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectArgsStruct, InspectRenderDefault};

pub fn render_inspect(world: &mut World, ui: &Ui, entity: Entity) {
    if let Some(x) = world.write_component::<Transform>().get_mut(entity) {
        <Transform as InspectRenderDefault<Transform>>::render_mut(
            &mut [x],
            "inspect_transform",
            ui,
            &InspectArgsDefault::default(),
        );
    }
}
