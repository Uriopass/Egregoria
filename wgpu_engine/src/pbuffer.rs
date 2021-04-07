use crate::GfxContext;
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferDescriptor, BufferSize, BufferUsage,
};

/// Short for Persistent Buffer, keeps memory around to reuse it
#[derive(Clone)]
pub struct PBuffer {
    inner: Option<Arc<wgpu::Buffer>>,
    len: u64,
    capacity: u64,
    usage: BufferUsage,
}

impl PBuffer {
    pub fn new(usage: BufferUsage) -> Self {
        Self {
            inner: None,
            len: 0,
            capacity: 0,
            usage,
        }
    }

    pub fn write(&mut self, gfx: &GfxContext, data: &[u8]) {
        self.len = data.len() as u64;
        if self.len == 0 {
            return;
        }
        if self.capacity < self.len {
            self.capacity = self.len.next_power_of_two();
            self.inner = Some(mk_buffer(gfx, self.usage, self.capacity));
        }
        gfx.queue.write_buffer(
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
                resource: BindingResource::Buffer {
                    buffer: &buffer,
                    offset: 0,
                    size: Some(BufferSize::new(self.len)?),
                },
            }],
        }))
    }

    pub fn bindgroup_layout(
        gfx: &GfxContext,
        visibility: wgpu::ShaderStage,
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

    pub fn inner(&self) -> Option<Arc<wgpu::Buffer>> {
        if self.len == 0 {
            return None;
        }
        self.inner.clone()
    }
}

fn mk_buffer(gfx: &GfxContext, usage: BufferUsage, size: u64) -> Arc<wgpu::Buffer> {
    Arc::new(gfx.device.create_buffer(&BufferDescriptor {
        label: Some("pbuffer"),
        size,
        usage: usage | BufferUsage::COPY_DST,
        mapped_at_creation: false,
    }))
}
