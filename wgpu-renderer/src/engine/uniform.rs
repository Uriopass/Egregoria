use crate::engine::ToU8Slice;

pub struct Uniform<T> {
    pub buffer: wgpu::Buffer,
    pub bindgroup: wgpu::BindGroup,
    pub value: T,
}

impl<T: Copy> Uniform<T>
where
    [T]: ToU8Slice,
{
    pub fn new(value: T, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = device.create_buffer_with_data(
            ToU8Slice::to_slice([value].as_ref()),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &buffer,
                    range: 0..std::mem::size_of_val(&value) as wgpu::BufferAddress,
                },
            }],
            label: Some(
                format!(
                    "{} {}",
                    "uniform bindgroup for value of type",
                    std::any::type_name::<T>()
                )
                .as_ref(),
            ),
        });
        Self {
            buffer,
            bindgroup,
            value,
        }
    }

    pub fn upload_to_gpu(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let staging_buffer = device.create_buffer_with_data(
            ToU8Slice::to_slice([self.value].as_ref()),
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

impl<T> Uniform<T> {
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
            label: Some(
                format!(
                    "{} {}",
                    "Bindgroup layout for value of type",
                    std::any::type_name::<T>()
                )
                .as_ref(),
            ),
        })
    }
}
