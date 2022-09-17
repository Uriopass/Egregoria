use crate::{Drawable, GfxContext, Mesh, MeshBuilder, MeshVertex, Texture, VBDesc};
use wgpu::{RenderPass, TextureSampleType};

#[derive(Clone)]
pub struct Water {
    mesh: Mesh,
}

impl Water {
    pub fn new(gfx: &GfxContext, w: f32, h: f32) -> Self {
        let mut mb = MeshBuilder::new();

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
        let mesh = mb.build(gfx, gfx.palette()).unwrap();

        Self { mesh }
    }

    pub(crate) fn setup(gfx: &mut GfxContext) {
        gfx.register_pipeline::<Self>(
            &["lit_mesh.vert", "water.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];

                let layouts = &[
                    &gfx.projection.layout,
                    &gfx.render_params.layout,
                    &Texture::bindgroup_layout_complex(
                        &gfx.device,
                        TextureSampleType::Float { filterable: false },
                        1,
                        gfx.samples > 1,
                    ),
                ];

                gfx.color_pipeline(
                    "water pipeline",
                    layouts,
                    &[MeshVertex::desc()],
                    vert,
                    frag,
                    0,
                )
            }),
        );
    }
}

impl Drawable for Water {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = gfx.get_pipeline::<Self>();

        rp.set_pipeline(pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &gfx.fbos.depth_bg, &[]);

        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rp.draw_indexed(0..self.mesh.n_indices, 0, 0..1);
    }
}
