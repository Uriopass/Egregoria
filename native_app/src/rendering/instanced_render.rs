use common::Z_PATH_NOT_FOUND;
use egregoria::map_dynamic::Itinerary;
use egregoria::rendering::assets::AssetRender;
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec2};
use legion::query::*;
use wgpu_engine::{FrameContext, GfxContext, SpriteBatchBuilder};

pub struct InstancedRender {
    pub texs: Vec<SpriteBatchBuilder>,
    pub path_not_found: SpriteBatchBuilder,
}

impl InstancedRender {
    pub fn new(ctx: &mut GfxContext) -> Self {
        let car = ctx.texture("assets/car.png", Some("cartex"));
        let spr_car = SpriteBatchBuilder::new(car);

        let truck = ctx.texture("assets/truck.png", Some("trucktex"));
        let spr_truck = SpriteBatchBuilder::new(truck);

        let texs = vec![spr_car, spr_truck];
        InstancedRender {
            texs,
            path_not_found: SpriteBatchBuilder::new(
                ctx.texture("assets/path_not_found.png", Some("path_not_found")),
            ),
        }
    }

    pub fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext) {
        for x in &mut self.texs {
            x.clear();
        }

        for (trans, ar) in <(&Transform, &AssetRender)>::query().iter(goria.world()) {
            if ar.hide {
                continue;
            }

            self.texs[ar.id.id as usize].push(
                trans.position(),
                trans.direction(),
                ar.z,
                ar.tint.into(),
                (ar.scale, ar.scale),
            );
        }

        self.path_not_found.clear();
        for (trans, itin) in <(&Transform, &Itinerary)>::query().iter(goria.world()) {
            let itin: &Itinerary = itin;
            if let Some(wait) = itin.is_wait_for_reroute() {
                if wait == 0 {
                    continue;
                }

                let r = wait as f32 / 200.0;
                let off = 1.0 - r;

                let s = 7.0;
                self.path_not_found.push(
                    trans.position() + off * 3.0 * Vec2::UNIT_Y,
                    Vec2::UNIT_X,
                    Z_PATH_NOT_FOUND,
                    LinearColor::RED.a(r),
                    (s, s),
                );
            }
        }

        for x in &mut self.texs {
            if let Some(x) = x.build(fctx.gfx) {
                fctx.objs.push(Box::new(x));
            }
        }
        if let Some(x) = self.path_not_found.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
    }
}
