use crate::ToU8Slice;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::ShaderStage;

pub struct Uniform<T> {
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bindgroup: wgpu::BindGroup,
    pub value: T,
}

impl<T: Copy> Uniform<T>
where
    [T]: ToU8Slice,
{
    pub fn new(value: T, device: &wgpu::Device) -> Self {
        let layout = Self::bindgroup_layout(&device, ShaderStage::VERTEX);

        let r = [value];
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: ToU8Slice::to_slice(r.as_ref()),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
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
            layout,
        }
    }

    pub fn upload_to_gpu(&self, queue: &wgpu::Queue) {
        let r = [self.value];
        let data = ToU8Slice::to_slice(r.as_ref());
        queue.write_buffer(&self.buffer, 0, data);
    }
}

impl<T> Uniform<T> {
    pub fn bindgroup_layout(
        device: &wgpu::Device,
        visibility: wgpu::ShaderStage,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false, // The dynamic field indicates whether this buffer will change size or not. This is useful if we want to store an array of things in our uniforms.
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(format!("{} {}", "bglayout for", std::any::type_name::<T>()).as_ref()),
        })
    }
}
