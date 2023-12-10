use crate::{
    bg_layout_litmesh, pbuffer::PBuffer, CompiledModule, Drawable, FrameContext, GfxContext,
    IndexType, PipelineBuilder, RenderParams, TerrainVertex, Texture, Uniform, TL,
};
use geom::{vec2, vec3, Camera, InfiniteFrustrum, Intersect3, Matrix4, Vec2, AABB3};
use std::sync::Arc;
use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BufferUsages, Extent3d, FilterMode,
    ImageCopyTexture, ImageDataLayout, IndexFormat, Origin3d, RenderPass, RenderPipeline,
    TextureFormat, TextureUsages, VertexAttribute, VertexBufferLayout,
};

const LOD: usize = 4;
const LOD_MIN_DIST_LOG2: f32 = 11.0; // 2^10 = 1024, meaning until 2048m away, we use the highest lod
const MAX_HEIGHT: f32 = 1024.0;
const MAX_DIFF: f32 = 32.0;

pub struct TerrainChunk {
    pub dirt_id: u32,
}

pub struct TerrainRender<const CSIZE: usize, const CRESOLUTION: usize> {
    terrain_tex: Arc<Texture>,
    #[allow(unused)]
    grass_tex: Arc<Texture>, // kept alive
    vertices: [PBuffer; LOD],
    indices: [(PBuffer, u32); LOD],
    instances: [(PBuffer, u32); LOD],
    bgs: [Arc<wgpu::BindGroup>; LOD],
    w: u32,
    h: u32,
}

pub struct TerrainPrepared {
    terrainbgs: [Arc<wgpu::BindGroup>; LOD],
    vertices: [PBuffer; LOD],
    indices: [(PBuffer, u32); LOD],
    instances: [(PBuffer, u32); LOD],
}

impl<const CSIZE: usize, const CRESOLUTION: usize> TerrainRender<CSIZE, CRESOLUTION> {
    pub fn new(gfx: &mut GfxContext, w: u32, h: u32, grass: Arc<Texture>) -> Self {
        let (indices, vertices) = Self::generate_indices_mesh(gfx);
        let mut tex = Texture::create_fbo(
            &gfx.device,
            (w * CRESOLUTION as u32 + 1, h * CRESOLUTION as u32 + 1),
            TextureFormat::Rg16Uint,
            TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            None,
        );
        tex.sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let mut bgs = vec![];
        for lod in 0..LOD {
            let uni = Uniform::new(
                TerrainChunkData {
                    lod: lod as u32,
                    resolution: 1 + CRESOLUTION as u32 / (1 << lod as u32),
                    distance_lod_cutoff: 2.0f32.powf(1.0 + LOD_MIN_DIST_LOG2 + lod as f32)
                        - std::f32::consts::SQRT_2 * CSIZE as f32,
                    cell_size: CSIZE as f32 / CRESOLUTION as f32,
                    inv_cell_size: CRESOLUTION as f32 / CSIZE as f32,
                },
                &gfx.device,
            );

            let texs = &[&tex, &grass];
            let mut bg_entries = Vec::with_capacity(3);
            bg_entries.extend(Texture::multi_bindgroup_entries(0, texs));
            bg_entries.push(uni.bindgroup_entry(4));
            bgs.push(Arc::new(
                gfx.device.create_bind_group(&BindGroupDescriptor {
                    layout: &gfx
                        .get_pipeline(TerrainPipeline {
                            depth: false,
                            smap: false,
                        })
                        .get_bind_group_layout(2),
                    entries: &bg_entries,
                    label: Some("terrain bindgroup"),
                }),
            ));
        }

        defer!(log::info!("finished init of terrain render"));
        Self {
            bgs: collect_arrlod(bgs),
            terrain_tex: Arc::new(tex),
            grass_tex: grass,
            indices,
            vertices,
            w,
            h,
            instances: collect_arrlod((0..LOD).map(|_| (PBuffer::new(BufferUsages::VERTEX), 0))),
        }
    }

    pub fn update_chunk(
        &mut self,
        gfx: &mut GfxContext,
        cell: (u32, u32),
        chunk: &[[f32; CRESOLUTION]; CRESOLUTION],
        get_up: impl Fn(usize) -> Option<f32>,
        get_down: impl Fn(usize) -> Option<f32>,
        get_right: impl Fn(usize) -> Option<f32>,
        get_left: impl Fn(usize) -> Option<f32>,
    ) -> bool {
        fn pack(height: f32, diffx: f32, diffy: f32) -> [u8; 4] {
            let a = ((height.clamp(-MAX_HEIGHT, MAX_HEIGHT) / MAX_HEIGHT * i16::MAX as f32
                + 32768.0) as u16)
                .to_le_bytes();

            if height >= MAX_HEIGHT || height <= -MAX_HEIGHT {
                return [a[0], a[1], 128, 128]; // normal is zero if we hit max height
            }

            let b = (diffx.clamp(-MAX_DIFF, MAX_DIFF) / MAX_DIFF * i8::MAX as f32 + 128.0) as u8;
            let c = (diffy.clamp(-MAX_DIFF, MAX_DIFF) / MAX_DIFF * i8::MAX as f32 + 128.0) as u8;
            [a[0], a[1], b, c]
        }

        // Need to add one more vertex on the edge of the map because when rendering a chunk
        // we render "to the next chunk", which doesn't exist on the edge
        let extrax = cell.0 + 1 == self.w;
        let extray = cell.1 + 1 == self.h;
        let mut contents =
            Vec::with_capacity((CRESOLUTION + extrax as usize) * (CRESOLUTION + extray as usize));

        let mut holder_y_edge: [f32; CRESOLUTION] = [0.0; CRESOLUTION];
        let mut j = 0;
        let mut last_ys = &[(); CRESOLUTION].map(|_| {
            let height_down = get_down(j).unwrap_or(chunk[0][j]);
            j += 1;
            height_down
        });
        for i in 0..CRESOLUTION {
            let ys = &chunk[i];
            let next_ys = chunk.get(i + 1).unwrap_or_else(|| {
                for j in 0..CRESOLUTION {
                    holder_y_edge[j] = get_up(j).unwrap_or(ys[j]);
                }
                &holder_y_edge
            });

            let mut last_height = get_left(i).unwrap_or(ys[0]);
            for j in 0..CRESOLUTION {
                let height = ys[j];
                let dh_x = last_height
                    - ys.get(j + 1)
                        .copied()
                        .unwrap_or_else(|| get_right(i).unwrap_or(height));
                let dh_y = last_ys[j] - next_ys[j];

                contents.extend(pack(height, dh_x, dh_y));
                last_height = height;
            }
            if extrax {
                contents.extend(pack(ys[ys.len() - 1], 0.0, 0.0));
            }

            last_ys = ys;
        }
        if extray {
            for i in 0..CRESOLUTION {
                contents.extend(pack(chunk[CRESOLUTION - 1][i], 0.0, 0.0));
            }
            if extrax {
                contents.extend(pack(chunk[CRESOLUTION - 1][CRESOLUTION - 1], 0.0, 0.0));
            }
        }

        let w = CRESOLUTION as u32 + extrax as u32;
        let h = CRESOLUTION as u32 + extray as u32;

        gfx.queue.write_texture(
            ImageCopyTexture {
                texture: &self.terrain_tex.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: cell.0 * CRESOLUTION as u32,
                    y: cell.1 * CRESOLUTION as u32,
                    z: 0,
                },
                aspect: Default::default(),
            },
            &contents,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        true
    }

    pub fn draw_terrain(
        &mut self,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        fctx: &mut FrameContext<'_>,
    ) {
        profiling::scope!("terrain::draw_terrain");
        let eye = cam.eye();

        let mut instances = vec![Vec::<TerrainInstance>::new(); LOD];
        for y in 0..self.h {
            for x in 0..self.w {
                let chunk_corner = vec2(x as f32, y as f32) * CSIZE as f32;
                let chunk_center = chunk_corner + Vec2::splat(CSIZE as f32 * 0.5);

                if !frustrum.intersects(&AABB3::centered(
                    chunk_center.z0(),
                    vec3(CSIZE as f32, CSIZE as f32, 2000.0),
                )) {
                    continue;
                }

                let lod =
                    (eye.distance(chunk_center.z0()).log2() - LOD_MIN_DIST_LOG2).max(0.0) as usize;
                let lod = lod.min(LOD - 1);

                instances[lod].push(TerrainInstance {
                    offset: chunk_corner,
                })
            }
        }

        for (lod, instance) in instances.into_iter().enumerate() {
            self.instances[lod].1 = instance.len() as u32;
            self.instances[lod]
                .0
                .write(fctx.gfx, bytemuck::cast_slice(&instance));
        }

        fctx.objs.push(Box::new(TerrainPrepared {
            terrainbgs: self.bgs.clone(),
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            instances: self.instances.clone(),
        }));
    }

    fn generate_indices_mesh(gfx: &GfxContext) -> ([(PBuffer, u32); LOD], [PBuffer; LOD]) {
        let mut indlod = vec![];
        let mut vertlod = vec![];
        let cell_size = (CSIZE / CRESOLUTION) as f32;
        for lod in 0..LOD {
            let scale = 1 << lod;
            let resolution = CRESOLUTION / scale;

            let mut indices: Vec<IndexType> = Vec::with_capacity(6 * resolution * resolution);
            let mut vertices: Vec<TerrainVertex> =
                Vec::with_capacity((resolution + 1) * (resolution + 1));

            let resolution = resolution as IndexType;
            let w = resolution + 1;
            for y in 0..=resolution {
                for x in 0..=resolution {
                    let pos = vec2(x as f32, y as f32) * cell_size * scale as f32;
                    vertices.push(TerrainVertex {
                        position: [pos.x, pos.y],
                    });

                    if x < resolution && y < resolution {
                        let idx = y * w + x;
                        indices.push(idx);
                        indices.push(idx + 1);
                        indices.push(idx + w + 1);

                        indices.push(idx);
                        indices.push(idx + w + 1);
                        indices.push(idx + w);
                    }
                }
            }

            let l = indices.len();

            let mut buf = PBuffer::new(BufferUsages::INDEX);
            buf.write(gfx, bytemuck::cast_slice(&indices));
            indlod.push((buf, l as u32));

            let mut buf = PBuffer::new(BufferUsages::VERTEX);
            buf.write(gfx, bytemuck::cast_slice(&vertices));
            vertlod.push(buf);
        }
        (collect_arrlod(indlod), collect_arrlod(vertlod))
    }
}

#[derive(Hash)]
struct TerrainPipeline {
    depth: bool,
    smap: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct TerrainInstance {
    pub offset: Vec2,
}
u8slice_impl!(TerrainInstance);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerrainChunkData {
    lod: u32,                 // 0 = highest resolution, 1 = half resolution, etc.
    resolution: u32,          // width of the vertex grid
    distance_lod_cutoff: f32, // max distance at which to switch to the next lod to have smooth transitions
    cell_size: f32,
    inv_cell_size: f32,
}
u8slice_impl!(TerrainChunkData);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![1 => Float32x2];

impl TerrainInstance {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

impl TerrainPrepared {
    fn set_buffers<'a>(&'a self, rp: &mut RenderPass<'a>) {
        for lod in 0..LOD {
            let (instances, n_instances) = &self.instances[lod];
            if *n_instances == 0 {
                continue;
            }

            let (ind, n_indices) = &self.indices[lod];
            let vertices = &self.vertices[lod];

            rp.set_bind_group(2, &self.terrainbgs[lod], &[]);
            rp.set_vertex_buffer(0, vertices.slice().unwrap());
            rp.set_vertex_buffer(1, instances.slice().unwrap());
            rp.set_index_buffer(ind.slice().unwrap(), IndexFormat::Uint32);
            rp.draw_indexed(0..*n_indices, 0, 0..*n_instances);
        }
    }
}

impl PipelineBuilder for TerrainPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        let terrainlayout = gfx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &Texture::bindgroup_layout_entries(0, [TL::UInt, TL::Float].into_iter())
                    .chain(std::iter::once(
                        Uniform::<TerrainChunkData>::bindgroup_layout_entry(4),
                    ))
                    .collect::<Vec<_>>(),
                label: Some("terrain bindgroup layout"),
            });
        let vert = &mk_module("terrain.vert");

        if !self.depth {
            let frag = &mk_module("terrain.frag");

            return gfx.color_pipeline(
                "terrain",
                &[
                    &gfx.projection.layout,
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    &terrainlayout,
                    &bg_layout_litmesh(&gfx.device),
                ],
                &[TerrainVertex::desc(), TerrainInstance::desc()],
                vert,
                frag,
            );
        }

        gfx.depth_pipeline_bglayout(
            &[TerrainVertex::desc(), TerrainInstance::desc()],
            vert,
            None,
            self.smap,
            &[
                &gfx.projection.layout,
                &gfx.render_params.layout,
                &terrainlayout,
            ],
        )
    }
}

impl Drawable for TerrainPrepared {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = gfx.get_pipeline(TerrainPipeline {
            depth: false,
            smap: false,
        });

        rp.set_pipeline(pipeline);

        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);

        self.set_buffers(rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        if shadow_cascade.is_some() {
            // Terrain don't cast shadows for now as they are hard to do properly
            // It needs separate frustrum culling + actual good shadow acne fix
            return;
        }
        rp.set_pipeline(gfx.get_pipeline(TerrainPipeline {
            depth: true,
            smap: shadow_cascade.is_some(),
        }));
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);

        self.set_buffers(rp);
    }
}

fn collect_arrlod<T>(x: impl IntoIterator<Item = T>) -> [T; LOD] {
    let mut iter = x.into_iter();
    [(); LOD].map(move |_| iter.next().expect("iterator too short"))
}
