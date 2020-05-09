use crate::engine::{FrameContext, GfxContext, InstanceRaw, SpriteBatchBuilder, Texture};
use scale::physics::Transform;
use scale::rendering::assets::AssetRender;
use scale::specs::{Join, World, WorldExt};

pub struct InstancedRender {
    pub texs: Vec<SpriteBatchBuilder>,
}

impl InstancedRender {
    pub fn new(ctx: &mut GfxContext) -> Self {
        let car = Texture::from_path(ctx, "resources/car.png", Some("cartex")).unwrap();
        let spr_car = SpriteBatchBuilder::new(car);
        let texs = vec![spr_car];
        InstancedRender { texs }
    }

    pub fn render(&mut self, world: &mut World, fctx: &mut FrameContext) {
        let transforms = world.read_component::<Transform>();
        let ass_render = world.write_component::<AssetRender>();

        for x in &mut self.texs {
            x.instances.clear();
        }

        for (trans, ar) in (&transforms, &ass_render).join() {
            if ar.hide {
                continue;
            }

            let instance = InstanceRaw::new(
                trans.to_matrix4(ar.z),
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
