use crate::{compile_shader, Drawable, GfxContext, IndexType, Texture, UvVertex, VBDesc};
use geom::{LinearColor, Vec2};
use std::path::Path;
use std::rc::Rc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{IndexFormat, RenderPass, RenderPipeline, VertexBufferLayout};

pub struct SpriteBatchBuilder {
    pub tex: Texture,
    pub instances: Vec<InstanceRaw>,
}

#[derive(Clone)]
pub struct SpriteBatch {
    vertex_buffer: Rc<wgpu::Buffer>,
    index_buffer: Rc<wgpu::Buffer>,
    instance_buffer: Rc<wgpu::Buffer>,
    pub n_indices: u32,
    pub n_instances: u32,
    pub alpha_blend: bool,
    pub tex: Texture,
    pub tex_bg: Rc<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InstanceRaw {
    pos: [f32; 3],
    dir: [f32; 2],
    tint: [f32; 4],
}

impl InstanceRaw {
    pub fn new(pos: Vec2, direction: Vec2, z: f32, col: LinearColor, scale: f32) -> InstanceRaw {
        Self {
            pos: [pos.x, pos.y, z],
            dir: [direction.x * scale, direction.y * scale],
            tint: col.into(),
        }
    }
}

u8slice_impl!(InstanceRaw);

impl VBDesc for InstanceRaw {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: Box::leak(Box::new(
                wgpu::vertex_attr_array![2 => Float3, 3 => Float2, 4 => Float4],
            )),
        }
    }
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [0.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

impl SpriteBatchBuilder {
    pub fn from_path(ctx: &GfxContext, path: impl AsRef<Path>) -> Self {
        Self {
            tex: Texture::from_path(ctx, path, None).unwrap(),
            instances: vec![],
        }
    }

    pub fn new(tex: Texture) -> Self {
        Self {
            tex,
            instances: vec![],
        }
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        let m = self.tex.extent.width.max(self.tex.extent.height) as f32;

        let x = self.tex.extent.width as f32 / (2.0 * m);
        let y = self.tex.extent.height as f32 / (2.0 * m);

        let v = [
            UvVertex {
                position: [-x, -y, 0.0],
                ..UV_VERTICES[0]
            },
            UvVertex {
                position: [x, -y, 0.0],
                ..UV_VERTICES[1]
            },
            UvVertex {
                position: [x, y, 0.0],
                ..UV_VERTICES[2]
            },
            UvVertex {
                position: [-x, y, 0.0],
                ..UV_VERTICES[3]
            },
        ];

        if self.instances.is_empty() {
            return None;
        }

        let vertex_buffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&v),
            usage: wgpu::BufferUsage::VERTEX,
        }));

        let index_buffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        }));

        let instance_buffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsage::VERTEX,
        }));

        let tex_bg = Rc::new(
            self.tex
                .bindgroup(&gfx.device, &pipeline.get_bind_group_layout(0)),
        );

        Some(SpriteBatch {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            n_indices: UV_INDICES.len() as u32,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            tex: self.tex.clone(),
            tex_bg,
        })
    }
}

impl Drawable for SpriteBatch {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = compile_shader("assets/shaders/spritebatch.vert", None);
        let frag = compile_shader("assets/shaders/spritebatch.frag", None);

        gfx.basic_pipeline(
            &[
                &Texture::bindgroup_layout(&gfx.device),
                &gfx.projection.layout,
            ],
            &[UvVertex::desc(), InstanceRaw::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &self.tex_bg, &[]);
        rp.set_bind_group(1, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..self.n_instances);
    }
}
