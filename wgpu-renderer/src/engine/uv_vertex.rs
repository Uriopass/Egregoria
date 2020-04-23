#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UvVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

unsafe impl bytemuck::Pod for UvVertex {}
unsafe impl bytemuck::Zeroable for UvVertex {}

impl UvVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<UvVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float3, 1 => Float4, 2 => Float2],
        }
    }
}
