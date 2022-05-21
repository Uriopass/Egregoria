use crate::GfxContext;
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferDescriptor,
    BufferSize, BufferSlice, BufferUsages, Device, Queue,
};

/// Short for Persistent Buffer, keeps memory around to reuse it
#[derive(Clone)]
pub struct PBuffer {
    inner: Option<Arc<wgpu::Buffer>>,
    len: u64,
    capacity: u64,
    usage: BufferUsages,
}

impl PBuffer {
    pub fn new(usage: BufferUsages) -> Self {
        Self {
            inner: None,
            len: 0,
            capacity: 0,
            usage,
        }
    }

    pub fn write(&mut self, gfx: &GfxContext, data: &[u8]) {
        self.write_qd(&gfx.queue, &gfx.device, data);
    }

    pub fn write_qd(&mut self, queue: &Queue, device: &Device, data: &[u8]) {
        self.len = data.len() as u64;
        if self.len == 0 {
            return;
        }
        if self.capacity < self.len {
            self.capacity = self.len.next_power_of_two();
            self.inner = Some(mk_buffer(device, self.usage, self.capacity));
            //log::info!("reallocating {} bytes", self.capacity);
        }
        queue.write_buffer(
            self.inner.as_ref().expect("inner was never initialized ?"),
            0,
            data,
        );
    }

    pub fn bindgroup(&self, gfx: &GfxContext, layout: &BindGroupLayout) -> Option<wgpu::BindGroup> {
        if self.len == 0 {
            return None;
        }
        let buffer = self.inner.as_ref()?;
        Some(gfx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("pbuffer bg"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer,
                    offset: 0,
                    size: Some(BufferSize::new(self.len)?),
                }),
            }],
        }))
    }

    pub fn bindgroup_layout(
        gfx: &GfxContext,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BufferBindingType,
    ) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("pbuffer bglayout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    ty: BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty,
                    },
                    count: None,
                }],
            })
    }

    pub fn slice(&self) -> Option<BufferSlice> {
        if self.len == 0 {
            return None;
        }
        self.inner.as_ref().map(|x| x.slice(..))
    }

    pub fn inner(&self) -> Option<Arc<wgpu::Buffer>> {
        if self.len == 0 {
            return None;
        }
        self.inner.clone()
    }
}

fn mk_buffer(device: &Device, usage: BufferUsages, size: u64) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&BufferDescriptor {
        label: Some("pbuffer"),
        size,
        usage: usage | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    }))
}
