use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::interaction::{FollowEntity, Movable, MovedEvent};
use crate::map_model::IntersectionComponent;
use crate::pedestrians::PedestrianComponent;
use crate::physics::{Collider, Kinematics, Transform};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::VehicleComponent;
use cgmath::InnerSpace;
use imgui::im_str;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use specs::shrev::EventChannel;
use specs::{Component, Entity, World, WorldExt};
use std::marker::PhantomData;

pub struct InspectDragf;
impl InspectRenderDefault<f32> for InspectDragf {
    fn render(
        data: &[&f32],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0];
        ui.drag_float(&im_str!("{}", label), &mut cp)
            .speed(args.step.unwrap_or(0.1))
            .build();
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        ui.drag_float(&im_str!("{}", label), data[0])
            .speed(args.step.unwrap_or(0.1))
            .build()
    }
}

impl InspectRenderDefault<f64> for InspectDragf {
    fn render(
        data: &[&f64],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        ui.drag_float(&im_str!("{}", label), &mut cp)
            .speed(args.step.unwrap_or(0.1))
            .build();
    }

    fn render_mut(
        data: &mut [&mut f64],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        let changed = ui
            .drag_float(&im_str!("{}", label), &mut cp)
            .speed(args.step.unwrap_or(0.1))
            .build();
        *data[0] = cp as f64;
        changed
    }
}

pub struct InspectVec2;
impl InspectRenderDefault<Vec2> for InspectVec2 {
    fn render(data: &[&Vec2], label: &'static str, _: &mut World, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        imgui::InputFloat2::new(ui, &im_str!("{}", label), &mut [x.x, x.y])
            .always_insert_mode(false)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut conv = [x.x, x.y];
        let changed = ui
            .drag_float2(&im_str!("{}", label), &mut conv)
            .speed(args.step.unwrap_or(0.1))
            .build();
        x.x = conv[0];
        x.y = conv[1];
        changed
    }
}

pub struct InspectVec2Rotation;
impl InspectRenderDefault<Vec2> for InspectVec2Rotation {
    fn render(data: &[&Vec2], label: &'static str, _: &mut World, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        let ang = x.angle(vec2!(0.0, 1.0));
        ui.text(&im_str!("{} {}", label, ang.0));
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut ang = f32::atan2(x.y, x.x);

        let changed = ui
            .drag_float(&im_str!("{}", label), &mut ang)
            .speed(-args.step.unwrap_or(0.1))
            .build();
        x.x = ang.cos();
        x.y = ang.sin();
        changed
    }
}

pub struct ImEntity;
impl InspectRenderDefault<Entity> for ImEntity {
    fn render(
        data: &[&Entity],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() != 1 {
            unimplemented!();
        }
        ui.text(&im_str!("{:?} {}", *data[0], label));
    }

    fn render_mut(
        data: &mut [&mut Entity],
        label: &'static str,
        _: &mut World,
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

pub struct InspectVec<T> {
    _phantom: PhantomData<T>,
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Vec<T>> for InspectVec<T> {
    fn render(
        _data: &[&Vec<T>],
        _label: &'static str,
        _: &mut World,
        _ui: &Ui,
        _args: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut Vec<T>],
        label: &str,
        w: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];

        let mut changed = false;
        if ui.collapsing_header(&im_str!("{}", label)).build() {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <T as InspectRenderDefault<T>>::render_mut(&mut [x], "", w, ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}

impl InspectRenderDefault<PolyLine> for PolyLine {
    fn render(
        _data: &[&PolyLine],
        _label: &'static str,
        _: &mut World,
        _ui: &Ui,
        _args: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut PolyLine],
        label: &str,
        w: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];
        let mut changed = false;

        if ui.collapsing_header(&im_str!("{}", label)).build() {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <InspectVec2 as InspectRenderDefault<Vec2>>::render_mut(
                    &mut [x],
                    "",
                    w,
                    ui,
                    args,
                );
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}

#[rustfmt::skip]
macro_rules! empty_inspect_impl {
    ($x : ty) => {
        impl imgui_inspect::InspectRenderDefault<$x> for $x {
            fn render(_: &[&$x], _: &'static str, _: &mut specs::World, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault) {
                ui.text(std::stringify!($x))
            }

            fn render_mut(_: &mut [&mut $x], _: &'static str, _: &mut specs::World, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault) -> bool {
                ui.text(std::stringify!($x));
                false
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! enum_inspect_impl {
    ($t: ty; $($x: pat),+) => {
        impl imgui_inspect::InspectRenderDefault<$t> for $t {
            fn render(_: &[&$t], _: &'static str, _: &mut specs::World, _: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault,
            ) {
                unimplemented!()
            }

            fn render_mut(
                data: &mut [&mut $t],
                label: &'static str,
                _: &mut specs::World,
                ui: &imgui::Ui,
                _: &imgui_inspect::InspectArgsDefault,
            ) -> bool {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &mut data[0];
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(imgui::im_str!("{} {}", &aha, label));
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

fn clone_and_modify<T: Component + Clone>(
    world: &mut World,
    entity: Entity,
    f: impl FnOnce(&mut World, T) -> Option<T>,
) -> bool {
    let c = world.write_component::<T>().get_mut(entity).cloned();

    if let Some(x) = c {
        let m = f(world, x);
        if let Some(v) = m {
            *world.write_component::<T>().get_mut(entity).unwrap() = v;
            true
        } else {
            false
        }
    } else {
        false
    }
}

impl<'a, 'b> InspectRenderer<'a, 'b> {
    fn inspect_component<T: Component + Clone + InspectRenderDefault<T>>(&mut self) -> bool {
        let ui = self.ui;
        clone_and_modify(self.world, self.entity, |world, mut x| {
            if <T as InspectRenderDefault<T>>::render_mut(
                &mut [&mut x],
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                world,
                ui,
                &InspectArgsDefault::default(),
            ) {
                Some(x)
            } else {
                None
            }
        })
    }

    pub fn render(mut self) -> bool {
        let ui = self.ui;
        let mut event = None;
        let mut dirty = false;
        dirty |= clone_and_modify(self.world, self.entity, |world, mut x: Transform| {
            let mut position = x.position();
            let mut direction = x.direction();
            let mut changed = <InspectVec2 as InspectRenderDefault<Vec2>>::render_mut(
                &mut [&mut position],
                "position",
                world,
                ui,
                &InspectArgsDefault::default(),
            );

            if changed {
                event = Some(position);
            }
            changed |= <InspectVec2Rotation as InspectRenderDefault<Vec2>>::render_mut(
                &mut [&mut direction],
                "direction",
                world,
                ui,
                &InspectArgsDefault::default(),
            );
            x.set_direction(direction);
            x.set_position(position);
            if changed {
                Some(x)
            } else {
                None
            }
        });

        if let Some(new_pos) = event {
            self.world
                .write_resource::<EventChannel<MovedEvent>>()
                .single_write(MovedEvent {
                    entity: self.entity,
                    new_pos,
                });
        }
        dirty |= self.inspect_component::<VehicleComponent>();
        dirty |= self.inspect_component::<PedestrianComponent>();
        dirty |= self.inspect_component::<AssetRender>();
        dirty |= self.inspect_component::<MeshRender>();
        dirty |= self.inspect_component::<Kinematics>();
        dirty |= self.inspect_component::<Collider>();
        dirty |= self.inspect_component::<Movable>();
        dirty |= self.inspect_component::<IntersectionComponent>();

        let follow = &mut self.world.write_resource::<FollowEntity>().0;
        if follow.is_none() {
            if ui.small_button(im_str!("Follow")) {
                follow.replace(self.entity);
            }
        } else if ui.small_button(im_str!("Unfollow")) {
            follow.take();
        }
        if dirty {
            ui.text("dirty");
        }
        dirty
    }
}
