use crate::engine_interaction::KeyCode;
use crate::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::Color;
use cgmath::InnerSpace;
use imgui_inspect_derive::*;
use specs::prelude::*;
use specs::shred::DynamicSystemData;
use specs::Component;
use std::f32;

#[derive(Component, Default, Inspect)]
#[storage(NullStorage)]
pub struct Selectable;

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
        Read<'a, KeyboardInfo>,
        Write<'a, SelectedEntity>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Selectable>,
        WriteStorage<'a, MeshRender>,
    );

    fn run(
        &mut self,
        (entities, mouse, kbinfo, mut selected, mut transforms, selectables, mut meshrenders): Self::SystemData,
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

            meshrenders.get_mut(self.aura.unwrap()).unwrap().hide = false;
        }

        if kbinfo.just_pressed.contains(&KeyCode::Escape) {
            *selected = SelectedEntity(None);
        }

        if let Some(sel) = selected.0 {
            if let Some(pos) = transforms.get(sel).map(|x| x.position()) {
                transforms
                    .get_mut(self.aura.unwrap())
                    .unwrap()
                    .set_position(pos)
            } else {
                *selected = SelectedEntity(None);
            }
        } else {
            meshrenders.get_mut(self.aura.unwrap()).unwrap().hide = true;
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        let mut mr = MeshRender::simple(
            CircleRender {
                offset: [0.0, 0.0].into(),
                filled: false,
                color: Color::gray(0.7),
                radius: 3.0,
            },
            9,
        );
        mr.hide = true;
        self.aura = Some(
            world
                .create_entity()
                .with(Transform::zero())
                .with(mr)
                .build(),
        );
    }
}
