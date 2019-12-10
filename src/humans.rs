use cgmath::{InnerSpace, Vector2};
use ggez::graphics::WHITE;

use specs::prelude::*;
use specs::Component;

use crate::engine::components::{CircleRender, Collider, LineRender, Movable, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;

use nalgebra as na;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Ball, ShapeHandle};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Human {
    size: f32,
    objective: Vector2<f32>,
}

impl Human {
    fn calc_acceleration(
        &self,
        position: &Position,
        speed: &Velocity,
        others: &[(&Position, &Human)],
    ) -> Vector2<f32> {
        if (self.objective - position.0).magnitude2() < 1. {
            return [0.0, 0.0].into();
        }
        let mut force: Vector2<f32> = (self.objective - position.0).normalize() * 20.;

        force -= speed.0;

        for (p, h) in others {
            let mut x: Vector2<f32> = position.0 - p.0;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            let d = x.magnitude();
            x *= (h.size * self.size * 0.1) / (d * d);
            force += x;
        }
        force
    }
}

pub struct HumanUpdate;

impl<'a> System<'a> for HumanUpdate {
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Human>,
    );

    fn run(&mut self, (delta, pos, mut vel, humans): Self::SystemData) {
        let delta = delta.0;

        let xx: Vec<(&Position, &Human)> = (&pos, &humans).join().collect();

        (&pos, &mut vel, &humans).par_join().for_each(|(p, v, h)| {
            let acc = h.calc_acceleration(&p, &v, &xx);
            v.0 += acc * delta * 2.;
        })
    }
}

pub fn setup(world: &mut World, coworld: &mut PhysicsWorld) {
    let mut last: Option<Entity> = None;
    let gr = CollisionGroups::new();
    const SCALE: f32 = 500.;
    for _ in 0..100 {
        let size = 10.;

        let x: f32 = if rand::random() {
            rand::random::<f32>() * SCALE
        } else {
            SCALE * 5. + rand::random::<f32>() * SCALE
        };
        let y: f32 = rand::random::<f32>() * SCALE;

        let mut y = world
            .create_entity()
            .with(CircleRender {
                radius: size,
                color: WHITE,
            })
            .with(Position([x, y].into()))
            .with(Velocity([0.0, 1.0].into()))
            .with(Human {
                size: size * 2.,
                objective: [SCALE * 5. - x, y].into(),
            })
            .with(Movable);
        if let Some(x) = last {
            y = y.with(LineRender {
                color: ggez::graphics::WHITE,
                to: x,
            });
        }

        let e = y.build();
        let shape = Ball::new(size);

        let (h, _) = coworld.add(
            na::Isometry2::new(na::Vector2::new(0., 0.), na::zero()),
            ShapeHandle::new(shape),
            gr,
            GeometricQueryType::Contacts(0.0, 0.0),
            e,
        );

        let mut x = world.write_component::<Collider>();
        x.insert(e, Collider { 0: h }).unwrap();

        last = Some(e);
    }
}
