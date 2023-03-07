use crate::{Drawable, GfxContext, Material, Mesh, MeshBuilder, MeshVertex, Texture, VBDesc};
use std::sync::Arc;
use wgpu::{BindGroup, RenderPass, TextureSampleType};

#[derive(Clone)]
pub struct Water {
    mesh: Mesh,
    wavy_bg: Arc<BindGroup>,
}

#[derive(Hash)]
struct WaterPipeline;

impl Water {
    pub fn new(gfx: &mut GfxContext, w: f32, h: f32) -> Self {
        let mat = gfx.register_material(Material::new(gfx, gfx.palette()));
        let mut mb = MeshBuilder::new(mat);

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
        let wavy_bg =
            Arc::new(wavy.bindgroup(&gfx.device, &Texture::bindgroup_layout(&gfx.device)));

        Self { mesh, wavy_bg }
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
                    &Texture::bindgroup_layout_complex(
                        &gfx.device,
                        TextureSampleType::Float { filterable: false },
                        1,
                        gfx.samples > 1,
                    ),
                    &Texture::bindgroup_layout(&gfx.device),
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
        rp.draw_indexed(0..self.mesh.n_indices, 0, 0..1);
    }
}
