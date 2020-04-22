pub struct Uniform<T: bytemuck::Pod> {
    pub buffer: wgpu::Buffer,
    pub bindgroup: wgpu::BindGroup,
    pub value: T,
}

impl<T: bytemuck::Pod> Uniform<T> {
    pub fn bindgroup_layout(
        device: &wgpu::Device,
        binding: u32,
        visibility: wgpu::ShaderStage,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false, // The dynamic field indicates whether this buffer will change size or not. This is useful if we want to store an array of things in our uniforms.
                },
            }],
            label: None,
        })
    }

    pub fn upload_to_gpu(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let staging_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.value]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of::<T>() as wgpu::BufferAddress,
        );
    }
}
