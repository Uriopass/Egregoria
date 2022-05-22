use crate::{GfxContext, IndexType, Mesh, MeshBuilder, MeshVertex, TextureBuilder};
use geom::{Matrix4, Quaternion, Vec2, Vec3};
use gltf::image::Format;
use gltf::json::texture::{MagFilter, MinFilter};
use image::{DynamicImage, ImageBuffer};
use std::path::Path;
use std::sync::Arc;
use wgpu::FilterMode;

pub fn load_mesh(path: impl AsRef<Path>, gfx: &GfxContext) -> Option<Mesh> {
    let mut flat_vertices: Vec<MeshVertex> = vec![];
    let mut indices = vec![];

    let (doc, data, images) = gltf::import(path)
        .map_err(|e| log::error!("invalid mesh: {}", e))
        .ok()?;

    let nodes = doc.nodes();

    if doc.materials().len() != 1 {
        log::error!(
            "invalid mesh: only 1 material is supported. got: {}",
            doc.materials().len()
        );
        return None;
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
            log::info!("{} vertices", raw.len());
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
        log::error!("invalid mesh: no valid mesh in obj");
        return None;
    }

    let mat = unwrap_or!(doc.materials().next(), {
        log::error!("invalid mesh: no material in mesh");
        return None;
    });
    let tex = unwrap_or!(mat.pbr_metallic_roughness().base_color_texture(), {
        log::error!("invalid mesh: no base color texture");
        return None;
    })
    .texture();

    //    let sampler = tex.sampler().mag_filter().unwrap()
    let albedo_data = unwrap_or!(images.into_iter().nth(tex.source().index()), {
        log::error!("invalid mesh: couldn't find nth image");
        return None;
    });

    let w = albedo_data.width;
    let h = albedo_data.height;
    let d = albedo_data.pixels;
    let albedo_img = match albedo_data.format {
        Format::R8 => DynamicImage::ImageLuma8(ImageBuffer::from_raw(w, h, d)?),
        Format::R8G8 => DynamicImage::ImageLumaA8(ImageBuffer::from_raw(w, h, d)?),
        Format::R8G8B8 => DynamicImage::ImageRgb8(ImageBuffer::from_raw(w, h, d)?),
        Format::R8G8B8A8 => DynamicImage::ImageRgba8(ImageBuffer::from_raw(w, h, d)?),
        _ => {
            log::error!("invalid mesh: unsupported 16 bits pixel texture");
            return None;
        }
    };

    let sampl = tex.sampler();

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

    let albedo = Arc::new(
        TextureBuilder::from_img(albedo_img)
            .with_label("some material albedo")
            .with_sampler(sampler)
            .build(&gfx.device, &gfx.queue),
    );

    let mut meshb = MeshBuilder::new();
    meshb.vertices = flat_vertices;
    meshb.indices = indices;

    meshb.build(gfx, albedo)
}
