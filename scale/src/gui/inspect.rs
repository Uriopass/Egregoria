use crate::interaction::Movable;
use crate::physics::physics_components::{Drag, Kinematics, Transform};
use cgmath::Vector2;
use imgui::im_str;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};

use crate::cars::car_data::{CarComponent, CarObjective};
use crate::rendering::meshrender_component::MeshRenderComponent;
use specs::prelude::*;
use std::marker::PhantomData;

pub struct ImCgVec2;
impl InspectRenderDefault<Vector2<f32>> for ImCgVec2 {
    // FIXME: Immutable inspecting of vec2
    fn render(data: &[&Vector2<f32>], _label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        let xs: Vec<&f32> = data.iter().map(|x| &x.x).collect();
        let ys: Vec<&f32> = data.iter().map(|x| &x.y).collect();
        <f32 as InspectRenderDefault<f32>>::render(xs.as_slice(), "x", ui, args);
        <f32 as InspectRenderDefault<f32>>::render(ys.as_slice(), "y", ui, args);
    }

    fn render_mut(
        data: &mut [&mut Vector2<f32>],
        label: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut conv = [x.x, x.y];
        ui.input_float2(&im_str!("{}", label), &mut conv).build();
        if conv[0] == x.x && conv[1] == x.y {
            return false;
        }
        x.x = conv[0];
        x.y = conv[1];
        true
    }
}

pub struct ImEntity;
impl InspectRenderDefault<Entity> for ImEntity {
    fn render(data: &[&Entity], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        ui.text(&im_str!("{:?} {}", *data[0], label));
    }

    fn render_mut(
        data: &mut [&mut Entity],
        label: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        ui.text(&im_str!("{:?} {}", *data[0], label));
        false
    }
}

pub struct ImVec<T> {
    _phantom: PhantomData<T>,
}
impl<T: InspectRenderDefault<T>> InspectRenderDefault<Vec<T>> for ImVec<T> {
    fn render(data: &[&Vec<T>], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut Vec<T>],
        label: &str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];

        if ui.collapsing_header(&im_str!("{}", label)).build() {
            for (i, x) in v.into_iter().enumerate() {
                let id = ui.push_id(i as i32);
                <T as InspectRenderDefault<T>>::render_mut(&mut [x], "", ui, args);
                id.pop(ui);
            }
        }

        false
    }
}

#[macro_export]
macro_rules! empty_struct_inspect_impl {
    ($x : ty) => {
        impl InspectRenderDefault<$x> for $x {
            fn render(_: &[&$x], _: &'static str, ui: &Ui, _: &InspectArgsDefault) {
                ui.text(std::stringify!($x))
            }

            fn render_mut(
                _: &mut [&mut $x],
                _: &'static str,
                ui: &Ui,
                _: &InspectArgsDefault,
            ) -> bool {
                ui.text(std::stringify!($x));
                false
            }
        }
    };
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
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
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
            let mut position = x.position();
            <ImCgVec2 as InspectRenderDefault<Vector2<f32>>>::render_mut(
                &mut [&mut position],
                "Pos",
                self.ui,
                &InspectArgsDefault::default(),
            );
            x.set_position(position);
        }

        self.inspect_component::<CarComponent>();
        self.inspect_component::<MeshRenderComponent>();
        self.inspect_component::<Kinematics>();
        self.inspect_component::<Drag>();
        self.inspect_component::<Movable>();
    }
}
