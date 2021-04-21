use crate::VBDesc;
use wgpu::VertexAttribute;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NorUvVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

u8slice_impl!(NorUvVertex);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float3, 1 => Float3, 2 => Float2];

impl VBDesc for NorUvVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}
