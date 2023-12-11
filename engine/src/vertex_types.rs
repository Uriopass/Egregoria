use geom::Vec3;
use wgpu::VertexAttribute;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: Vec3,
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub tangent: [f32; 4],
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: Vec3::Z,
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            tangent: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

u8slice_impl!(MeshVertex);

const ATTRS_MV: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 4 => Float32x4];

impl MeshVertex {
    pub(crate) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS_MV,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UvVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

u8slice_impl!(UvVertex);

const ATTRS_UV: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

impl UvVertex {
    pub(crate) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<UvVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS_UV,
        }
    }
}
