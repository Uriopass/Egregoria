use crate::engine::{FrameContext, GfxContext, InstanceRaw, SpriteBatchBuilder, Texture};
use egregoria::legion::query::*;
use egregoria::rendering::assets::AssetRender;
use egregoria::Egregoria;
use geom::Transform;

pub struct InstancedRender {
    pub texs: Vec<SpriteBatchBuilder>,
}

impl InstancedRender {
    pub fn new(ctx: &mut GfxContext) -> Self {
        let car = Texture::from_path(ctx, "assets/car.png", Some("cartex")).unwrap(); // Unwrap ok: File is there
        let spr_car = SpriteBatchBuilder::new(car);
        let texs = vec![spr_car];
        InstancedRender { texs }
    }

    pub fn render(&mut self, goria: &mut Egregoria, fctx: &mut FrameContext) {
        for x in &mut self.texs {
            x.instances.clear();
        }

        for (trans, ar) in <(&Transform, &AssetRender)>::query().iter(&goria.world) {
            if ar.hide {
                continue;
            }

            let instance = InstanceRaw::new(
                trans.position(),
                trans.direction(),
                ar.z,
                [ar.tint.r, ar.tint.g, ar.tint.b],
                ar.scale,
            );

            self.texs[ar.id.id as usize].instances.push(instance);
        }

        for x in &mut self.texs {
            if let Some(x) = x.build(fctx.gfx) {
                fctx.objs.push(Box::new(x));
            }
        }
    }
}
