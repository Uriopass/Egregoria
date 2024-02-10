use crate::meshbuild::MeshBuilder;
use crate::{
    GfxContext, IndexType, Material, MaterialID, Mesh, MeshVertex, MetallicRoughness, Texture,
    TextureBuilder,
};
use geom::{Color, LinearColor, Matrix4, Quaternion, Vec2, Vec3, AABB3};
use gltf::buffer::Source;
use gltf::image::{Data, Format};
use gltf::json::texture::{MagFilter, MinFilter};
use gltf::texture::WrappingMode;
use gltf::{Document, Node, Scene};
use image::{DynamicImage, ImageBuffer};
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use wgpu::{AddressMode, FilterMode};

#[derive(Clone, Debug)]
pub enum ImageLoadError {
    InvalidFormat(Format),
    InvalidData,
    ImageNotFound,
}

pub fn load_image(
    gfx: &GfxContext,
    matname: Option<&str>,
    tex: &gltf::Texture,
    images: &[Data],
    srgb: bool,
) -> Result<Arc<Texture>, ImageLoadError> {
    let idx = tex.source().index();
    if idx > images.len() {
        return Err(ImageLoadError::ImageNotFound);
    }
    let data = images[tex.source().index()].clone();

    let sampl = tex.sampler();

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
            .with_label(tex.name().or(matname).unwrap_or("mesh texture"))
            .with_sampler(sampler)
            .with_mipmaps(&gfx.mipmap_gen)
            .with_srgb(srgb)
            .build(&gfx.device, &gfx.queue),
    );

    Ok(ent.insert(tex).clone())
}

fn load_materials(
    gfx: &mut GfxContext,
    doc: &Document,
    images: &[Data],
) -> Result<(Vec<MaterialID>, bool), LoadMeshError> {
    let mut v = Vec::with_capacity(doc.materials().len());
    let mut needs_tangents = false;
    for gltfmat in doc.materials() {
        let pbr_mr = gltfmat.pbr_metallic_roughness();

        let metallic_v = pbr_mr.metallic_factor();
        let roughness_v = pbr_mr.roughness_factor();

        let mut metallic_roughness = MetallicRoughness {
            metallic: metallic_v,
            roughness: roughness_v,
            tex: None,
        };

        if let Some(metallic_roughness_tex) = pbr_mr.metallic_roughness_texture() {
            metallic_roughness.tex = Some(load_image(
                gfx,
                gltfmat.name(),
                &metallic_roughness_tex.texture(),
                images,
                false,
            )?);
        }

        let mut normal = None;
        if let Some(normal_tex) = gltfmat.normal_texture() {
            normal = Some(load_image(
                gfx,
                gltfmat.name(),
                &normal_tex.texture(),
                images,
                false,
            )?);
            needs_tangents = true;
        }

        let albedo;
        if let Some(albedo_tex) = pbr_mr.base_color_texture() {
            albedo = load_image(gfx, gltfmat.name(), &albedo_tex.texture(), images, true)?;
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
                .with_label(&format!("{}: albedo 1x1", gltfmat.name().unwrap_or("mat")))
                .with_sampler(Texture::nearest_sampler())
                .build(&gfx.device, &gfx.queue),
            );
        }
        let transparent = albedo.transparent;
        let mut gfxmat = Material::new(gfx, &albedo, metallic_roughness, normal.as_deref());
        gfxmat.transparent = transparent;
        let matid = gfx.register_material(gfxmat);
        v.push(matid)
    }
    debug_assert_eq!(v.len(), doc.materials().len());
    Ok((v, needs_tangents))
}

#[derive(Clone, Debug)]
pub enum LoadMeshError {
    GltfLoadError(Arc<gltf::Error>),
    /// Mesh doesn't have a material
    NoMaterial,
    NoIndices,
    NoVertices,
    InvalidImage(ImageLoadError),
    NoDefaultScene,
}

impl From<ImageLoadError> for LoadMeshError {
    fn from(value: ImageLoadError) -> Self {
        LoadMeshError::InvalidImage(value)
    }
}

#[derive(Clone)]
pub struct CPUMesh {
    pub n_textures: usize,
    pub n_triangles: usize,
    pub gltf_doc: Document,
    pub gltf_data: Vec<gltf::buffer::Data>,
    pub asset_path: PathBuf,
}

pub fn glb_buffer_id(document: &Document) -> usize {
    document
        .buffers()
        .enumerate()
        .find(|(_, b)| matches!(b.source(), Source::Bin))
        .unwrap()
        .0
}

fn mat_rot(node: &Node) -> (Matrix4, Quaternion) {
    let transform = node.transform();
    let rot_qat = Quaternion::from(transform.clone().decomposed().1);
    let transform_mat = Matrix4::from(transform.matrix());
    (transform_mat, rot_qat)
}

/// Returns a list of nodes, their LOD level, screen coverage and their global transforms
pub fn find_nodes<'a>(
    scene: &'a Scene,
    getnode: impl Fn(usize) -> Node<'a>,
) -> Vec<(Node<'a>, usize, f64, Matrix4, Quaternion)> {
    let mut result = Vec::new();
    for node in scene.nodes() {
        let (mat, rot) = mat_rot(&node);
        result.push((node, 0, 0.0, mat, rot));
    }
    let mut traversed = 0;
    while traversed < result.len() {
        let (node, lod_level, _, parent_mat, parent_rot) = result[traversed].clone();
        if lod_level > 0 {
            traversed += 1;
            continue;
        }

        if let Some(v) = node.extension_value("MSFT_lod") {
            let coverage = v.get("screencoverage").map(|v| {
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect::<Vec<_>>()
            });

            if let Some(coverage) = coverage {
                let mut coverage_iter = coverage.into_iter();

                if let Some(arr) = v.get("ids").and_then(|ids| ids.as_array()) {
                    result[traversed].2 = coverage_iter.next().unwrap();

                    for ((lod_level, v), coverage) in arr.iter().enumerate().zip(coverage_iter) {
                        if let Some(id) = v.as_u64() {
                            let node = getnode(id as usize);
                            result.push((node, 1 + lod_level, coverage, parent_mat, parent_rot));
                        }
                    }
                }
            }
        }

        result.extend(node.children().map(|node| {
            let (mat, rot) = mat_rot(&node);
            (node, 0, 0.0, parent_mat * mat, parent_rot * rot)
        }));

        traversed += 1;
    }

    result
}

pub fn load_mesh(gfx: &mut GfxContext, asset_name: &Path) -> Result<Mesh, LoadMeshError> {
    load_mesh_with_properties(gfx, asset_name, false).map(|x| x.0)
}

pub fn load_mesh_with_properties(
    gfx: &mut GfxContext,
    asset_name: &Path,
    force_base_model: bool,
) -> Result<(Mesh, CPUMesh), LoadMeshError> {
    let mut path = PathBuf::new();
    path.push("assets/models_opt/");
    path.push(asset_name);

    if !path.exists() || force_base_model {
        path.clear();
        path.push("assets/models/");
        path.push(asset_name);
    }

    let t = Instant::now();

    let (doc, data, images) =
        gltf::import(&path).map_err(|e| LoadMeshError::GltfLoadError(Arc::new(e)))?;

    let exts = doc
        .extensions_used()
        .filter(|x| !matches!(x, &"MSFT_lod"))
        .fold(String::new(), |a, b| a + ", " + b);
    if !exts.is_empty() {
        log::warn!("extension not supported: {}", exts)
    }

    let scene = doc.default_scene().ok_or(LoadMeshError::NoDefaultScene)?;

    let (mats, needs_tangents) = load_materials(gfx, &doc, &images)?;

    let mut meshb = MeshBuilder::<false>::new_without_mat();

    let getnode = |id| doc.nodes().nth(id).unwrap();

    for (node, lod_id, coverage, transform_mat, rot_qat) in find_nodes(&scene, getnode) {
        let mesh = unwrap_cont!(node.mesh());
        let mut primitives = mesh.primitives().collect::<Vec<_>>();
        primitives.sort_unstable_by_key(|x| x.material().index());

        meshb.set_lod(lod_id, coverage);

        for primitive in primitives {
            let bbox = primitive.bounding_box();

            let reader = primitive.reader(|b| Some(&data.get(b.index())?.0[..b.length()]));
            let matid = primitive
                .material()
                .index()
                .ok_or(LoadMeshError::NoMaterial)?;

            let positions = unwrap_cont!(reader.read_positions()).map(Vec3::from);
            let normals = unwrap_cont!(reader.read_normals()).map(Vec3::from);
            let uv = unwrap_cont!(reader.read_tex_coords(0))
                .into_f32()
                .map(Vec2::from);
            let read_indices: Vec<u32> = unwrap_cont!(reader.read_indices()).into_u32().collect();
            let raw = positions.zip(normals).zip(uv).map(|((p, n), uv)| {
                let pos = transform_mat * p.w(1.0);
                let pos = pos.xyz() / pos.w;
                (pos, rot_qat * n, uv)
            });

            meshb.extend_with(Some(mats[matid]), |vertices, add_idx| {
                for (pos, normal, uv) in raw {
                    vertices.push(MeshVertex {
                        position: pos.into(),
                        normal,
                        uv: uv.into(),
                        color: [1.0, 1.0, 1.0, 1.0],
                        tangent: [0.0; 4],
                    })
                }
                for idx in read_indices {
                    add_idx(idx as IndexType);
                }
            });
            meshb.set_bounds(AABB3 {
                ll: bbox.min.into(),
                ur: bbox.max.into(),
            });
        }
    }

    let props = CPUMesh {
        n_textures: images.len(),
        n_triangles: meshb.lods()[0].n_indices / 3,
        gltf_doc: doc,
        gltf_data: data,
        asset_path: path,
    };

    if needs_tangents {
        meshb.compute_tangents();
    }
    let m = meshb.build(gfx).ok_or(LoadMeshError::NoVertices)?;

    log::info!(
        "loaded mesh {:?} in {}ms{}",
        &props.asset_path,
        1000.0 * t.elapsed().as_secs_f32(),
        if needs_tangents { " (tangents)" } else { "" }
    );

    Ok((m, props))
}
