use crate::ToU8Slice;
use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroupEntry, BufferBinding, BufferBindingType, Device, ShaderStages};

pub struct Uniform<T> {
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bg: wgpu::BindGroup,
    value: T,
    pub changed: AtomicBool,
}

impl<T> Uniform<T>
where
    T: ToU8Slice,
{
    pub fn new(value: T, device: &wgpu::Device) -> Self {
        let layout = Self::bindgroup_layout(device);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: ToU8Slice::cast_slice(std::slice::from_ref(&value)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
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
            bg: bindgroup,
            value,
            changed: AtomicBool::from(true),
            layout,
        }
    }

    pub(crate) fn bindgroup_entry(&self, binding: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: None,
            }),
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        *self.changed.get_mut() = true;
        &mut self.value
    }

    pub fn upload_to_gpu(&self, queue: &wgpu::Queue) {
        if self.changed.load(Ordering::SeqCst) {
            let data = ToU8Slice::cast_slice(std::slice::from_ref(&self.value));
            queue.write_buffer(&self.buffer, 0, data);
            self.changed.store(false, Ordering::SeqCst);
        }
    }

    pub fn write_direct(&self, queue: &wgpu::Queue, value: &T) {
        let data = ToU8Slice::cast_slice(std::slice::from_ref(value));
        queue.write_buffer(&self.buffer, 0, data);
        self.changed.store(false, Ordering::SeqCst);
    }
}

impl<T: Clone + ToU8Slice> Uniform<T> {
    pub fn clone(&self, device: &Device) -> Self {
        let layout = Self::bindgroup_layout(device);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: ToU8Slice::cast_slice(std::slice::from_ref(&self.value)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
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
            bg: bindgroup,
            value: self.value.clone(),
            changed: AtomicBool::from(false),
            layout,
        }
    }
}

impl<T> Uniform<T> {
    pub(crate) fn bindgroup_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false, // The dynamic field indicates whether this buffer will change size or not. This is useful if we want to store an array of things in our uniforms.
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub(crate) fn bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[Self::bindgroup_layout_entry(0)],
            label: Some(format!("bglayout for {}", std::any::type_name::<T>()).as_ref()),
        })
    }
}
