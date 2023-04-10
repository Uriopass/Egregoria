use crate::pbuffer::PBuffer;
use crate::{
    Drawable, GfxContext, IndexType, Material, MaterialID, MeshInstance, MeshVertex, RenderParams,
    Texture, Uniform, TL,
};
use smallvec::SmallVec;
use std::sync::Arc;
use wgpu::{BindGroupLayout, BufferUsages, Device, IndexFormat, RenderPass, VertexBufferLayout};

pub struct MeshBuilder {
    pub(crate) vertices: Vec<MeshVertex>,
    pub(crate) indices: Vec<IndexType>,
    pub(crate) vbuffer: PBuffer,
    pub(crate) ibuffer: PBuffer,
    /// List of materialID and the starting offset
    pub(crate) materials: SmallVec<[(MaterialID, u32); 1]>,
}

impl MeshBuilder {
    pub fn new(mat: MaterialID) -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            vbuffer: PBuffer::new(BufferUsages::VERTEX),
            ibuffer: PBuffer::new(BufferUsages::INDEX),
            materials: smallvec::smallvec![(mat, 0)],
        }
    }

    pub fn new_without_mat() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            vbuffer: PBuffer::new(BufferUsages::VERTEX),
            ibuffer: PBuffer::new(BufferUsages::INDEX),
            materials: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn extend(&mut self, vertices: &[MeshVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    /// Sets the material for all future indice pushes
    pub fn set_material(&mut self, material: MaterialID) {
        let n = self.indices.len() as u32;
        self.materials.push((material, n));
    }

    #[inline(always)]
    pub fn extend_with(&mut self, f: impl FnOnce(&mut Vec<MeshVertex>, &mut dyn FnMut(IndexType))) {
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<Mesh> {
        if self.vertices.is_empty() {
            return None;
        }

        self.vbuffer
            .write(gfx, bytemuck::cast_slice(&self.vertices));
        self.ibuffer.write(gfx, bytemuck::cast_slice(&self.indices));

        // convert materials to mesh format (from offsets to lengths)
        let mut materials = SmallVec::with_capacity(self.materials.len());
        let mut mats = self.materials.iter().peekable();
        while let Some((mat, start)) = mats.next() {
            let end = mats
                .peek()
                .map(|(_, x)| *x)
                .unwrap_or(self.indices.len() as u32);
            let l = end - start;
            if l == 0 {
                continue;
            }
            materials.push((*mat, l));
        }

        Some(Mesh {
            vertex_buffer: self.vbuffer.inner()?,
            index_buffer: self.ibuffer.inner()?,
            materials,
            skip_depth: false,
        })
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    /// List of materialID and the indice length
    pub materials: SmallVec<[(MaterialID, u32); 1]>,
    pub skip_depth: bool,
}

impl Mesh {
    /// Returns an iterator over the materials used by this mesh
    /// The iterator returns the materialID, the index offset and the number of indices for that material
    pub fn iter_materials(&self) -> impl Iterator<Item = (MaterialID, u32, u32)> + '_ {
        let mut offset = 0;
        self.materials.iter().map(move |(mat, n)| {
            let ret = (*mat, offset, *n);
            offset += *n;
            ret
        })
    }
}

#[derive(Clone, Copy, Hash)]
pub(crate) struct MeshPipeline {
    pub(crate) instanced: bool,
    pub(crate) alpha: bool,
    pub(crate) smap: bool,
    pub(crate) depth: bool,
    pub(crate) double_sided: bool,
}

const VB_INSTANCED: &[VertexBufferLayout] = &[MeshVertex::desc(), MeshInstance::desc()];
const VB: &[VertexBufferLayout] = &[MeshVertex::desc()];

impl Mesh {
    pub fn setup(gfx: &mut GfxContext) {
        for instanced in [false, true] {
            let vert_shader = if instanced {
                "instanced_mesh.vert"
            } else {
                "lit_mesh.vert"
            };

            let vb: &[VertexBufferLayout] = if instanced { VB_INSTANCED } else { VB };

            for double_sided in [false, true] {
                let pipeline = MeshPipeline {
                    instanced,
                    double_sided,
                    alpha: false,
                    smap: false,
                    depth: false,
                };

                gfx.register_pipeline(
                    pipeline,
                    &[vert_shader, "pixel.frag"],
                    Box::new(move |m, gfx| {
                        let vert = &m[0];
                        let frag = &m[1];
                        gfx.color_pipeline(
                            "lit_mesh",
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
                    let pipeline_depth = MeshPipeline {
                        instanced,
                        smap,
                        double_sided,
                        alpha: false,
                        depth: true,
                    };
                    gfx.register_pipeline(
                        pipeline_depth,
                        &[vert_shader],
                        Box::new(move |m, gfx| {
                            let vert = &m[0];
                            gfx.depth_pipeline(vb, vert, None, smap, double_sided)
                        }),
                    );

                    let pipeline_depth_alpha = MeshPipeline {
                        instanced,
                        smap,
                        double_sided,
                        alpha: true,
                        depth: true,
                    };
                    gfx.register_pipeline(
                        pipeline_depth_alpha,
                        &[vert_shader, "alpha_discard.frag"],
                        Box::new(move |m, gfx| {
                            let vert = &m[0];
                            let frag = &m[1];
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
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, offset, length) in self.iter_materials() {
            let mat = gfx.material(mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                instanced: false,
                alpha: false,
                smap: false,
                depth: false,
                double_sided: mat.double_sided,
            }));
            rp.set_bind_group(2, &mat.bg, &[]);
            rp.draw_indexed(offset..offset + length, 0, 0..1);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        if self.skip_depth {
            return;
        }
        rp.set_bind_group(0, proj, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, offset, length) in self.iter_materials() {
            let mat = gfx.material(mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                instanced: false,
                alpha: mat.transparent,
                smap: shadow_map,
                depth: true,
                double_sided: mat.double_sided,
            }));

            if mat.transparent {
                rp.set_bind_group(1, &mat.bg, &[]);
            }
            rp.draw_indexed(offset..offset + length, 0, 0..1);
        }
    }
}

pub struct LitMeshDepth;
pub struct LitMeshDepthSMap;

pub fn bg_layout_litmesh(device: &Device) -> BindGroupLayout {
    Texture::bindgroup_layout(
        device,
        [
            TL::Float,
            TL::Float,
            TL::DepthArray,
            TL::Cube,
            TL::Cube,
            TL::Float,
        ],
    )
}
