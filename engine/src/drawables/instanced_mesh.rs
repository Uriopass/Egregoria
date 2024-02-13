#![allow(clippy::collapsible_else_if)]

use crate::pbuffer::PBuffer;
use crate::{Drawable, GfxContext, Mesh, MeshPipeline};
use geom::{LinearColor, Matrix4, Vec3};
use std::sync::Arc;
use wgpu::{BufferUsages, IndexFormat, RenderPass, VertexAttribute, VertexBufferLayout};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MeshInstance {
    pub pos: Vec3,
    pub dir: Vec3,
    pub tint: LinearColor,
}

u8slice_impl!(MeshInstance);

const ATTRS: &[VertexAttribute] =
    &wgpu::vertex_attr_array![5 => Float32x3, 6 => Float32x3, 7 => Float32x4];

impl MeshInstance {
    pub(crate) const fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

pub struct InstancedMeshBuilder<const PERSISTENT: bool> {
    mesh: Mesh,
    ibuffer: PBuffer,
    pub instances: Vec<MeshInstance>,
}

impl<const PERSISTENT: bool> InstancedMeshBuilder<PERSISTENT> {
    pub fn new(mesh: Mesh) -> Self {
        InstancedMeshBuilder {
            mesh,
            instances: Vec::with_capacity(4),
            ibuffer: PBuffer::new(BufferUsages::VERTEX),
        }
    }

    pub fn new_ref(mesh: &Mesh) -> Self {
        InstancedMeshBuilder {
            mesh: mesh.clone(),
            instances: Vec::with_capacity(4),
            ibuffer: PBuffer::new(BufferUsages::VERTEX),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<InstancedMesh> {
        if self.instances.is_empty() {
            return None;
        }

        let mut temp;
        let ibuffer;
        if PERSISTENT {
            ibuffer = &mut self.ibuffer;
        } else {
            temp = PBuffer::new(BufferUsages::VERTEX);
            ibuffer = &mut temp;
        }

        ibuffer.write(gfx, bytemuck::cast_slice(&self.instances));

        Some(InstancedMesh {
            mesh: self.mesh.clone(),
            instance_buffer: ibuffer.inner()?,
            n_instances: self.instances.len() as u32,
        })
    }
}

#[derive(Clone)]
pub struct InstancedMesh {
    mesh: Mesh,
    instance_buffer: Arc<wgpu::Buffer>,
    n_instances: u32,
}

impl Drawable for InstancedMesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let Some(lod_select) = self.mesh.lods.first() else {
            return;
        };

        rp.set_bind_group(1, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, indices) in &lod_select.primitives {
            let mat = gfx.material(*mat);
            let pipeline = gfx.get_pipeline(MeshPipeline {
                offscreen_render: false,
                instanced: true,
                alpha: false,
                smap: false,
                depth: false,
            });
            rp.set_pipeline(pipeline);
            rp.set_bind_group(2, &mat.bg, &[]);
            rp.draw_indexed(indices.clone(), 0, 0..self.n_instances);
            gfx.perf
                .drawcall((indices.end - indices.start) / 3 * self.n_instances);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        let Some(lod_select) = self.mesh.lods.first() else {
            return;
        };

        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, indices) in &lod_select.primitives {
            let mat = gfx.material(*mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                offscreen_render: false,
                instanced: true,
                alpha: mat.transparent,
                smap: shadow_cascade.is_some(),
                depth: true,
            }));

            if mat.transparent {
                rp.set_bind_group(1, &mat.bg, &[]);
            }
            rp.draw_indexed(indices.clone(), 0, 0..self.n_instances);
            gfx.perf.depth_drawcall(
                (indices.end - indices.start) / 3 * self.n_instances,
                shadow_cascade.is_some(),
            );
        }
    }
}
