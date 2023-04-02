use crate::pbuffer::PBuffer;
use crate::{
    bg_layout_litmesh, Drawable, GfxContext, Material, MaterialID, MetallicRoughness, Texture,
    UvVertex, VBDesc,
};
use geom::{LinearColor, Vec3};
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::{BufferUsages, IndexFormat, RenderPass, VertexAttribute, VertexBufferLayout};

pub struct SpriteBatchBuilder {
    pub material: MaterialID,
    instances: Vec<InstanceRaw>,
    stretch_x: f32,
    stretch_y: f32,
    pub instance_sbuffer: PBuffer,
}

pub struct SpriteBatch {
    instance_buf: Arc<wgpu::Buffer>,
    pub n_instances: u32,
    pub material: MaterialID,
}

impl SpriteBatch {
    pub fn builder(gfx: &mut GfxContext, tex: Arc<Texture>) -> SpriteBatchBuilder {
        SpriteBatchBuilder::new(tex, gfx)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct InstanceRaw {
    tint: [f32; 4],
    pos: Vec3,
    dir: Vec3,
    scale: [f32; 2],
}

u8slice_impl!(InstanceRaw);

impl VBDesc for InstanceRaw {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        const ARR: &[VertexAttribute; 4] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x3, 4 => Float32x3, 5 => Float32x2];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARR,
        }
    }
}

impl SpriteBatchBuilder {
    pub fn from_path(gfx: &mut GfxContext, path: impl Into<PathBuf>) -> Self {
        let tex = gfx.texture(path, "some spritebatch tex");
        Self::new(tex, gfx)
    }

    pub fn clear(&mut self) {
        self.instances.clear()
    }

    pub fn push(&mut self, pos: Vec3, dir: Vec3, col: LinearColor, scale: (f32, f32)) -> &mut Self {
        self.instances.push(InstanceRaw {
            tint: col.into(),
            dir,
            scale: [scale.0 * self.stretch_x, scale.1 * self.stretch_y],
            pos,
        });
        self
    }

    pub fn new(albedo: Arc<Texture>, gfx: &mut GfxContext) -> Self {
        let max_extent = albedo.extent.width.max(albedo.extent.height) as f32;

        let stretch_x = 0.5 * albedo.extent.width as f32 / max_extent;
        let stretch_y = 0.5 * albedo.extent.height as f32 / max_extent;

        let mat = Material::new(
            gfx,
            albedo,
            MetallicRoughness {
                metallic: 0.0,
                roughness: 1.0,
                tex: None,
            },
        );
        let matid = gfx.register_material(mat);

        Self {
            stretch_x,
            stretch_y,
            material: matid,
            instances: vec![],
            instance_sbuffer: PBuffer::new(BufferUsages::VERTEX),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<SpriteBatch> {
        if self.instances.is_empty() {
            return None;
        }

        self.instance_sbuffer
            .write(gfx, bytemuck::cast_slice(&self.instances));

        Some(SpriteBatch {
            instance_buf: self.instance_sbuffer.inner().unwrap(),
            n_instances: self.instances.len() as u32,
            material: self.material,
        })
    }
}

impl SpriteBatch {
    pub fn setup(gfx: &mut GfxContext) {
        gfx.register_pipeline(
            SBPipeline,
            &["spritebatch.vert", "pixel.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];

                gfx.color_pipeline(
                    "spritebatch",
                    &[
                        &gfx.projection.layout,
                        &gfx.render_params.layout,
                        &Material::bindgroup_layout(&gfx.device),
                        &bg_layout_litmesh(&gfx.device),
                    ],
                    &[UvVertex::desc(), InstanceRaw::desc()],
                    vert,
                    frag,
                    false,
                )
            }),
        );
    }
}

impl Drawable for SpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline(SBPipeline);
        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
        rp.set_vertex_buffer(1, self.instance_buf.slice(..));
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &gfx.material(self.material).bg, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..6, 0, 0..self.n_instances);
    }
}

#[derive(Hash)]
struct SBPipeline;
