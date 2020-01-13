use cgmath::Vector2;
use engine::components::{Movable, Transform};
use engine::specs::prelude::*;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;

#[derive(Inspect)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl From<Vector2<f32>> for Vec2 {
    fn from(v: Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl Into<Vector2<f32>> for Vec2 {
    fn into(self) -> Vector2<f32> {
        [self.x, self.y].into()
    }
}

pub struct InspectRenderer<'a, 'b> {
    pub world: &'a mut World,
    pub entity: Entity,
    pub ui: &'b Ui<'b>,
}

impl<'a, 'b> InspectRenderer<'a, 'b> {
    fn inspect_component<T: Component + InspectRenderDefault<T>>(&self) {
        if let Some(x) = self.world.write_component::<T>().get_mut(self.entity) {
            <T as InspectRenderDefault<T>>::render_mut(
                &mut [x],
                "generated_label",
                self.ui,
                &InspectArgsDefault::default(),
            );
        }
    }

    pub fn render(self) {
        if let Some(x) = self
            .world
            .write_component::<Transform>()
            .get_mut(self.entity)
        {
            let mut position = Vec2::from(x.get_position());
            <Vec2 as InspectRenderDefault<Vec2>>::render_mut(
                &mut [&mut position],
                "generated_label",
                self.ui,
                &InspectArgsDefault::default(),
            );
            x.set_position(position.into());
        }
    }
}
