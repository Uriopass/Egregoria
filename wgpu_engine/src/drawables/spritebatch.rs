use crate::pbuffer::PBuffer;
use crate::{bg_layout_litmesh, compile_shader, Drawable, GfxContext, Texture, UvVertex, VBDesc};
use geom::{LinearColor, Vec3};
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::{BindGroup, BufferUsages, IndexFormat, RenderPass, VertexBufferLayout};

pub struct SpriteBatchBuilder {
    pub albedo: Arc<Texture>,
    instances: Vec<InstanceRaw>,
    stretch_x: f32,
    stretch_y: f32,
    pub instance_sbuffer: PBuffer,
}

pub struct SpriteBatch {
    instance_buf: Arc<wgpu::Buffer>,
    pub n_instances: u32,
    pub albedo_bg: BindGroup,
}

impl SpriteBatch {
    pub fn builder(tex: Arc<Texture>) -> SpriteBatchBuilder {
        SpriteBatchBuilder::new(tex)
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
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Box::leak(Box::new(
                wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x3, 4 => Float32x3, 5 => Float32x2],
            )),
        }
    }
}

impl SpriteBatchBuilder {
    pub fn from_path(ctx: &mut GfxContext, path: impl Into<PathBuf>) -> Self {
        Self::new(ctx.texture(path, "some spritebatch tex"))
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

    pub fn new(albedo: Arc<Texture>) -> Self {
        let m = albedo.extent.width.max(albedo.extent.height) as f32;

        let stretch_x = 0.5 * albedo.extent.width as f32 / m;
        let stretch_y = 0.5 * albedo.extent.height as f32 / m;

        Self {
            stretch_x,
            stretch_y,
            albedo,
            instances: vec![],
            instance_sbuffer: PBuffer::new(BufferUsages::VERTEX),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        if self.instances.is_empty() {
            return None;
        }

        self.instance_sbuffer
            .write(gfx, bytemuck::cast_slice(&self.instances));

        let albedo_bg = self
            .albedo
            .bindgroup(&gfx.device, &pipeline.get_bind_group_layout(2));

        Some(SpriteBatch {
            instance_buf: self.instance_sbuffer.inner().unwrap(),
            n_instances: self.instances.len() as u32,
            albedo_bg,
        })
    }
}

impl SpriteBatch {
    pub fn setup(gfx: &mut GfxContext) {
        let vert = compile_shader(&gfx.device, "assets/shaders/spritebatch.vert", None);
        let frag = compile_shader(&gfx.device, "assets/shaders/pixel.frag", None);

        let pipe = gfx.color_pipeline(
            &[
                &gfx.projection.layout,
                &gfx.render_params.layout,
                &Texture::bindgroup_layout(&gfx.device),
                &bg_layout_litmesh(&gfx.device),
            ],
            &[UvVertex::desc(), InstanceRaw::desc()],
            &vert,
            &frag,
        );
        gfx.register_pipeline::<Self>(pipe);

        gfx.register_pipeline::<SBDepthMultisample>(gfx.depth_pipeline(
            &[UvVertex::desc(), InstanceRaw::desc()],
            &vert,
            false,
        ));

        gfx.register_pipeline::<SBDepth>(gfx.depth_pipeline(
            &[UvVertex::desc(), InstanceRaw::desc()],
            &vert,
            true,
        ));
    }
}

impl Drawable for SpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(pipeline);
        rp.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
        rp.set_vertex_buffer(1, self.instance_buf.slice(..));
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &self.albedo_bg, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..6, 0, 0..self.n_instances);
    }
    /*fn draw_depth<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        return;
        if gfx.samples == 1 {
            rp.set_pipeline(&gfx.get_pipeline::<SBDepth>());
        } else {
            return;
            rp.set_pipeline(&gfx.get_pipeline::<SBDepthMultisample>());
        }
        rp.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
        rp.set_vertex_buffer(1, self.instance_buf.slice(..));
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &self.tex_bg, &[]);
        rp.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..6, 0, 0..self.n_instances);
    }*/
}

struct SBDepthMultisample;
struct SBDepth;
