use crate::{
    GfxContext, IndexType, Material, Mesh, MeshBuilder, MeshVertex, MetallicRoughness,
    TextureBuilder,
};
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
    srgb: bool,
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
    let img = match data.format {
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

    let mag = sampl
        .mag_filter()
        .map(|x| {
            use MagFilter::*;
            match x {
                Nearest => FilterMode::Nearest,
                Linear => FilterMode::Linear,
            }
        })
        .unwrap_or_default();

    let sampler = wgpu::SamplerDescriptor {
        label: Some("mesh sampler"),
        address_mode_u: Default::default(),
        address_mode_v: Default::default(),
        address_mode_w: Default::default(),
        mag_filter: mag,
        min_filter: min,
        mipmap_filter: mipmap,
        ..Default::default()
    };

    let tex = Arc::new(
        TextureBuilder::from_img(img)
            .with_label("some material albedo")
            .with_sampler(sampler)
            .with_mipmaps(gfx.mipmap_module())
            .with_srgb(srgb)
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

pub fn load_mesh(gfx: &mut GfxContext, asset_name: &str) -> Result<Mesh, LoadMeshError> {
    let mut path = PathBuf::new();
    path.push("assets/models/");
    path.push(asset_name);

    let t = Instant::now();

    let mut flat_vertices: Vec<MeshVertex> = vec![];
    let mut indices = vec![];

    let (doc, data, mut images) = gltf::import(&path).map_err(LoadMeshError::GltfLoadError)?;

    let nodes = doc.nodes();

    if doc.materials().len() != 1 {
        return Err(LoadMeshError::NotSingleMaterial(doc.materials().len()));
    }

    for node in nodes {
        let mesh = unwrap_cont!(node.mesh());
        let transform = node.transform();
        let rot_qat = Quaternion::from(transform.clone().decomposed().1);
        let transform_mat = Matrix4::from(transform.matrix());

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
                let pos = transform_mat * p.w(1.0);
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

        for &[a, b, c] in bytemuck::cast_slice::<u32, [u32; 3]>(&read_indices) {
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
    let pbr_mr = mat.pbr_metallic_roughness();
    let albedo_tex = unwrap_or!(pbr_mr.base_color_texture(), {
        return Err(LoadMeshError::NoBaseColorTexture);
    })
    .texture();
    let metallic_v = pbr_mr.metallic_factor();
    let roughness_v = pbr_mr.roughness_factor();

    let mut metallic_roughness = MetallicRoughness::Static {
        metallic: metallic_v,
        roughness: roughness_v,
    };

    if let Some(metallic_roughness_tex) = pbr_mr.metallic_roughness_texture() {
        let metallic_roughness_tex = metallic_roughness_tex.texture();
        let idx = metallic_roughness_tex.source().index();
        if idx > images.len() {
            return Err(LoadMeshError::ImageNotFound);
        }
        let metallic_roughness_data = std::mem::replace(
            &mut images[metallic_roughness_tex.source().index()],
            gltf::image::Data {
                pixels: vec![],
                format: Format::R8,
                width: 0,
                height: 0,
            },
        );

        let tex = load_image(
            gfx,
            metallic_roughness_data,
            metallic_roughness_tex.sampler(),
            false,
        )
        .map_err(LoadMeshError::InvalidImage)?;
        metallic_roughness = MetallicRoughness::Texture(tex);
    }

    //    let sampler = tex.sampler().mag_filter().unwrap()
    let idx = albedo_tex.source().index();
    if idx > images.len() {
        return Err(LoadMeshError::ImageNotFound);
    }
    let albedo_data = std::mem::replace(
        &mut images[albedo_tex.source().index()],
        gltf::image::Data {
            pixels: vec![],
            format: Format::R8,
            width: 0,
            height: 0,
        },
    );

    let albedo = load_image(gfx, albedo_data, albedo_tex.sampler(), true)
        .map_err(LoadMeshError::InvalidImage)?;
    let transparent = albedo.transparent;

    let matid = gfx.register_material(Material::new(gfx, albedo, metallic_roughness));

    let mut meshb = MeshBuilder::new(matid);
    meshb.vertices = flat_vertices;
    meshb.indices = indices;
    let mut m = meshb.build(gfx).ok_or(LoadMeshError::NoVertices)?;
    m.transparent = transparent;
    m.double_sided = mat.double_sided();

    log::info!(
        "loaded mesh {:?} in {}ms ({} tris{})",
        path,
        1000.0 * t.elapsed().as_secs_f32(),
        m.n_indices / 3,
        if m.double_sided { ", double sided" } else { "" }
    );

    Ok(m)
}
