use crate::pbuffer::PBuffer;
use crate::{
    bg_layout_litmesh, CompiledModule, Drawable, GfxContext, Material, MaterialID,
    MetallicRoughness, PipelineBuilder, PipelineKey, Texture, UvVertex,
};
use geom::{LinearColor, Vec3};
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::{
    BufferUsages, IndexFormat, RenderPass, RenderPipeline, VertexAttribute, VertexBufferLayout,
};

pub struct SpriteBatchBuilder<const PERSISTENT: bool> {
    pub material: MaterialID,
    instances: Vec<InstanceRaw>,
    stretch_x: f32,
    stretch_y: f32,
    pub instance_sbuffer: Option<Box<PBuffer>>,
}

#[derive(Clone)]
pub struct SpriteBatch {
    instance_buf: Arc<wgpu::Buffer>,
    pub n_instances: u32,
    pub material: MaterialID,
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

impl InstanceRaw {
    fn desc() -> VertexBufferLayout<'static> {
        const ARR: &[VertexAttribute; 4] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x3, 4 => Float32x3, 5 => Float32x2];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARR,
        }
    }
}

impl<const PERSISTENT: bool> SpriteBatchBuilder<PERSISTENT> {
    pub fn from_path(gfx: &mut GfxContext, path: impl Into<PathBuf>) -> Self {
        let tex = gfx.texture(path, "some spritebatch tex");
        Self::new(&tex, gfx)
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

    pub fn new(albedo: &Texture, gfx: &mut GfxContext) -> Self {
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
            None,
        );
        let matid = gfx.register_material(mat);

        Self {
            stretch_x,
            stretch_y,
            material: matid,
            instances: vec![],
            instance_sbuffer: PERSISTENT.then(|| Box::new(PBuffer::new(BufferUsages::VERTEX))),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<SpriteBatch> {
        if self.instances.is_empty() {
            return None;
        }

        let mut temp;
        let ibuffer;

        if PERSISTENT {
            unsafe {
                ibuffer = self.instance_sbuffer.as_deref_mut().unwrap_unchecked();
            }
        } else {
            temp = PBuffer::new(BufferUsages::VERTEX);
            ibuffer = &mut temp;
        }

        ibuffer.write(gfx, bytemuck::cast_slice(&self.instances));

        Some(SpriteBatch {
            instance_buf: ibuffer.inner().unwrap(),
            n_instances: self.instances.len() as u32,
            material: self.material,
        })
    }
}

impl PipelineKey for SBPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let vert = &mk_module("spritebatch.vert", &[]);
        let frag = &mk_module("pixel.frag", &[]);

        PipelineBuilder::color(
            "spritebatch",
            &[
                &gfx.render_params.layout,
                &bg_layout_litmesh(&gfx.device),
                &Material::bindgroup_layout(&gfx.device),
            ],
            &[UvVertex::desc(), InstanceRaw::desc()],
            vert,
            frag,
            gfx.sc_desc.format,
        )
        .with_samples(gfx.samples)
        .build(&gfx.device)
    }
}

impl Drawable for SpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline(SBPipeline);
        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
        rp.set_vertex_buffer(1, self.instance_buf.slice(..));

        rp.set_bind_group(1, &gfx.simplelit_bg, &[]);
        rp.set_bind_group(2, &gfx.material(self.material).bg, &[]);
        rp.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..6, 0, 0..self.n_instances);

        gfx.perf.drawcall(2 * self.n_instances);
    }
}

#[derive(Hash)]
struct SBPipeline;
