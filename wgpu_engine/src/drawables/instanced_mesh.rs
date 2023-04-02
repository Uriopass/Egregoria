#![allow(clippy::collapsible_else_if)]
use crate::pbuffer::PBuffer;
use crate::{
    bg_layout_litmesh, Drawable, GfxContext, Material, Mesh, MeshVertex, RenderParams, Uniform,
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

#[derive(Clone, Copy, Hash)]
struct InstancedMeshPipeline {
    alpha: bool,
    smap: bool,
    depth: bool,
    double_sided: bool,
}

impl InstancedMesh {
    pub fn setup(gfx: &mut GfxContext) {
        for double_sided in [false, true] {
            let pipeline = InstancedMeshPipeline {
                alpha: false,
                smap: false,
                depth: false,
                double_sided,
            };

            gfx.register_pipeline(
                pipeline,
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
                            &Material::bindgroup_layout(&gfx.device),
                            &bg_layout_litmesh(&gfx.device),
                        ],
                        vb,
                        vert,
                        frag,
                        double_sided,
                    )
                }),
            );

            for smap in [false, true] {
                let pipeline_depth = InstancedMeshPipeline {
                    alpha: false,
                    smap,
                    depth: true,
                    double_sided,
                };
                gfx.register_pipeline(
                    pipeline_depth,
                    &["instanced_mesh.vert"],
                    Box::new(move |m, gfx| {
                        let vert = &m[0];
                        let vb = &[MeshVertex::desc(), MeshInstance::desc()];

                        gfx.depth_pipeline(vb, vert, None, smap, double_sided)
                    }),
                );

                let pipeline_depth_alpha = InstancedMeshPipeline {
                    alpha: true,
                    smap,
                    depth: true,
                    double_sided,
                };
                gfx.register_pipeline(
                    pipeline_depth_alpha,
                    &["instanced_mesh.vert", "alpha_discard.frag"],
                    Box::new(move |m, gfx| {
                        let vert = &m[0];
                        let frag = &m[1];
                        let vb = &[MeshVertex::desc(), MeshInstance::desc()];

                        gfx.depth_pipeline_bglayout(
                            vb,
                            vert,
                            Some(frag),
                            smap,
                            &[
                                &gfx.projection.layout,
                                &Material::bindgroup_layout(&gfx.device),
                            ],
                            double_sided,
                        )
                    }),
                );
            }
        }
    }
}

impl Drawable for InstancedMesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, offset, length) in self.mesh.iter_materials() {
            let mat = gfx.material(mat);
            let pipeline = &gfx.get_pipeline(InstancedMeshPipeline {
                alpha: false,
                smap: false,
                depth: false,
                double_sided: mat.double_sided,
            });
            rp.set_pipeline(pipeline);
            rp.set_bind_group(2, &mat.bg, &[]);
            rp.draw_indexed(offset..offset + length, 0, 0..self.n_instances);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        rp.set_bind_group(0, proj, &[]);
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, offset, length) in self.mesh.iter_materials() {
            let mat = gfx.material(mat);
            rp.set_pipeline(gfx.get_pipeline(InstancedMeshPipeline {
                alpha: mat.transparent,
                smap: shadow_map,
                depth: true,
                double_sided: mat.double_sided,
            }));

            if mat.transparent {
                rp.set_bind_group(1, &mat.bg, &[]);
            }
            rp.draw_indexed(offset..offset + length, 0, 0..self.n_instances);
        }
    }
}
