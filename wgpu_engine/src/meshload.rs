use crate::{
    GfxContext, IndexType, Material, MaterialID, Mesh, MeshBuilder, MeshVertex, MetallicRoughness,
    Texture, TextureBuilder,
};
use geom::{Color, LinearColor, Matrix4, Quaternion, Vec2, Vec3};
use gltf::image::{Data, Format};
use gltf::json::texture::{MagFilter, MinFilter};
use gltf::texture::WrappingMode;
use gltf::Document;
use image::{DynamicImage, ImageBuffer};
use smallvec::SmallVec;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{AddressMode, FilterMode};

#[derive(Debug)]
pub enum ImageLoadError {
    InvalidFormat(Format),
    InvalidData,
}

pub fn load_image(
    gfx: &GfxContext,
    data: Data,
    sampl: gltf::texture::Sampler,
    srgb: bool,
) -> Result<Arc<Texture>, ImageLoadError> {
    let hash = common::hash_u64((
        &data.pixels,
        data.width,
        data.height,
        sampl.min_filter().map(|x| x.as_gl_enum()),
        sampl.mag_filter().map(|x| x.as_gl_enum()),
        sampl.wrap_s().as_gl_enum(),
        sampl.wrap_t().as_gl_enum(),
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

    let wrap_s = match sampl.wrap_s() {
        WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
        WrappingMode::Repeat => AddressMode::Repeat,
    };

    let wrap_t = match sampl.wrap_t() {
        WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
        WrappingMode::Repeat => AddressMode::Repeat,
    };

    let sampler = wgpu::SamplerDescriptor {
        label: Some("mesh sampler"),
        address_mode_u: wrap_s,
        address_mode_v: wrap_t,
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

fn load_materials(
    gfx: &mut GfxContext,
    doc: &Document,
    images: &mut Vec<Data>,
) -> Result<Vec<MaterialID>, LoadMeshError> {
    let mut v = Vec::with_capacity(doc.materials().len());
    for gltfmat in doc.materials() {
        let pbr_mr = gltfmat.pbr_metallic_roughness();

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
                Data {
                    pixels: vec![],
                    format: Format::R8,
                    width: 0xDEAD,
                    height: 0,
                },
            );

            if metallic_roughness_data.pixels.is_empty() && metallic_roughness_data.width == 0xDEAD
            {
                return Err(LoadMeshError::ImageAlreadyUsed);
            }

            let tex = load_image(
                gfx,
                metallic_roughness_data,
                metallic_roughness_tex.sampler(),
                false,
            )
            .map_err(LoadMeshError::InvalidImage)?;
            metallic_roughness = MetallicRoughness::Texture(tex);
        }

        let albedo;
        if let Some(albedo_tex) = pbr_mr.base_color_texture() {
            let albedo_tex = albedo_tex.texture();
            let idx = albedo_tex.source().index();
            if idx > images.len() {
                return Err(LoadMeshError::ImageNotFound);
            }
            let albedo_data = std::mem::replace(
                &mut images[albedo_tex.source().index()],
                Data {
                    pixels: vec![],
                    format: Format::R8,
                    width: 0xDEAD,
                    height: 0,
                },
            );

            if albedo_data.pixels.is_empty() && albedo_data.width == 0xDEAD {
                return Err(LoadMeshError::ImageAlreadyUsed);
            }

            albedo = load_image(gfx, albedo_data, albedo_tex.sampler(), true)
                .map_err(LoadMeshError::InvalidImage)?;
        } else {
            let v: LinearColor = LinearColor::from(pbr_mr.base_color_factor());
            let srgb: Color = v.into();
            albedo = Arc::new(
                TextureBuilder::from_img(DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                    1,
                    1,
                    image::Rgba::<u8>::from([
                        (srgb.r * 255.0).round() as u8,
                        (srgb.g * 255.0).round() as u8,
                        (srgb.b * 255.0).round() as u8,
                        (srgb.a * 255.0).round() as u8,
                    ]),
                )))
                .with_srgb(true)
                .with_label("some material albedo 1x1")
                .with_sampler(Texture::nearest_sampler())
                .build(&gfx.device, &gfx.queue),
            );
        }
        let transparent = albedo.transparent;
        let mut gfxmat = Material::new(gfx, albedo, metallic_roughness);
        gfxmat.transparent = transparent;
        gfxmat.double_sided = gltfmat.double_sided();
        let matid = gfx.register_material(gfxmat);
        v.push(matid)
    }
    debug_assert_eq!(v.len(), doc.materials().len());
    Ok(v)
}

#[derive(Debug)]
pub enum LoadMeshError {
    GltfLoadError(gltf::Error),
    /// Mesh doesn't have a material
    NoMaterial,
    NoIndices,
    NoVertices,
    ImageNotFound,
    /// Two materials refer to the same texture, not supported for now because of srgb handling
    ImageAlreadyUsed,
    InvalidImage(ImageLoadError),
}

pub fn load_mesh(gfx: &mut GfxContext, asset_name: &str) -> Result<Mesh, LoadMeshError> {
    let mut path = PathBuf::new();
    path.push("assets/models/");
    path.push(asset_name);

    let t = Instant::now();

    let mut flat_vertices: Vec<MeshVertex> = vec![];
    let mut indices = vec![];
    let mut materials_idx = SmallVec::new();

    let (doc, data, mut images) = gltf::import(&path).map_err(LoadMeshError::GltfLoadError)?;

    let exts = doc
        .extensions_used()
        .fold(String::new(), |a, b| a + ", " + b);
    if !exts.is_empty() {
        log::warn!("extension not supported: {}", exts)
    }
    let nodes = doc.nodes();

    let mats = load_materials(gfx, &doc, &mut images)?;

    for node in nodes {
        let mesh = unwrap_cont!(node.mesh());
        let transform = node.transform();
        let rot_qat = Quaternion::from(transform.clone().decomposed().1);
        let transform_mat = Matrix4::from(transform.matrix());

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| Some(&data.get(b.index())?.0[..b.length()]));
            let matid = primitive
                .material()
                .index()
                .ok_or(LoadMeshError::NoMaterial)?;

            materials_idx.push((mats[matid], indices.len() as u32));

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
    }

    if indices.is_empty() {
        return Err(LoadMeshError::NoIndices);
    }

    let mut meshb = MeshBuilder::new_without_mat();
    meshb.vertices = flat_vertices;
    meshb.indices = indices;
    meshb.materials = materials_idx;
    let m = meshb.build(gfx).ok_or(LoadMeshError::NoVertices)?;

    log::info!(
        "loaded mesh {:?} in {}ms ({} tris{})",
        path,
        1000.0 * t.elapsed().as_secs_f32(),
        m.materials.iter().map(|x| x.1).sum::<u32>() / 3,
        if m.materials.iter().any(|x| gfx.material(x.0).double_sided) {
            ", double sided"
        } else {
            ""
        }
    );

    Ok(m)
}
