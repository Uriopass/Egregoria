use crate::empty_struct_inspect_impl;
use crate::engine_interaction::{MouseButton, MouseInfo};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRenderComponent, MeshRenderEnum};
use crate::rendering::Color;
use cgmath::InnerSpace;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use specs::prelude::*;
use specs::shred::DynamicSystemData;
use specs::Component;
use std::f32;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Selectable;
empty_struct_inspect_impl!(Selectable);

#[derive(Default, Clone, Copy)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Default)]
pub struct SelectableSystem {
    aura: Option<Entity>,
}

impl<'a> System<'a> for SelectableSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, MouseInfo>,
        Write<'a, SelectedEntity>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Selectable>,
        WriteStorage<'a, MeshRenderComponent>,
    );

    fn run(
        &mut self,
        (entities, mouse, mut selected, mut transforms, selectables, mut meshrenders): Self::SystemData,
    ) {
        if mouse.just_pressed.contains(&MouseButton::Left) {
            let mut min_dist = f32::MAX;
            let mut closest = None;
            for (entity, trans, _) in (&entities, &transforms, &selectables).join() {
                let dist: f32 = (trans.position() - mouse.unprojected).magnitude2();
                if dist <= min_dist {
                    closest = Some(entity);
                    min_dist = dist;
                }
            }
            *selected = SelectedEntity(closest);
            println!("Selected new entity");

            meshrenders
                .get_mut(self.aura.unwrap())
                .map(|mr| match &mut mr.orders[0] {
                    MeshRenderEnum::Circle(x) => {
                        x.radius = if closest.is_some() { 3.0 } else { 0.0 };
                    }
                    _ => (),
                });
        }

        if let Some(sel) = selected.0 {
            let pos = transforms.get(sel).unwrap().position();

            transforms
                .get_mut(self.aura.unwrap())
                .unwrap()
                .set_position(pos)
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        self.aura = Some(
            world
                .create_entity()
                .with(Transform::zero())
                .with(MeshRenderComponent::from(CircleRender {
                    offset: [0.0, 0.0].into(),
                    filled: false,
                    color: Color::gray(0.7),
                    radius: 0.0,
                }))
                .build(),
        );
    }
}
