use crate::interaction::InspectedEntity;
use crate::physics::Transform;
use crate::rendering::meshrender_component::{MeshRender, StrokeCircleRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::DynamicSystemData;

#[derive(Default)]
pub struct InspectedAuraSystem {
    aura: Option<Entity>,
}

impl<'a> System<'a> for InspectedAuraSystem {
    type SystemData = (
        Read<'a, InspectedEntity>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, MeshRender>,
    );

    fn run(&mut self, (inspected, mut transforms, mut meshrenders): Self::SystemData) {
        meshrenders.get_mut(self.aura.unwrap()).unwrap().hide = true;

        if let Some(pos) = inspected
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
            StrokeCircleRender {
                offset: [0.0, 0.0].into(),
                color: Color::gray(0.7),
                radius: 3.0,
                thickness: 0.1,
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
