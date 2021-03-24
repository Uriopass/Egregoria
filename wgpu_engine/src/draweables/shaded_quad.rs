use crate::{
    Drawable, GfxContext, IndexType, Shaders, Texture, ToU8Slice, Uniform, UvVertex, VBDesc,
};

use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, IndexFormat, RenderPass, RenderPipeline};

pub struct ShadedQuadTex<T: Shaders, U: ToU8Slice + 'static> {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub alpha_blend: bool,
    pub uniform: Uniform<U>,
    pub tex: Texture,
    pub texbg: BindGroup,
    _phantom: PhantomData<T>,
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -3.0, 0.0],
        uv: [0.0, 2.0],
    },
    UvVertex {
        position: [3.0, 1.0, 0.0],
        uv: [2.0, 0.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2];

impl<T: Shaders, U: 'static + ToU8Slice> ShadedQuadTex<T, U> {
    pub fn new(gfx: &GfxContext, uniform: Uniform<U>, tex: Texture) -> ShadedQuadTex<T, U> {
        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let pipeline = gfx.get_pipeline::<Self>();

        let texbg = tex.bindgroup(&gfx.device, &pipeline.get_bind_group_layout(2));

        ShadedQuadTex {
            vertex_buffer,
            index_buffer,
            alpha_blend: false,
            uniform,
            texbg,
            _phantom: Default::default(),
            tex,
        }
    }
}

impl<T: 'static + Shaders, U: ToU8Slice + 'static> Drawable for ShadedQuadTex<T, U> {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = T::vert_shader(&gfx.device);
        let frag = T::frag_shader(&gfx.device);

        gfx.basic_pipeline(
            &[
                &gfx.inv_projection.layout,
                &Uniform::<U>::bindgroup_layout(&gfx.device),
                &Texture::bindgroup_layout(&gfx.device),
            ],
            &[UvVertex::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.uniform.upload_to_gpu(&gfx.queue);
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &gfx.inv_projection.bindgroup, &[]);
        rp.set_bind_group(1, &self.uniform.bindgroup, &[]);
        rp.set_bind_group(2, &self.texbg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
    }
}
