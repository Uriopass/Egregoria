#![allow(dead_code)]
use crate::pbuffer::PBuffer;
use crate::{
    bg_layout_litmesh, Drawable, GfxContext, Mesh, MeshVertex, RenderParams, Texture, Uniform,
    VBDesc,
};
use geom::{LinearColor, Vec3};
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
    &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x4];

impl VBDesc for MeshInstance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

pub struct InstancedMeshBuilder {
    mesh: Mesh,
    ibuffer: PBuffer,
    pub instances: Vec<MeshInstance>,
}

impl InstancedMeshBuilder {
    pub fn new(mesh: Mesh) -> Self {
        InstancedMeshBuilder {
            mesh,
            instances: Vec::with_capacity(4),
            ibuffer: PBuffer::new(BufferUsages::VERTEX),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<InstancedMesh> {
        if self.instances.is_empty() {
            return None;
        }

        self.ibuffer
            .write(gfx, bytemuck::cast_slice(&self.instances));

        Some(InstancedMesh {
            mesh: self.mesh.clone(),
            instance_buffer: self.ibuffer.inner()?,
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

impl InstancedMesh {
    pub fn setup(gfx: &mut GfxContext) {
        gfx.register_pipeline::<Self>(
            &["instanced_mesh.vert", "pixel.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];
                let vb = &[MeshVertex::desc(), MeshInstance::desc()];
                gfx.color_pipeline(
                    "instanced_mesh",
                    &[
                        &gfx.projection.layout,
                        &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                        &Texture::bindgroup_layout(&gfx.device),
                        &bg_layout_litmesh(&gfx.device),
                    ],
                    vb,
                    vert,
                    frag,
                    0,
                )
            }),
        );
        gfx.register_pipeline::<InstancedMeshDepth>(
            &["instanced_mesh.vert"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let vb = &[MeshVertex::desc(), MeshInstance::desc()];
                gfx.depth_pipeline(vb, vert, None, false)
            }),
        );
        gfx.register_pipeline::<InstancedMeshDepthAlpha>(
            &["instanced_mesh.vert", "alpha_discard.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];
                let vb = &[MeshVertex::desc(), MeshInstance::desc()];
                gfx.depth_pipeline_bglayout(
                    vb,
                    vert,
                    Some(frag),
                    false,
                    &[
                        &gfx.projection.layout,
                        &Texture::bindgroup_layout(&gfx.device),
                    ],
                )
            }),
        );
        gfx.register_pipeline::<InstancedMeshDepthSMap>(
            &["instanced_mesh.vert"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let vb = &[MeshVertex::desc(), MeshInstance::desc()];
                gfx.depth_pipeline(vb, vert, None, true)
            }),
        );
        gfx.register_pipeline::<InstancedMeshDepthSMapAlpha>(
            &["instanced_mesh.vert", "alpha_discard.frag"],
            Box::new(move |m, gfx| {
                let vert = &m[0];
                let frag = &m[1];
                let vb = &[MeshVertex::desc(), MeshInstance::desc()];
                gfx.depth_pipeline_bglayout(
                    vb,
                    vert,
                    Some(frag),
                    true,
                    &[
                        &gfx.projection.layout,
                        &Texture::bindgroup_layout(&gfx.device),
                    ],
                )
            }),
        );
    }
}

impl Drawable for InstancedMesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &self.mesh.albedo_bg, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.mesh.n_indices, 0, 0..self.n_instances);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        if self.mesh.transparent {
            if shadow_map {
                rp.set_pipeline(gfx.get_pipeline::<InstancedMeshDepthSMapAlpha>());
            } else {
                rp.set_pipeline(gfx.get_pipeline::<InstancedMeshDepthAlpha>());
            }
        } else {
            if shadow_map {
                rp.set_pipeline(gfx.get_pipeline::<InstancedMeshDepthSMap>());
            } else {
                rp.set_pipeline(gfx.get_pipeline::<InstancedMeshDepth>());
            }
        }

        rp.set_bind_group(0, proj, &[]);
        if self.mesh.transparent {
            rp.set_bind_group(1, &self.mesh.albedo_bg, &[]);
        }
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.mesh.n_indices, 0, 0..self.n_instances);
    }
}

struct InstancedMeshDepth;
struct InstancedMeshDepthSMap;
struct InstancedMeshDepthAlpha;
struct InstancedMeshDepthSMapAlpha;
