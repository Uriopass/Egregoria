use crate::{GfxContext, IndexType, Mesh, MeshBuilder, MeshVertex, Texture};
use geom::{vec2, vec3, Vec2, Vec3};
use std::path::Path;
use std::sync::Arc;

pub fn obj_to_mesh(
    path: impl AsRef<Path> + std::fmt::Debug,
    gfx: &GfxContext,
    albedo: Arc<Texture>,
) -> Option<Mesh> {
    let (models, _) = tobj::load_obj(path, true)
        .map_err(|e| log::error!("{}", e))
        .ok()?;
    let model = models.first()?;

    if model.mesh.normals.is_empty() {
        return None;
    }

    let mut raw = vec![];

    let positions = model.mesh.positions.chunks_exact(3);
    let normals = model.mesh.normals.chunks_exact(3);
    let uv = model.mesh.texcoords.chunks_exact(2);

    for ((p, n), uv) in positions.zip(normals).zip(uv) {
        raw.push((
            vec3(p[0], p[1], p[2]),
            vec3(n[0], n[1], n[2]),
            vec2(uv[0], 1.0 - uv[1]),
        ));
    }

    let mut flat_vertices: Vec<MeshVertex> = vec![];
    let mut indices = vec![];

    for triangle in model.mesh.indices.chunks(3) {
        let a = raw[triangle[0] as usize];
        let b = raw[triangle[1] as usize];
        let c = raw[triangle[2] as usize];

        let t_normal = (a.1 + b.1 + c.1) / 3.0;

        let mk_v = |p: Vec3, u: Vec2| MeshVertex {
            position: p.into(),
            normal: t_normal.into(),
            uv: u.into(),
            color: [1.0, 1.0, 1.0, 1.0],
        };

        indices.push(flat_vertices.len() as IndexType);
        flat_vertices.push(mk_v(a.0, a.2));

        indices.push(flat_vertices.len() as IndexType);
        flat_vertices.push(mk_v(b.0, b.2));

        indices.push(flat_vertices.len() as IndexType);
        flat_vertices.push(mk_v(c.0, c.2));
    }

    let mut b = MeshBuilder::new();
    b.vertices = flat_vertices;
    b.indices = indices;

    b.build(gfx, albedo)
}
