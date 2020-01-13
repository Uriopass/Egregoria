use engine::components::Transform;
use engine::specs::prelude::*;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};

macro_rules! inspect_macro {
    [
        $(
            $x: ty
        ),*
    ] => {
        pub fn render_inspect(world: &mut World, ui: &Ui, entity: Entity) {
           $(if let Some(x) = world.write_component::<$x>().get_mut(entity) {
                 <$x as InspectRenderDefault<$x>>::render_mut(
                    &mut [x],
                    "generated_label",
                    ui,
                    &InspectArgsDefault::default(),
                );
            });*
        }
    }
}

inspect_macro![Transform];
