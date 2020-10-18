use crate::{Drawable, GfxContext, IndexType, Shaders, ToU8Slice, Uniform, UvVertex, VBDesc};

use std::marker::PhantomData;
use std::rc::Rc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::RenderPass;

pub struct ShadedQuad<T: Shaders, U: ToU8Slice> {
    vertex_buffer: Rc<wgpu::Buffer>,
    index_buffer: Rc<wgpu::Buffer>,
    pub n_indices: u32,
    pub alpha_blend: bool,
    pub uniform: Uniform<U>,
    _phantom: PhantomData<T>,
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

impl<T: Shaders, U: ToU8Slice> ShadedQuad<T, U> {
    pub fn new(gfx: &GfxContext, uniform: Uniform<U>) -> ShadedQuad<T, U> {
        let vertex_buffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        }));

        let index_buffer = Rc::new(gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        }));

        ShadedQuad {
            vertex_buffer,
            index_buffer,
            n_indices: UV_INDICES.len() as u32,
            alpha_blend: false,
            uniform,
            _phantom: Default::default(),
        }
    }
}

impl<T: 'static + Shaders, U: ToU8Slice + 'static> Drawable for ShadedQuad<T, U> {
    fn create_pipeline(gfx: &GfxContext) -> super::PreparedPipeline {
        let vert = T::vert_shader();
        let frag = T::frag_shader();

        let pipeline = gfx.basic_pipeline(
            &[
                &gfx.projection.layout,
                &Uniform::<U>::bindgroup_layout(&gfx.device),
            ],
            &[UvVertex::desc()],
            vert,
            frag,
        );

        super::PreparedPipeline(pipeline)
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.uniform.upload_to_gpu(&gfx.queue);
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline.0);
        rp.set_bind_group(0, &gfx.inv_projection.bindgroup, &[]);
        rp.set_bind_group(1, &self.uniform.bindgroup, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..));
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
