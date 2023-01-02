use crate::{GfxContext, IndexType, Mesh, MeshBuilder, MeshVertex, TextureBuilder};
use geom::{Matrix4, Quaternion, Vec2, Vec3};
use gltf::image::Format;
use gltf::json::texture::{MagFilter, MinFilter};
use image::{DynamicImage, ImageBuffer};
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use wgpu::FilterMode;

#[derive(Debug)]
pub enum ImageLoadError {
    InvalidFormat(Format),
    InvalidData,
}

pub fn load_image(
    gfx: &GfxContext,
    data: gltf::image::Data,
    sampl: gltf::texture::Sampler,
) -> Result<Arc<crate::Texture>, ImageLoadError> {
    let hash = common::hash_u64((
        &data.pixels,
        data.width,
        data.height,
        sampl.min_filter().map(|x| x.as_gl_enum()),
        sampl.mag_filter().map(|x| x.as_gl_enum()),
    ));

    let mut cache = gfx.texture_cache_bytes.lock().unwrap();

    let ent = cache.entry(hash);

    let ent = match ent {
        Entry::Occupied(ent) => {
            return Ok(ent.get().clone());
        }
        Entry::Vacant(v) => v,
    };

    let w = data.width;
    let h = data.height;
    let d = data.pixels;
    let albedo_img = match data.format {
        Format::R8 => DynamicImage::ImageLuma8(
            ImageBuffer::from_raw(w, h, d).ok_or(ImageLoadError::InvalidData)?,
        ),
        Format::R8G8 => DynamicImage::ImageLumaA8(
            ImageBuffer::from_raw(w, h, d).ok_or(ImageLoadError::InvalidData)?,
        ),
        Format::R8G8B8 => DynamicImage::ImageRgb8(
            ImageBuffer::from_raw(w, h, d).ok_or(ImageLoadError::InvalidData)?,
        ),
        Format::R8G8B8A8 => DynamicImage::ImageRgba8(
            ImageBuffer::from_raw(w, h, d).ok_or(ImageLoadError::InvalidData)?,
        ),
        f => {
            return Err(ImageLoadError::InvalidFormat(f));
        }
    };

    let (min, mipmap) = sampl
        .min_filter()
        .map(|x| {
            use MinFilter::*;
            match x {
                Nearest | NearestMipmapLinear => (FilterMode::Nearest, FilterMode::Linear),
                Linear | LinearMipmapLinear => (FilterMode::Linear, FilterMode::Linear),
                NearestMipmapNearest => (FilterMode::Nearest, FilterMode::Nearest),
                LinearMipmapNearest => (FilterMode::Linear, FilterMode::Nearest),
            }
        })
        .unwrap_or_default();

    let sampler = wgpu::SamplerDescriptor {
        label: Some("mesh sampler"),
        address_mode_u: Default::default(),
        address_mode_v: Default::default(),
        address_mode_w: Default::default(),
        mag_filter: sampl
            .mag_filter()
            .map(|x| match x {
                MagFilter::Nearest => FilterMode::Nearest,
                MagFilter::Linear => FilterMode::Linear,
            })
            .unwrap_or_default(),
        min_filter: min,
        mipmap_filter: mipmap,
        ..Default::default()
    };

    let tex = Arc::new(
        TextureBuilder::from_img(albedo_img)
            .with_label("some material albedo")
            .with_sampler(sampler)
            .build(&gfx.device, &gfx.queue),
    );

    Ok(ent.insert(tex).clone())
}

#[derive(Debug)]
pub enum LoadMeshError {
    GltfLoadError(gltf::Error),
    NotSingleMaterial(usize),
    NoIndices,
    NoVertices,
    NoBaseColorTexture,
    ImageNotFound,
    InvalidImage(ImageLoadError),
}

pub fn load_mesh(asset_name: &str, gfx: &GfxContext) -> Result<Mesh, LoadMeshError> {
    let mut path = PathBuf::new();
    path.push("assets/models/");
    path.push(asset_name);

    let t = Instant::now();

    let mut flat_vertices: Vec<MeshVertex> = vec![];
    let mut indices = vec![];

    let (doc, data, images) = gltf::import(&path).map_err(|e| LoadMeshError::GltfLoadError(e))?;

    let nodes = doc.nodes();

    if doc.materials().len() != 1 {
        return Err(LoadMeshError::NotSingleMaterial(doc.materials().len()));
    }

    for node in nodes {
        let mesh = unwrap_cont!(node.mesh());
        let translation = node.transform();
        let rot_qat = Quaternion::from(translation.clone().decomposed().1);
        let mat = Matrix4::from(translation.matrix());
        let invert_winding = mat.determinent() < 0.0;

        let primitive = unwrap_cont!(mesh.primitives().next());
        let reader = primitive.reader(|b| Some(&data.get(b.index())?.0[..b.length()]));

        let positions = unwrap_cont!(reader.read_positions()).map(Vec3::from);
        let normals = unwrap_cont!(reader.read_normals()).map(Vec3::from);
        let uv = unwrap_cont!(reader.read_tex_coords(0))
            .into_f32()
            .map(Vec2::from);
        let read_indices: Vec<u32> = unwrap_cont!(reader.read_indices()).into_u32().collect();

        let raw: Vec<_> = positions
            .zip(normals)
            .zip(uv)
            .map(|((p, n), uv)| {
                let pos = mat * p.w(1.0);
                let pos = pos.xyz() / pos.w;
                (pos, rot_qat * n, uv)
            })
            .collect();

        if raw.is_empty() {
            continue;
        }

        let shade_smooth = true;

        let vtx_offset = flat_vertices.len() as IndexType;
        if shade_smooth {
            for (pos, normal, uv) in &raw {
                flat_vertices.push(MeshVertex {
                    position: pos.into(),
                    normal: *normal,
                    uv: (*uv).into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                })
            }
        }

        for triangle in read_indices.chunks_exact(3) {
            let (mut a, b, mut c) = if let [a, b, c] = *triangle {
                (a, b, c)
            } else {
                continue;
            };

            if invert_winding {
                std::mem::swap(&mut a, &mut c);
            }

            if shade_smooth {
                indices.push(vtx_offset + a as IndexType);
                indices.push(vtx_offset + b as IndexType);
                indices.push(vtx_offset + c as IndexType);
                continue;
            }

            let a = raw[a as usize];
            let b = raw[b as usize];
            let c = raw[c as usize];

            let t_normal = (a.1 + b.1 + c.1) / 3.0;

            let mk_v = |p: Vec3, u: Vec2| MeshVertex {
                position: p.into(),
                normal: t_normal,
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
    }

    if indices.is_empty() {
        return Err(LoadMeshError::NoIndices);
    }

    let mat = doc.materials().next().unwrap();
    let tex = unwrap_or!(mat.pbr_metallic_roughness().base_color_texture(), {
        return Err(LoadMeshError::NoBaseColorTexture);
    })
    .texture();

    //    let sampler = tex.sampler().mag_filter().unwrap()
    let data = unwrap_or!(images.into_iter().nth(tex.source().index()), {
        return Err(LoadMeshError::ImageNotFound);
    });

    let mut meshb = MeshBuilder::new();
    meshb.vertices = flat_vertices;
    meshb.indices = indices;

    let albedo =
        load_image(gfx, data, tex.sampler()).map_err(|e| LoadMeshError::InvalidImage(e))?;

    let m = meshb.build(gfx, albedo).ok_or(LoadMeshError::NoVertices)?;

    log::info!(
        "loaded mesh {:?} in {}ms",
        path,
        1000.0 * t.elapsed().as_secs_f32()
    );

    Ok(m)
}
