use crate::engine::{
    compile_shader, Drawable, GfxContext, HasPipeline, IndexType, Texture, UvVertex, VBDesc,
};

use std::rc::Rc;
use wgpu::{RenderPass, VertexBufferDescriptor};

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
    pub bind_group: Rc<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InstanceRaw {
    model: mint::ColumnMatrix4<f32>,
    tint: [f32; 3],
}

impl InstanceRaw {
    pub fn new(mut model: mint::ColumnMatrix4<f32>, tint: [f32; 3], scale: f32) -> InstanceRaw {
        model.x.x *= scale;
        model.x.y *= scale;
        model.y.x *= scale;
        model.y.y *= scale;
        Self { model, tint }
    }
}

u8slice_impl!(InstanceRaw);

impl VBDesc for InstanceRaw {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![2 => Float4, 3 => Float4, 4 => Float4, 5 => Float4, 6 => Float3],
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
    pub fn new(tex: Texture) -> Self {
        Self {
            tex,
            instances: vec![],
        }
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        let m = self.tex.width.max(self.tex.height);

        let x = self.tex.width / (2.0 * m);
        let y = self.tex.height / (2.0 * m);

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

        let vertex_buffer = Rc::new(
            gfx.device
                .create_buffer_with_data(bytemuck::cast_slice(&v), wgpu::BufferUsage::VERTEX),
        );
        let index_buffer =
            Rc::new(gfx.device.create_buffer_with_data(
                bytemuck::cast_slice(UV_INDICES),
                wgpu::BufferUsage::INDEX,
            ));
        let instance_buffer = Rc::new(gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.instances),
            wgpu::BufferUsage::VERTEX,
        ));

        let bind_group = Rc::new(gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bindgroupslayouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.tex.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.tex.sampler),
                },
            ],
            label: Some("bind group for spritebatch"),
        }));

        Some(SpriteBatch {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            n_indices: UV_INDICES.len() as u32,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            tex: self.tex.clone(),
            bind_group,
        })
    }
}

impl HasPipeline for SpriteBatch {
    fn create_pipeline(gfx: &GfxContext) -> super::PreparedPipeline {
        let vert = compile_shader("resources/shaders/spritebatch.vert", None);
        let frag = compile_shader("resources/shaders/spritebatch.frag", None);

        let layouts = vec![Texture::bindgroup_layout(&gfx.device)];

        let pipeline = gfx.basic_pipeline(
            &[&layouts[0], &gfx.projection_layout],
            &[UvVertex::desc(), InstanceRaw::desc()],
            &vert,
            &frag,
        );

        super::PreparedPipeline {
            pipeline,
            bindgroupslayouts: layouts,
        }
    }
}

impl Drawable for SpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline.pipeline);
        rp.set_bind_group(0, &self.bind_group, &[]);
        rp.set_bind_group(1, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        rp.set_vertex_buffer(1, &self.instance_buffer, 0, 0);
        rp.set_index_buffer(&self.index_buffer, 0, 0);
        rp.draw_indexed(0..self.n_indices, 0, 0..self.n_instances);
    }
}
