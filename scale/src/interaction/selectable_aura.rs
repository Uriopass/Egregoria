use crate::interaction::SelectedEntity;
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::DynamicSystemData;

#[derive(Default)]
pub struct SelectableAuraSystem {
    aura: Option<Entity>,
}

impl<'a> System<'a> for SelectableAuraSystem {
    type SystemData = (
        Read<'a, SelectedEntity>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, MeshRender>,
    );

    fn run(&mut self, (selected, mut transforms, mut meshrenders): Self::SystemData) {
        meshrenders.get_mut(self.aura.unwrap()).unwrap().hide = true;

        if let Some(pos) = selected
            .e
            .and_then(|sel| transforms.get(sel).map(|x| x.position()))
        {
            transforms
                .get_mut(self.aura.unwrap())
                .unwrap()
                .set_position(pos);
            meshrenders.get_mut(self.aura.unwrap()).unwrap().hide = false;
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        let mut mr = MeshRender::simple(
            CircleRender {
                offset: [0.0, 0.0].into(),
                color: Color::gray(0.7),
                radius: 3.0,
            },
            0.9,
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
