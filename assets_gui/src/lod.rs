use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use meshopt::{optimize_vertex_fetch, SimplifyOptions, Vertex, VertexDataAdapter};

use common::unwrap_cont;
use engine::gltf::json::validation::{Checked, USize64};
use engine::gltf::json::{accessor, Accessor, Index, Value};
use engine::gltf::{json, Document, Semantic};
use engine::meshload::CPUMesh;
use geom::{Vec2, Vec3};

#[derive(Debug)]
pub enum LodGenerateError {
    AlreadyHasLOD,
}

#[derive(Debug, Copy, Clone)]
pub struct LodGenerateParams {
    pub n_lods: usize,
    pub quality: f32,
    pub sloppy: bool,
}

pub fn lod_generate(m: &mut CPUMesh, params: LodGenerateParams) -> Result<(), LodGenerateError> {
    let doc = &mut m.gltf_doc;
    let data = &mut m.gltf_data;

    let scene = doc.default_scene().expect("no default scene");

    let mut generated_lods = vec![];

    let getnode = |id: usize| doc.nodes().nth(id).unwrap();

    for (node, lod, _, _, _) in engine::meshload::find_nodes(&scene, getnode) {
        let Some(mesh) = node.mesh() else {
            continue;
        };
        if lod != 0 {
            return Err(LodGenerateError::AlreadyHasLOD);
        }

        let mut primitives = mesh.primitives().collect::<Vec<_>>();
        primitives.sort_unstable_by_key(|x| x.material().index());

        for i in 0..params.n_lods {
            let mut primitive_lods = vec![];

            for primitive in &primitives {
                let reader = primitive.reader(|b| Some(&data.get(b.index())?.0[..b.length()]));

                let positions = unwrap_cont!(reader.read_positions()).map(Vec3::from);
                let normals = unwrap_cont!(reader.read_normals()).map(Vec3::from);
                let uv = unwrap_cont!(reader.read_tex_coords(0))
                    .into_f32()
                    .map(Vec2::from);
                let indices: Vec<u32> = unwrap_cont!(reader.read_indices()).into_u32().collect();

                let mut vertices = Vec::new();

                for ((p, normal), uv) in positions.zip(normals).zip(uv) {
                    vertices.push(Vertex {
                        p: [p.x, p.y, p.z],
                        n: [normal.x, normal.y, normal.z],
                        t: [uv.x, uv.y],
                    });
                }

                let position_offset = 0;
                let vertex_stride = std::mem::size_of::<Vertex>();
                let vertex_data = meshopt::typed_to_bytes(&vertices);

                let adapter = VertexDataAdapter::new(vertex_data, vertex_stride, position_offset)
                    .expect("failed to create vertex data reader");

                let target_count = indices.len() / 3 / (i + 1);
                let target_error = (0.1 + i as f32 * 0.1) * (1.0 - params.quality);

                let mut optimized_indices = if params.sloppy {
                    meshopt::simplify_sloppy(&indices, &adapter, target_count, target_error, None)
                } else {
                    meshopt::simplify(
                        &indices,
                        &adapter,
                        target_count,
                        target_error,
                        SimplifyOptions::empty(),
                        None,
                    )
                };

                let optimized_vertices = optimize_vertex_fetch(&mut optimized_indices, &vertices);

                primitive_lods.push((primitive.index(), optimized_vertices, optimized_indices));
            }
            generated_lods.push((i, node.index(), mesh.index(), primitive_lods));
        }
    }

    let glb_buffer_id = engine::meshload::glb_buffer_id(doc);
    let glb_data = &mut data[glb_buffer_id].0;

    let mut json = doc.clone().into_json();

    let mut create_view = |count: usize,
                           data: Vec<u8>,
                           component_type: accessor::ComponentType,
                           type_: accessor::Type,
                           min: Option<Value>,
                           max: Option<Value>| {
        let view = json::buffer::View {
            buffer: Index::new(glb_buffer_id as u32),
            byte_length: USize64::from(data.len()),
            byte_offset: Some(USize64::from(glb_data.len())),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        };
        glb_data.extend(data);

        json.accessors.push(Accessor {
            buffer_view: Some(Index::new(json.buffer_views.len() as u32)),
            byte_offset: None,
            component_type: Checked::Valid(accessor::GenericComponentType(component_type)),
            count: USize64::from(count),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(type_),
            min,
            max,
            name: None,
            normalized: false,
            sparse: None,
        });
        json.buffer_views.push(view);

        Index::<Accessor>::new(json.accessors.len() as u32 - 1)
    };

    for (lod_id, node_id, mesh_id, primitives) in generated_lods {
        let new_idx_id = json.meshes.len() as u32;
        let orig_mesh = json.meshes.get_mut(mesh_id).unwrap();

        let mut triangles = 0;
        let mut json_primitives = vec![];
        for (primitive_id, vertices, indices) in primitives {
            triangles += indices.len() / 3;
            let json_primitive = orig_mesh.primitives.get(primitive_id).unwrap();

            let mut json_attributes = BTreeMap::new();

            let mut minp = [f32::MAX, f32::MAX, f32::MAX];
            let mut maxp = [f32::MIN, f32::MIN, f32::MIN];

            for v in vertices.iter() {
                minp[0] = minp[0].min(v.p[0]);
                minp[1] = minp[1].min(v.p[1]);
                minp[2] = minp[2].min(v.p[2]);

                maxp[0] = maxp[0].max(v.p[0]);
                maxp[1] = maxp[1].max(v.p[1]);
                maxp[2] = maxp[2].max(v.p[2]);
            }

            let minp = minp.iter().map(|&x| Value::from(x)).collect();
            let maxp = maxp.iter().map(|&x| Value::from(x)).collect();

            let positions = vertices.iter().map(|v| v.p).collect::<Vec<_>>();
            json_attributes.insert(
                Checked::Valid(Semantic::Positions),
                create_view(
                    positions.len(),
                    to_padded_byte_vector(positions),
                    accessor::ComponentType::F32,
                    accessor::Type::Vec3,
                    Some(minp),
                    Some(maxp),
                ),
            );

            let normals = vertices.iter().map(|v| v.n).collect::<Vec<_>>();
            json_attributes.insert(
                Checked::Valid(Semantic::Normals),
                create_view(
                    normals.len(),
                    to_padded_byte_vector(normals),
                    accessor::ComponentType::F32,
                    accessor::Type::Vec3,
                    None,
                    None,
                ),
            );

            let tex_coords = vertices.iter().map(|v| v.t).collect::<Vec<_>>();
            json_attributes.insert(
                Checked::Valid(Semantic::TexCoords(0)),
                create_view(
                    tex_coords.len(),
                    to_padded_byte_vector(tex_coords),
                    accessor::ComponentType::F32,
                    accessor::Type::Vec2,
                    None,
                    None,
                ),
            );

            let indice_view = create_view(
                indices.len(),
                to_padded_byte_vector(indices),
                accessor::ComponentType::U32,
                accessor::Type::Scalar,
                None,
                None,
            );

            let json_primitive = json::mesh::Primitive {
                attributes: json_attributes,
                extensions: json_primitive.extensions.clone(),
                extras: json_primitive.extras.clone(),
                indices: Some(indice_view),
                material: json_primitive.material,
                mode: json_primitive.mode,
                targets: json_primitive.targets.clone(),
            };
            json_primitives.push(json_primitive);
        }

        let lod_mesh = json::Mesh {
            extensions: orig_mesh.extensions.clone(),
            extras: orig_mesh.extras.clone(),
            name: None,
            primitives: json_primitives,
            weights: orig_mesh.weights.clone(),
        };

        let orig_node = json.nodes.get_mut(node_id).unwrap();
        if orig_node.extensions.is_none() {
            orig_node.extensions = Some(json::extensions::scene::Node::default());
        }

        let ext = orig_node.extensions.as_mut().unwrap();

        let entry = ext.others.entry("MSFT_lod".to_string());

        fn autocoverage(triangles: usize) -> f32 {
            (triangles as f32 / 100000.0).min(0.5)
        }

        let obj = entry.or_insert_with(|| {
            Value::Object(
                [
                    ("ids".to_string(), Value::Array(vec![])),
                    (
                        "screencoverage".to_string(),
                        Value::Array(vec![autocoverage(m.n_triangles).into()]),
                    ),
                ]
                .into_iter()
                .collect(),
            )
        });
        obj["ids"].as_array_mut().unwrap().push(new_idx_id.into());
        obj["screencoverage"]
            .as_array_mut()
            .unwrap()
            .push(autocoverage(triangles).into());

        let lod_node = json::Node {
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            name: Some(format!(
                "lod {} for {} ({})",
                lod_id,
                orig_mesh.name.as_deref().unwrap_or(""),
                node_id
            )),
            mesh: Some(Index::new(json.meshes.len() as u32)),
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        };

        json.meshes.push(lod_mesh);
        json.nodes.push(lod_node);
        //json.scenes[0]
        //    .nodes
        //    .push(Index::new(json.nodes.len() as u32 - 1)); // add to scene to show in blender
    }

    json.buffers[glb_buffer_id].byte_length = USize64::from(glb_data.len());

    if !json.extensions_used.contains(&"MSFT_lod".to_string()) {
        json.extensions_used.push("MSFT_lod".to_string());
    }

    *doc = Document::from_json_without_validation(json);
    Ok(())
}

pub fn export_doc_opt(mesh: &CPUMesh) {
    let glb_buffer_id = engine::meshload::glb_buffer_id(&mesh.gltf_doc);
    let json = mesh.gltf_doc.clone().into_json();
    let glb_data = &*mesh.gltf_data[glb_buffer_id];

    let json_string = json::serialize::to_string(&json).expect("Serialization error");
    let mut json_offset = json_string.len();
    align_to_multiple_of_four(&mut json_offset);

    let glb = engine::gltf::binary::Glb {
        header: engine::gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            // N.B., the size of binary glTF file is limited to range of `u32`.
            length: (json_offset + glb_data.len())
                .try_into()
                .expect("file size exceeds binary glTF limit"),
        },
        bin: Some(Cow::Borrowed(glb_data)),
        json: Cow::Owned(json_string.into_bytes()),
    };
    let Some(name) = mesh.asset_path.file_name() else {
        log::error!("asset path has no file name");
        return;
    };

    let mut asset_path = PathBuf::new();
    asset_path.push("assets");
    asset_path.push("models_opt");
    asset_path.push(name);

    let writer = std::fs::File::create(asset_path).expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
}

fn to_padded_byte_vector<T>(vec: Vec<T>) -> Vec<u8> {
    let byte_length = vec.len() * std::mem::size_of::<T>();
    let byte_capacity = vec.capacity() * std::mem::size_of::<T>();
    let alloc = vec.into_boxed_slice();
    let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
    let mut new_vec = unsafe { Vec::from_raw_parts(ptr, byte_length, byte_capacity) };
    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }
    new_vec
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}
