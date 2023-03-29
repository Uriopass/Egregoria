use crate::{Drawable, GfxContext, Mesh, MeshBuilder, MeshVertex, Texture, VBDesc, TL};
use std::sync::Arc;
use wgpu::{BindGroup, RenderPass};

#[derive(Clone)]
pub struct Water {
    mesh: Mesh,
    n_indices: u32,
    wavy_bg: Arc<BindGroup>,
}

#[derive(Hash)]
struct WaterPipeline;

impl Water {
    pub fn new(gfx: &mut GfxContext, w: f32, h: f32) -> Self {
        let mut mb = MeshBuilder::new_without_mat();

        mb.vertices.extend_from_slice(&[
            MeshVertex {
                position: [0.0, 0.0, -10.0],
                ..Default::default()
            },
            MeshVertex {
                position: [w, 0.0, -10.0],
                ..Default::default()
            },
            MeshVertex {
                position: [w, h, -10.0],
                ..Default::default()
            },
            MeshVertex {
                position: [0.0, h, -10.0],
                ..Default::default()
            },
        ]);

        mb.indices.extend_from_slice(&[0, 1, 2, 2, 3, 0]);

        // unwrap ok: we just added vertices
        let mesh = mb.build(gfx).unwrap();

        let wavy = gfx.texture("assets/sprites/wavy.jpeg", "wavy");
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

    pub(crate) fn setup(gfx: &mut GfxContext) {
        gfx.register_pipeline(
            WaterPipeline,
            &["lit_mesh.vert", "water.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];

                let layouts = &[
                    &gfx.projection.layout,
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
                ];

                gfx.color_pipeline(
                    "water pipeline",
                    layouts,
                    &[MeshVertex::desc()],
                    vert,
                    frag,
                    0,
                    false,
                )
            }),
        );
    }
}

impl Drawable for Water {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = gfx.get_pipeline(WaterPipeline);

        rp.set_pipeline(pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &gfx.fbos.depth_bg, &[]);
        rp.set_bind_group(3, &self.wavy_bg, &[]);

        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
