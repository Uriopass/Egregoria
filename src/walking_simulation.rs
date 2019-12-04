use crate::gsb::GSB;
use crate::shape_render::ShapeRenderer;
use cgmath::{InnerSpace, Point2, Vector2};
use ggez::input::mouse::MouseButton;
use ggez::Context;
use rand::prelude::SmallRng;
use rand::SeedableRng;

type Vector2f = Vector2<f32>;
use crate::EVACOLOR;
use ggez::graphics::Color;
use rayon::prelude::*;

#[allow(dead_code)]
struct Human {
    position: Vector2f,
    speed: Vector2f,
    direction: Vector2f,
    size: f32,
    objective: Vector2f,
    color: Color,
}

impl Human {
    fn calc_acceleration(&self, others: &Vec<Human>) -> Vector2f {
        let mut force: Vector2f = (self.objective - self.position) * 0.3;

        force -= self.speed;

        for h in others {
            let mut x: Vector2f = self.position - h.position;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            x *= h.size * h.size * 0.3 / x.magnitude2();
            force += x;
        }
        force
    }
}

pub struct HumanManager {
    humans: Vec<Human>,
    selected: Option<usize>,
    time: f32,
}

impl HumanManager {
    pub fn new_image(img: &Vec<EVACOLOR>) -> Self {
        const SCALE: i32 = 60;
        let mut humans: Vec<Human> = (0..10000)
            .zip(img)
            .filter(|(_, v)| **v != EVACOLOR::NONE)
            .map(|(i, v)| Human {
                position: [
                    (SCALE * 50) as f32 + (rand::random::<f32>() - 0.5) * 50000.,
                    (SCALE * 50) as f32 + (rand::random::<f32>() - 0.5) * 50000.,
                ]
                .into(),
                speed: [0., 0.].into(),
                direction: [1., 0.].into(),
                size: 15.
                    + rand::random::<f32>() * rand::random::<f32>() * rand::random::<f32>() * 90.,
                objective: {
                    let x = i % 100 * SCALE;
                    let y = i / 100 * SCALE;
                    [x as f32, (SCALE * 100 - y) as f32]
                }
                .into(),
                color: match &*v {
                    EVACOLOR::RED => ggez::graphics::WHITE,
                    EVACOLOR::WHITE => ggez::graphics::Color {
                        r: 0.9,
                        g: 0.,
                        b: 0.1,
                        a: 1.,
                    },
                    _ => ggez::graphics::WHITE,
                },
            })
            .collect();
        HumanManager {
            humans,
            selected: None,
            time: 0.,
        }
    }

    pub fn new(n_humans: i32) -> Self {
        let mut humans: Vec<Human> = (0..n_humans)
            .map(|_| Human {
                position: [
                    rand::random::<f32>() * 1000. - 500.,
                    rand::random::<f32>() * 1000. - 500.,
                ]
                .into(),
                speed: [0., 0.].into(),
                direction: [1., 0.].into(),
                size: 5.
                    + rand::random::<f32>() * rand::random::<f32>() * rand::random::<f32>() * 100.,
                objective: [
                    rand::random::<f32>() * 1000. - 500.,
                    rand::random::<f32>() * 1000. - 500.,
                ]
                .into(),
                color: ggez::graphics::WHITE,
            })
            .collect();

        /*humans.push(Human {
            position: [
                rand::random::<f32>() * 1000. - 500.,
                rand::random::<f32>() * 1000. - 500.,
            ]
            .into(),
            speed: [0., 0.].into(),
            direction: [1., 0.].into(),
            size: 1000.,
            objective: [rand::random::<f32>() * 1000., rand::random::<f32>() * 1000.].into(),
        });*/

        HumanManager {
            humans,
            selected: None,
            time: 0.,
        }
    }

    pub fn update(&mut self, ctx: &Context, gsb: &GSB, delta: f32) {
        self.time += delta;
        let accs: Vec<Vector2f> = self
            .humans
            .par_iter()
            .map(|h| h.calc_acceleration(&self.humans))
            .collect();

        if ggez::input::mouse::button_pressed(ctx, MouseButton::Left) {
            let click = gsb.unproject_mouse_click(ctx);
            let click: Vector2f = [click.x, click.y].into();
            match self.selected {
                Some(x) => {
                    self.humans[x].position = click;
                }
                None => {
                    let mut mindist = std::f32::MAX;
                    for (i, h) in self.humans.iter().enumerate() {
                        let dist = (click - h.position).magnitude2();
                        if dist < mindist {
                            mindist = dist;
                            self.selected = Some(i);
                        }
                    }
                }
            }
        } else {
            self.selected = None;
        }

        for (h, acc) in self.humans.iter_mut().zip(accs) {
            /*let mut nor = h.objective.normalize();
            let nor_perp: Vector2f = [-nor.y, nor.x].into();

            let sign = if (h.objective.magnitude() as i32) % 2 == 0 {
                -1.
            } else {
                1.
            };

            nor += sign * nor_perp * 0.0002 * delta * h.objective.magnitude();
            h.objective = nor * h.objective.magnitude();*/

            h.speed += acc * delta;
            h.position += h.speed * delta;
        }
    }

    pub fn draw(&self, sr: &mut ShapeRenderer) {
        for human in self.humans.iter() {
            sr.color = human.color;
            sr.color.a = (self.time / 8.).min(1.);
            sr.color.a *= sr.color.a;
            sr.draw_circle([human.position.x, human.position.y], human.size);
        }
    }
}
