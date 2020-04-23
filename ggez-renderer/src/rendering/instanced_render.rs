use crate::rendering::render_context::RenderContext;
use cgmath::{InnerSpace, Vector2};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::{DrawParam, Drawable, FilterMode, Image};
use ggez::Context;
use scale::physics::Transform;
use scale::rendering::assets::AssetRender;
use scale::specs::{Join, World, WorldExt};

pub struct InstancedRender {
    pub texs: Vec<SpriteBatch>,
    pub scales: Vec<f32>,
    pub offsets: Vec<Vector2<f32>>,
}

impl InstancedRender {
    pub fn new(ctx: &mut Context) -> Self {
        let mut scales = vec![];
        let mut offsets = vec![];

        let car = Image::new(ctx, "/car.png").unwrap();
        scales.push(1.0 / (car.width().max(car.height()) as f32));
        offsets.push(Vector2 {
            x: 0.5 * car.width() as f32,
            y: 0.5 * car.height() as f32,
        });
        let mut spr_car = SpriteBatch::new(car);
        spr_car.set_filter(FilterMode::Linear);

        let texs = vec![spr_car];
        InstancedRender {
            texs,
            scales,
            offsets,
        }
    }

    pub fn render(&mut self, world: &mut World, rc: &mut RenderContext) {
        let transforms = world.read_component::<Transform>();
        let ass_render = world.write_component::<AssetRender>();

        for x in &mut self.texs {
            x.clear();
        }

        for (trans, ar) in (&transforms, &ass_render).join() {
            if ar.hide {
                continue;
            }
            let scale = ar.scale * self.scales[ar.id.id as usize];
            let off = self.offsets[ar.id.id as usize];
            let dp = DrawParam {
                dest: [trans.project(-off * scale).x, trans.project(-off * scale).y].into(),
                rotation: Vector2::<f32>::unit_x().angle(trans.direction()).0,
                scale: [scale, scale].into(),
                offset: [0.0, 0.0].into(),
                color: ggez::graphics::Color {
                    r: ar.tint.r,
                    g: ar.tint.g,
                    b: ar.tint.b,
                    a: ar.tint.a,
                },
                ..Default::default()
            };
            self.texs[ar.id.id as usize].add(dp);
        }

        for x in &mut self.texs {
            x.draw(rc.ctx, DrawParam::default()).unwrap()
        }
    }
}
