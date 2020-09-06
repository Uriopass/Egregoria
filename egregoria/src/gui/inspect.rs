use crate::interaction::IntersectionComponent;
use crate::interaction::{FollowEntity, Movable, MovedEvent};
use crate::map_dynamic::Itinerary;
use crate::pedestrians::PedestrianComponent;
use crate::physics::{Collider, Kinematics, Transform};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::VehicleComponent;
use geom::{vec2, Vec2};
use imgui::im_str;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use specs::shrev::EventChannel;
use specs::{Component, Entity, World, WorldExt};
use std::marker::PhantomData;

pub struct InspectDragf;
impl InspectRenderDefault<f32> for InspectDragf {
    fn render(data: &[&f32], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let cp = *data[0];
        ui.text(&im_str!("{} {}", cp, label));
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,

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
    fn render(data: &[&f64], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let cp = *data[0];
        ui.text(&im_str!("{} {}", cp, label));
    }

    fn render_mut(
        data: &mut [&mut f64],
        label: &'static str,

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

pub struct InspectVec2Immutable;
impl InspectRenderDefault<Vec2> for InspectVec2Immutable {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
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

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        Self::render(&[&*data[0]], label, ui, args);
        false
    }
}

pub struct InspectVec2Rotation;
impl InspectRenderDefault<Vec2> for InspectVec2Rotation {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        let ang = x.angle(vec2(0.0, 1.0));
        ui.text(&im_str!("{} {}", label, ang));
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,

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
    fn render(data: &[&Entity], label: &'static str, ui: &Ui, _args: &InspectArgsDefault) {
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

pub struct InspectVec<T> {
    _phantom: PhantomData<T>,
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Vec<T>> for InspectVec<T> {
    fn render(_data: &[&Vec<T>], _label: &'static str, _ui: &Ui, _args: &InspectArgsDefault) {
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

        let mut changed = false;
        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(&ui) {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <T as InspectRenderDefault<T>>::render_mut(&mut [x], "", ui, args);
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
            fn render(_: &[&$x], _: &'static str, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault) {
                ui.text(std::stringify!($x))
            }

            fn render_mut(_: &mut [&mut $x], _: &'static str, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault) -> bool {
                ui.text(std::stringify!($x));
                false
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! debug_inspect_impl {
    ($t: ty) => {
        impl imgui_inspect::InspectRenderDefault<$t> for $t {
            fn render(
                data: &[&$t],
                label: &'static str,
                ui: &imgui::Ui,
                _: &imgui_inspect::InspectArgsDefault,
            ) {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &data[0];
                ui.text(imgui::im_str!("{:?} {}", d, label));
            }

            fn render_mut(
                data: &mut [&mut $t],
                label: &'static str,
                ui: &imgui::Ui,
                _: &imgui_inspect::InspectArgsDefault,
            ) -> bool {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &data[0];
                ui.text(imgui::im_str!("{:?} {}", d, label));
                false
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! enum_inspect_impl {
    ($t: ty; $($x: pat),+) => {
        impl imgui_inspect::InspectRenderDefault<$t> for $t {
            fn render(data: &[&$t], label: &'static str, ui: &imgui::Ui, _: &imgui_inspect::InspectArgsDefault,
            ) {
                if data.len() != 1 {
                    unimplemented!()
                }
                let d = &data[0];
                let mut aha = "No match";
                $(
                    if let $x = d {
                        aha = stringify!($x);
                    }
                )+

                ui.text(imgui::im_str!("{} {}", &aha, label));
            }

            fn render_mut(
                data: &mut [&mut $t],
                label: &'static str,
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

pub struct InspectRenderer {
    pub entity: Entity,
}

/// Avoids Cloning by mutably aliasing the component inside the world
/// Unsound if the inspector also try to get the component using the world borrow
fn modify<T: Component>(world: &mut World, entity: Entity, f: impl FnOnce(&mut T) -> bool) -> bool {
    let mut storage = world.write_component::<T>();
    let c = unwrap_or!(storage.get_mut(entity), return false);
    f(c)
}

impl InspectRenderer {
    fn inspect_component<T: Component + InspectRenderDefault<T>>(
        &self,
        world: &mut World,
        ui: &Ui,
    ) -> bool {
        modify(world, self.entity, |x| -> bool {
            <T as InspectRenderDefault<T>>::render_mut(
                &mut [x],
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                ui,
                &InspectArgsDefault::default(),
            )
        })
    }

    pub fn render(&self, world: &mut World, ui: &Ui) -> bool {
        let mut event = None;
        let mut dirty = false;
        let entity = self.entity;
        dirty |= modify(world, entity, |x: &mut Transform| -> bool {
            let mut position = x.position();
            let mut direction = x.direction();
            let old_pos = position;
            let mut changed = <Vec2 as InspectRenderDefault<Vec2>>::render_mut(
                &mut [&mut position],
                "position",
                ui,
                &InspectArgsDefault::default(),
            );

            if changed {
                event = Some(MovedEvent {
                    entity,
                    new_pos: position,
                    delta_pos: position - old_pos,
                });
            }
            changed |= <InspectVec2Rotation as InspectRenderDefault<Vec2>>::render_mut(
                &mut [&mut direction],
                "direction",
                ui,
                &InspectArgsDefault::default(),
            );
            x.set_direction(direction);
            x.set_position(position);
            changed
        });

        if let Some(ev) = event {
            world
                .write_resource::<EventChannel<MovedEvent>>()
                .single_write(ev);
        }
        dirty |= self.inspect_component::<VehicleComponent>(world, ui);
        dirty |= self.inspect_component::<PedestrianComponent>(world, ui);
        dirty |= self.inspect_component::<AssetRender>(world, ui);
        dirty |= self.inspect_component::<MeshRender>(world, ui);
        dirty |= self.inspect_component::<Kinematics>(world, ui);
        dirty |= self.inspect_component::<Collider>(world, ui);
        dirty |= self.inspect_component::<Movable>(world, ui);
        dirty |= self.inspect_component::<IntersectionComponent>(world, ui);
        dirty |= self.inspect_component::<Itinerary>(world, ui);

        let follow = &mut world.write_resource::<FollowEntity>().0;
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
