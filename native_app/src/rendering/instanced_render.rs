use egregoria::rendering::assets::AssetRender;
use egregoria::Egregoria;
use geom::Transform;
use legion::query::*;
use wgpu_engine::{FrameContext, GfxContext, SpriteBatchBuilder, Texture};

pub struct InstancedRender {
    pub texs: Vec<SpriteBatchBuilder>,
}

impl InstancedRender {
    pub fn new(ctx: &mut GfxContext) -> Self {
        let car = Texture::from_path(ctx, "assets/car.png", Some("cartex"));
        let spr_car = SpriteBatchBuilder::new(car);

        let truck = Texture::from_path(ctx, "assets/truck.png", Some("trucktex"));
        let spr_truck = SpriteBatchBuilder::new(truck);

        let texs = vec![spr_car, spr_truck];
        InstancedRender { texs }
    }

    pub fn render(&mut self, goria: &mut Egregoria, fctx: &mut FrameContext) {
        for x in &mut self.texs {
            x.clear();
        }

        for (trans, ar) in <(&Transform, &AssetRender)>::query().iter(&goria.world) {
            if ar.hide {
                continue;
            }

            self.texs[ar.id.id as usize].push(
                trans.position(),
                trans.direction(),
                ar.z,
                ar.tint.into(),
                ar.scale,
            );
        }

        for x in &mut self.texs {
            if let Some(x) = x.build(fctx.gfx) {
                fctx.objs.push(Box::new(x));
            }
        }
    }
}
