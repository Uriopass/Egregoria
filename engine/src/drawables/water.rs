use crate::meshbuild::MeshBuilder;
use crate::{
    CompiledModule, Drawable, GfxContext, Mesh, MeshVertex, PipelineBuilder, PipelineKey, Texture,
    TextureBuilder, TL,
};
use geom::AABB;
use std::sync::Arc;
use wgpu::{BindGroup, RenderPass, RenderPipeline};

#[derive(Clone)]
pub struct Water {
    mesh: Mesh,
    n_indices: u32,
    wavy_bg: Arc<BindGroup>,
}

#[derive(Hash)]
pub struct WaterPipeline;

impl Water {
    pub fn new(gfx: &mut GfxContext, bounds: AABB) -> Self {
        let mut mb = MeshBuilder::<false>::new_without_mat();

        mb.extend(
            None,
            &[
                MeshVertex {
                    position: [bounds.ll.x, bounds.ll.y, -10.0],
                    ..Default::default()
                },
                MeshVertex {
                    position: [bounds.ur.x, bounds.ll.y, -10.0],
                    ..Default::default()
                },
                MeshVertex {
                    position: [bounds.ur.x, bounds.ur.y, -10.0],
                    ..Default::default()
                },
                MeshVertex {
                    position: [bounds.ll.x, bounds.ur.y, -10.0],
                    ..Default::default()
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        );

        // unwrap ok: we just added vertices
        let mesh = mb.build(gfx).unwrap();

        let wavy = TextureBuilder::try_from_path("assets/sprites/wavy.jpeg")
            .expect("no wavy texture")
            .with_label("wavy")
            .with_mipmaps(&gfx.mipmap_gen)
            .with_srgb(false)
            .build(&gfx.device, &gfx.queue);

        let wavy_bg = Arc::new(wavy.bindgroup(
            &gfx.device,
            &Texture::bindgroup_layout(&gfx.device, [TL::Float]),
        ));

        Self {
            mesh,
            n_indices: 6,
            wavy_bg,
        }
    }
}

impl PipelineKey for WaterPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let vert = &mk_module("lit_mesh.vert", &[]);
        let frag = &mk_module("water.frag", &[]);

        let layouts = &[
            &gfx.render_params.layout,
            &Texture::bindgroup_layout(
                &gfx.device,
                [if gfx.samples > 1 {
                    TL::NonfilterableFloatMultisampled
                } else {
                    TL::NonfilterableFloat
                }],
            ),
            &Texture::bindgroup_layout(&gfx.device, [TL::Float]),
            &Texture::bindgroup_layout(&gfx.device, [TL::Float]),
        ];

        PipelineBuilder::color(
            "water",
            layouts,
            &[MeshVertex::desc()],
            vert,
            frag,
            gfx.sc_desc.format,
        )
        .with_samples(gfx.samples)
        .build(&gfx.device)
    }
}

impl Drawable for Water {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = gfx.get_pipeline(WaterPipeline);

        rp.set_pipeline(pipeline);

        rp.set_bind_group(1, &gfx.fbos.depth_bg, &[]);
        rp.set_bind_group(2, &self.wavy_bg, &[]);
        rp.set_bind_group(3, &gfx.water_bg, &[]);

        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);

        gfx.perf.drawcall(self.n_indices / 3);
    }
}
