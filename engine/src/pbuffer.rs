use std::sync::Arc;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferBinding,
    BufferDescriptor, BufferSize, BufferSlice, BufferUsages, Device, Queue,
};

use crate::GfxContext;

/// Short for Persistent Buffer, keeps memory around to reuse it
#[derive(Clone)]
pub struct PBuffer {
    inner: Option<Arc<wgpu::Buffer>>,
    len: u32,
    capacity: u32,
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
        self.len = data.len() as u32;
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

    pub fn bindgroup(&self, device: &Device, layout: &BindGroupLayout) -> Option<wgpu::BindGroup> {
        if self.len == 0 {
            return None;
        }
        let buffer = self.inner.as_ref()?;
        Some(device.create_bind_group(&BindGroupDescriptor {
            label: Some("pbuffer bg"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer,
                    offset: 0,
                    size: Some(BufferSize::new(self.len as u64)?),
                }),
            }],
        }))
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

fn mk_buffer(device: &Device, usage: BufferUsages, size: u32) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&BufferDescriptor {
        label: Some("pbuffer"),
        size: size as u64,
        usage: usage | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    }))
}
