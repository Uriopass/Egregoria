use crate::engine::{
    CompiledShader, Drawable, GfxContext, HasPipeline, IndexType, UvVertex, VBDesc,
};

use std::marker::PhantomData;
use std::rc::Rc;
use wgpu::{RenderPass, VertexBufferDescriptor};

pub trait Shaders {
    fn vert_shader() -> CompiledShader;
    fn frag_shader() -> CompiledShader;
}

pub struct ShadedBatchBuilder<T: Shaders> {
    pub instances: Vec<ShadedInstanceRaw>,
    _phantom: PhantomData<T>,
}

#[derive(Clone)]
pub struct ShadedBatch<T: Shaders> {
    vertex_buffer: Rc<wgpu::Buffer>,
    index_buffer: Rc<wgpu::Buffer>,
    instance_buffer: Rc<wgpu::Buffer>,
    pub n_indices: u32,
    pub n_instances: u32,
    pub alpha_blend: bool,
    _phantom: PhantomData<T>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShadedInstanceRaw {
    model: mint::ColumnMatrix4<f32>,
    tint: [f32; 4],
}

impl ShadedInstanceRaw {
    pub fn new(model: mint::ColumnMatrix4<f32>, tint: [f32; 4]) -> Self {
        Self { model, tint }
    }
}

u8slice_impl!(ShadedInstanceRaw);

impl VBDesc for ShadedInstanceRaw {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<ShadedInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![2 => Float4, 3 => Float4, 4 => Float4, 5 => Float4, 6 => Float4],
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

impl<T: Shaders> ShadedBatchBuilder<T> {
    pub fn new() -> Self {
        Self {
            instances: vec![],
            _phantom: Default::default(),
        }
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<ShadedBatch<T>> {
        if self.instances.is_empty() {
            return None;
        }

        let vertex_buffer =
            Rc::new(gfx.device.create_buffer_with_data(
                bytemuck::cast_slice(UV_VERTICES),
                wgpu::BufferUsage::VERTEX,
            ));
        let index_buffer =
            Rc::new(gfx.device.create_buffer_with_data(
                bytemuck::cast_slice(UV_INDICES),
                wgpu::BufferUsage::INDEX,
            ));
        let instance_buffer = Rc::new(gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.instances),
            wgpu::BufferUsage::VERTEX,
        ));

        Some(ShadedBatch {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            n_indices: UV_INDICES.len() as u32,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            _phantom: Default::default(),
        })
    }
}

impl<T: 'static + Shaders> HasPipeline for ShadedBatch<T> {
    fn create_pipeline(gfx: &GfxContext) -> super::PreparedPipeline {
        let vert = T::vert_shader();
        let frag = T::frag_shader();

        let pipeline = gfx.basic_pipeline(
            &[&gfx.projection_layout],
            &[UvVertex::desc(), ShadedInstanceRaw::desc()],
            &vert,
            &frag,
        );

        super::PreparedPipeline {
            pipeline,
            bindgroupslayouts: vec![],
        }
    }
}

impl<T: 'static + Shaders> Drawable for ShadedBatch<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline.pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        rp.set_vertex_buffer(1, &self.instance_buffer, 0, 0);
        rp.set_index_buffer(&self.index_buffer, 0, 0);
        rp.draw_indexed(0..self.n_indices, 0, 0..self.n_instances);
    }
}
