use crate::pbuffer::PBuffer;
use crate::{GfxContext, IndexType, MaterialID, Mesh, MeshVertex, MikktGeometry, Tesselator};
use geom::{Sphere, Vec3, AABB3};
use std::ops::Range;
use wgpu::BufferUsages;

pub struct MeshBuilder<const PERSISTENT: bool> {
    vertices: Vec<MeshVertex>,
    indices: Vec<IndexType>,
    vi_buffers: Option<Box<(PBuffer, PBuffer)>>,
    /// List of materialID and the starting offset
    lods: Vec<MeshLod>,
    current_lod: usize,
    default_mat: Option<MaterialID>,
}

struct MikktGenerate<'a> {
    vertices: &'a mut [MeshVertex],
    indices: &'a [IndexType],
}

impl<'a> MikktGeometry for MikktGenerate<'a> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let i = self.indices[face * 3 + vert] as usize;
        self.vertices[i].position
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let i = self.indices[face * 3 + vert] as usize;
        self.vertices[i].normal.into()
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let i = self.indices[face * 3 + vert] as usize;
        self.vertices[i].uv
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let i = self.indices[face * 3 + vert] as usize;
        self.vertices[i].tangent = tangent;
    }
}

impl<const PERSISTENT: bool> MeshBuilder<PERSISTENT> {
    pub fn new(default_mat: MaterialID) -> Self {
        Self {
            default_mat: Some(default_mat),
            ..Self::new_without_mat()
        }
    }

    pub fn new_without_mat() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            vi_buffers: PERSISTENT.then(|| {
                Box::new((
                    PBuffer::new(BufferUsages::VERTEX),
                    PBuffer::new(BufferUsages::INDEX),
                ))
            }),
            lods: vec![MeshLod::default()],
            current_lod: 0,
            default_mat: None,
        }
    }

    pub fn mk_tess(&mut self) -> Tesselator {
        Tesselator::new(&mut self.vertices, &mut self.indices, None, 1.0)
    }

    pub fn lods(&self) -> &[MeshLod] {
        &self.lods
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.lods.clear();
        self.lods.push(MeshLod::default());
    }

    pub fn extend(
        &mut self,
        mat: Option<MaterialID>,
        vertices: &[MeshVertex],
        indices: &[IndexType],
    ) -> &mut Self {
        self.set_material(mat);
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self.lods[self.current_lod].n_vertices += vertices.len();
        self.lods[self.current_lod].n_indices += indices.len();
        self.finish_material();
        self
    }

    /// Sets the material for all future indice pushes
    fn set_material(&mut self, material: Option<MaterialID>) {
        let Some(material) = material.or(self.default_mat) else {
            return;
        };
        let n = self.indices.len() as u32;
        let primitives = &mut self.lods[self.current_lod].primitives;
        if let Some(previous) = primitives.last_mut() {
            if previous.0 == material && previous.1.end == n {
                return;
            }
        }
        primitives.push((material, n..n));
    }

    /// Finishes the current material
    fn finish_material(&mut self) {
        let n = self.indices.len() as u32;
        let primitives = &mut self.lods[self.current_lod].primitives;
        if let Some(previous) = primitives.last_mut() {
            previous.1.end = n;
        } else if let Some(default_mat) = self.default_mat {
            if n > 0 {
                primitives.push((default_mat, 0..n));
            }
        }
    }

    /// new_lod indicates that all future vertex/index/material pushes will be in a new lod
    pub fn set_lod(&mut self, lod_level: usize, coverage: f64) {
        if self.lods.len() <= lod_level {
            self.lods
                .extend((self.lods.len()..=lod_level).map(|_| MeshLod::default()));
        }
        self.current_lod = lod_level;
        self.lods[self.current_lod].screen_coverage += coverage as f32;
    }

    /// Sets the bounds for the current lod
    pub fn set_bounds(&mut self, bounds: AABB3) {
        let aabb3 = &mut self.lods[self.current_lod].aabb3;
        if aabb3.ll == Vec3::ZERO && aabb3.ur == Vec3::ZERO {
            *aabb3 = bounds;
            return;
        }
        *aabb3 = aabb3.union(bounds);
    }

    #[inline(always)]
    pub fn extend_with(
        &mut self,
        mat: Option<MaterialID>,
        f: impl FnOnce(&mut Vec<MeshVertex>, &mut dyn FnMut(IndexType)),
    ) {
        self.set_material(mat);
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let n_vertices = vertices.len();
        let n_indices = indices.len();
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
        self.finish_material();
        self.lods[self.current_lod].n_vertices += self.vertices.len() - n_vertices;
        self.lods[self.current_lod].n_indices += self.indices.len() - n_indices;
    }

    pub fn compute_tangents(&mut self) {
        for lod in &mut self.lods {
            for (_, range) in &lod.primitives {
                let mut mg = MikktGenerate {
                    vertices: &mut self.vertices,
                    indices: &self.indices[range.start as usize..range.end as usize],
                };
                if !crate::geometry::generate_tangents(&mut mg) {
                    log::warn!("failed to generate tangents");
                }
            }
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<Mesh> {
        if self.vertices.is_empty() {
            return None;
        }
        self.finish_material();

        let mut tmpv;
        let mut tmpi;
        let vbuffer;
        let ibuffer;

        if PERSISTENT {
            let x = self.vi_buffers.as_deref_mut().unwrap();
            vbuffer = &mut x.0;
            ibuffer = &mut x.1;
        } else {
            tmpv = PBuffer::new(BufferUsages::VERTEX);
            tmpi = PBuffer::new(BufferUsages::INDEX);
            vbuffer = &mut tmpv;
            ibuffer = &mut tmpi;
        }

        vbuffer.write(gfx, bytemuck::cast_slice(&self.vertices));
        ibuffer.write(gfx, bytemuck::cast_slice(&self.indices));

        // Compute AABB3 for each lod
        for lod in &mut self.lods {
            if lod.aabb3.ll != Vec3::ZERO || lod.aabb3.ur != Vec3::ZERO {
                continue;
            }
            let mut aabb3 = AABB3::zero();
            for (_, range) in &lod.primitives {
                for &idx in &self.indices[range.start as usize..range.end as usize] {
                    aabb3 = aabb3.union_vec(self.vertices[idx as usize].position.into());
                }
            }
            lod.aabb3 = aabb3;
        }

        for lod in &mut self.lods {
            lod.bounding_sphere = lod.aabb3.bounding_sphere();
        }

        Some(Mesh {
            vertex_buffer: vbuffer.inner()?,
            index_buffer: ibuffer.inner()?,
            lods: self.lods.clone().into_boxed_slice(),
            skip_depth: false,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct MeshLod {
    /// List of materialID and the index range
    pub primitives: Vec<(MaterialID, Range<u32>)>,
    /// Percentage of vertical space the mesh takes up on screen before it switches to the next lod
    pub screen_coverage: f32,
    pub aabb3: AABB3,
    pub bounding_sphere: Sphere,
    pub n_vertices: usize,
    pub n_indices: usize,
}

impl MeshLod {
    pub fn draw_calls(&self) -> usize {
        self.primitives.len()
    }

    pub fn n_vertices(&self) -> usize {
        self.n_vertices
    }

    pub fn n_indices(&self) -> usize {
        self.n_indices
    }

    #[inline]
    pub fn passes_culling(&self, gfx: &GfxContext) -> bool {
        let screen_area = crate::screen_coverage(gfx, self.bounding_sphere);
        screen_area >= self.screen_coverage
    }
}
