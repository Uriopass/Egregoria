use std::sync::Arc;

use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, IndexFormat,
    Origin3d, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat, TextureView,
    VertexAttribute, VertexBufferLayout,
};

use geom::{vec2, vec3, Camera, HeightmapChunk, Intersect3, Matrix4, Vec2, AABB3};

use crate::{
    bg_layout_litmesh, pbuffer::PBuffer, CompiledModule, Drawable, FrameContext, GfxContext,
    IndexType, PipelineBuilder, PipelineKey, Texture, TextureBuilder, Uniform, TL,
};

const LOD: usize = 5;
const LOD_MIN_DIST_LOG2: f32 = 9.0; // 2^9 = 512, meaning until 1048m away, we use the highest lod
const MAX_HEIGHT: f32 = 2008.0;
const MIN_HEIGHT: f32 = -40.0;
const UPSCALE_LOD: usize = 2; // amount of LOD that are superior to base heightmap data

/// CSIZE is the size of a chunk in meters
/// CRESOLUTION is the resolution of a chunk, in vertices, at the chunk data level (not LOD0 since we upsample)
pub struct HeightmapRender<const CSIZE: u32, const CRESOLUTION: usize> {
    heightmap_tex: Arc<Texture>,
    normal_tex: Arc<Texture>,

    indices: [(PBuffer, u32); LOD],
    instances: [(PBuffer, u32); LOD],
    bgs: Arc<[wgpu::BindGroup; LOD]>,
    w: u32,
    h: u32,

    normal_pipeline: RenderPipeline,
    normal_unis: [Uniform<f32>; LOD],
    downsample_pipeline: RenderPipeline,
    upsample_pipeline: RenderPipeline,
}

pub struct HeightmapPrepared {
    heightmapbgs: Arc<[wgpu::BindGroup; LOD]>,
    indices: [(PBuffer, u32); LOD],
    instances: [(PBuffer, u32); LOD],
}

impl<const CSIZE: u32, const CRESOLUTION: usize> HeightmapRender<CSIZE, CRESOLUTION> {
    const LOD0_RESOLUTION: usize = CRESOLUTION * (1 << UPSCALE_LOD);

    pub fn new(gfx: &mut GfxContext, w: u32, h: u32) -> Self {
        debug_assert!(
            Self::LOD0_RESOLUTION >= 1 << LOD,
            "LOD0 HEIGHTMAP RESOLUTION must be >= {}",
            1 << LOD
        );

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");
        let cliff = gfx.texture("assets/sprites/cliff.jpg", "cliff");

        let indices = Self::generate_indices_mesh(gfx);

        let heightmap_tex = TextureBuilder::empty(
            w * Self::LOD0_RESOLUTION as u32,
            h * Self::LOD0_RESOLUTION as u32,
            1,
            TextureFormat::R16Uint,
        )
        .with_fixed_mipmaps(LOD as u32)
        .with_sampler(wgpu::SamplerDescriptor {
            label: Some("heightmap sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        })
        .with_no_anisotropy()
        .build(&gfx.device, &gfx.queue);

        let normals_tex = TextureBuilder::empty(
            w * Self::LOD0_RESOLUTION as u32,
            h * Self::LOD0_RESOLUTION as u32,
            1,
            TextureFormat::R16Uint,
        )
        .with_fixed_mipmaps(LOD as u32)
        .with_sampler(wgpu::SamplerDescriptor {
            label: Some("heightmap normals sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        })
        .with_no_anisotropy()
        .build(&gfx.device, &gfx.queue);

        let mut bgs = vec![];
        for lod in 0..LOD {
            let scale = 1 << lod as u32;
            let uni = Uniform::new(
                HeightmapChunkData {
                    lod: lod as u32,
                    lod_pow2: scale,
                    resolution: 1 + Self::LOD0_RESOLUTION as u32 / scale,
                    distance_lod_cutoff: 2.0f32.powf(1.0 + LOD_MIN_DIST_LOG2 + lod as f32)
                        - std::f32::consts::FRAC_1_SQRT_2 * CSIZE as f32,
                    cell_size: CSIZE as f32 / Self::LOD0_RESOLUTION as f32,
                    inv_cell_size: Self::LOD0_RESOLUTION as f32 / CSIZE as f32,
                },
                &gfx.device,
            );

            let texs = &[&heightmap_tex, &normals_tex, &grass, &cliff];
            let mut bg_entries = Vec::with_capacity(12);
            bg_entries.extend(Texture::multi_bindgroup_entries(0, texs));
            bg_entries.push(uni.bindgroup_entry(8));
            bgs.push(
                gfx.device.create_bind_group(&BindGroupDescriptor {
                    layout: &gfx
                        .get_pipeline(HeightmapPipeline {
                            depth: false,
                            smap: false,
                        })
                        .get_bind_group_layout(1),
                    entries: &bg_entries,
                    label: Some("heightmap bindgroup"),
                }),
            );
        }

        defer!(log::info!("finished init of heightmap render"));
        Self {
            normal_pipeline: normal_pipeline(gfx, &normals_tex),
            normal_unis: collect_arrlod((0..LOD).map(|lod| {
                Uniform::new(
                    (CSIZE << lod) as f32 / Self::LOD0_RESOLUTION as f32,
                    &gfx.device,
                )
            })),
            downsample_pipeline: resample_pipeline(gfx, &heightmap_tex, "downsample"),
            upsample_pipeline: resample_pipeline(gfx, &heightmap_tex, "upsample"),

            bgs: Arc::new(collect_arrlod(bgs)),
            heightmap_tex: Arc::new(heightmap_tex),
            normal_tex: Arc::new(normals_tex),
            indices,
            w,
            h,
            instances: collect_arrlod((0..LOD).map(|_| (PBuffer::new(BufferUsages::VERTEX), 0))),
        }
    }

    pub fn update_chunk(
        &mut self,
        gfx: &mut GfxContext,
        cell: (u32, u32),
        chunk: &HeightmapChunk<CRESOLUTION, CSIZE>,
    ) {
        fn pack(height: f32) -> [u8; 2] {
            let h_encoded = ((height.clamp(MIN_HEIGHT, MAX_HEIGHT) - MIN_HEIGHT)
                / (MAX_HEIGHT - MIN_HEIGHT)
                * u16::MAX as f32) as u16;
            h_encoded.to_le_bytes()
        }

        let mut contents = Vec::with_capacity(CRESOLUTION * CRESOLUTION * 2);

        for y in 0..CRESOLUTION {
            for x in 0..CRESOLUTION {
                contents.extend(pack(chunk.height_idx(x, y).unwrap()));
            }
        }

        let h = CRESOLUTION as u32;
        let w = CRESOLUTION as u32;

        gfx.queue.write_texture(
            ImageCopyTexture {
                texture: &self.heightmap_tex.texture,
                mip_level: UPSCALE_LOD as u32,
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
                bytes_per_row: Some(w * 2),
                rows_per_image: Some(h),
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn draw_heightmap(&mut self, cam: &Camera, fctx: &mut FrameContext<'_>) {
        profiling::scope!("heightmap::draw_heightmap");
        let eye = cam.eye();

        let mut instances = vec![Vec::<HeightmapInstance>::new(); LOD];

        // We calculate lod in 2 passes to be able to generate the stitches
        // special: lod 0 = dont render, stored as 1 + lod
        let mut assigned_lod: Vec<u8> = vec![0; (self.h * self.w) as usize];

        // frustrum culling + lod assignment
        for y in 0..self.h {
            for x in 0..self.w {
                let chunk_corner = vec2(x as f32, y as f32) * CSIZE as f32;
                let chunk_center = chunk_corner + Vec2::splat(CSIZE as f32 * 0.5);

                if !fctx.gfx.frustrum.intersects(&AABB3::new(
                    chunk_corner.z(MIN_HEIGHT),
                    chunk_corner.z0() + vec3(CSIZE as f32, CSIZE as f32, MAX_HEIGHT + 16.0),
                )) {
                    continue;
                }

                let lod =
                    (eye.distance(chunk_center.z0()).log2() - LOD_MIN_DIST_LOG2).max(0.0) as usize;
                let lod = lod.min(LOD - 1);

                assigned_lod[(y * self.w + x) as usize] = 1 + lod as u8;
            }
        }

        // generate the instances and the stitches thanks to the LOD data
        // if neighbor lod > our lod, we need to stitch
        // lod 0 means dont render
        let h = self.h as usize;
        let w = self.w as usize;
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let lod = assigned_lod[idx];
                if lod == 0 {
                    continue;
                }

                let stitch_right = (x + 1 != w) && (assigned_lod[idx + 1] > lod);
                let stitch_left = (x != 0) && (assigned_lod[idx - 1] > lod);
                let stitch_up = (y + 1 != h) && (assigned_lod[idx + w] > lod);
                let stitch_down = (y != 0) && (assigned_lod[idx - w] > lod);

                instances[lod as usize - 1].push(HeightmapInstance {
                    offset: vec2(x as f32, y as f32) * CSIZE as f32,
                    stitch_dir_flags: (stitch_right as u32)
                        | (stitch_up as u32) << 1
                        | (stitch_left as u32) << 2
                        | (stitch_down as u32) << 3,
                })
            }
        }

        for (lod, instance) in instances.into_iter().enumerate() {
            self.instances[lod].1 = instance.len() as u32;
            self.instances[lod]
                .0
                .write(fctx.gfx, bytemuck::cast_slice(&instance));
        }

        fctx.objs.push(Box::new(HeightmapPrepared {
            heightmapbgs: self.bgs.clone(),
            indices: self.indices.clone(),
            instances: self.instances.clone(),
        }));
    }

    fn generate_indices_mesh(gfx: &GfxContext) -> [(PBuffer, u32); LOD] {
        let mut indlod = vec![];

        for lod in 0..LOD {
            let scale = 1 << lod;
            let resolution = Self::LOD0_RESOLUTION / scale;

            let mut indices: Vec<IndexType> = Vec::with_capacity(6 * resolution * resolution);

            let resolution = resolution as IndexType;
            let w = resolution + 1;

            // iterate over the grid, adding two triangles for each cell
            for y in 0..resolution {
                for x in 0..resolution {
                    let idx = y * w + x;
                    // avoid aliasing by alternating the triangles
                    // alternate at 2 different levels (x + y) and (x / 2 + y / 2)
                    // because of the LOD interpolation (each rectangle might end up being 2 times smaller)
                    if (x + y + x / 2 + y / 2) % 2 == 0 {
                        indices.push(idx);
                        indices.push(idx + 1);
                        indices.push(idx + w);

                        indices.push(idx + 1);
                        indices.push(idx + w + 1);
                        indices.push(idx + w);
                        continue;
                    }
                    indices.push(idx);
                    indices.push(idx + 1);
                    indices.push(idx + w + 1);

                    indices.push(idx);
                    indices.push(idx + w + 1);
                    indices.push(idx + w);
                }
            }

            let l = indices.len();

            let mut buf = PBuffer::new(BufferUsages::INDEX);
            buf.write(gfx, bytemuck::cast_slice(&indices));
            indlod.push((buf, l as u32));
        }

        collect_arrlod(indlod)
    }

    /// Updates the normals of the heightmap and gen mipmaps
    pub fn invalidate_height_normals(&mut self, gfx: &GfxContext) {
        if cfg!(debug_assertions) {
            self.downsample_pipeline = resample_pipeline(gfx, &self.heightmap_tex, "downsample");
            self.upsample_pipeline = resample_pipeline(gfx, &self.heightmap_tex, "upsample");
            self.normal_pipeline = normal_pipeline(gfx, &self.normal_tex);
        }

        let mut encoder = gfx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("heightmap invalidate encoder"),
            });

        // downsample, starting from the base resolution
        for mip in UPSCALE_LOD as u32..LOD as u32 - 1 {
            downsample_update(
                gfx,
                &self.downsample_pipeline,
                &mut encoder,
                &self.heightmap_tex,
                mip,
            );
        }

        // upsample
        for mip in (1..=UPSCALE_LOD as u32).rev() {
            upsample_update(
                gfx,
                &self.upsample_pipeline,
                &mut encoder,
                &self.heightmap_tex,
                mip,
            );
        }

        // update normals
        for mip in 0..LOD as u32 {
            normal_update(
                gfx,
                &self.normal_pipeline,
                &mut encoder,
                &self.heightmap_tex.mip_view(mip),
                &self.normal_tex.mip_view(mip),
                &self.normal_unis[mip as usize],
            );
        }

        gfx.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn normal_pipeline(gfx: &GfxContext, normals_tex: &Texture) -> RenderPipeline {
    let normal_module = gfx.get_module("heightmap/calc_normals");

    gfx.device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("heightmap normals pipeline"),
            layout: Some(
                &gfx.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("heightmap normals pipeline layout"),
                        bind_group_layouts: &[
                            &gfx.device
                                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                                    label: None,
                                    entries: &[Texture::bindgroup_layout_entries(
                                        0,
                                        [TL::UInt].into_iter(),
                                    )
                                        // We don't need a sampler
                                        .next()
                                        .unwrap()],
                                }),
                            &Uniform::<f32>::bindgroup_layout(&gfx.device),
                        ],
                        push_constant_ranges: &[],
                    }),
            ),
            vertex: wgpu::VertexState {
                module: &normal_module,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &normal_module,
                entry_point: "calc_normals",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: normals_tex.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
}

fn normal_update(
    gfx: &GfxContext,
    normal_pipeline: &RenderPipeline,
    encoder: &mut CommandEncoder,
    height_tex: &TextureView,
    normal_view: &TextureView,
    uni: &Uniform<f32>,
) {
    let bg = gfx.device.create_bind_group(&BindGroupDescriptor {
        layout: &normal_pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(height_tex),
        }],
        label: None,
    });

    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("heightmap normals render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: normal_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    rp.set_pipeline(normal_pipeline);
    rp.set_bind_group(0, &bg, &[]);
    rp.set_bind_group(1, &uni.bg, &[]);
    rp.draw(0..4, 0..1);
    drop(rp);
}

fn resample_pipeline(gfx: &GfxContext, height_tex: &Texture, entry_point: &str) -> RenderPipeline {
    let resample_module = gfx.get_module("heightmap/resample");

    gfx.device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("heightmap downsample pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &resample_module,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &resample_module,
                entry_point,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: height_tex.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
}

/// Downsamples the heightmap 1 mip up, the mip argument should be the base level
fn downsample_update(
    gfx: &GfxContext,
    downsample_pipeline: &RenderPipeline,
    encoder: &mut CommandEncoder,
    height_tex: &Texture,
    mip: u32,
) {
    let bg = gfx.device.create_bind_group(&BindGroupDescriptor {
        layout: &downsample_pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&height_tex.mip_view(mip)),
        }],
        label: None,
    });

    let render_view = height_tex.mip_view(mip + 1);
    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("heightmap downsample render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &render_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    rp.set_pipeline(downsample_pipeline);
    rp.set_bind_group(0, &bg, &[]);
    rp.draw(0..4, 0..1);
    drop(rp);
}

/// Downsamples the heightmap 1 mip down, the mip argument should be the base level
fn upsample_update(
    gfx: &GfxContext,
    upsample_pipeline: &RenderPipeline,
    encoder: &mut CommandEncoder,
    height_tex: &Texture,
    mip: u32,
) {
    let bg = gfx.device.create_bind_group(&BindGroupDescriptor {
        layout: &upsample_pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&height_tex.mip_view(mip)),
        }],
        label: None,
    });

    let render_view = height_tex.mip_view(mip - 1);
    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("heightmap upsample render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &render_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    rp.set_pipeline(upsample_pipeline);
    rp.set_bind_group(0, &bg, &[]);
    rp.draw(0..4, 0..1);
    drop(rp);
}

#[derive(Hash)]
struct HeightmapPipeline {
    depth: bool,
    smap: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct HeightmapInstance {
    pub offset: Vec2,
    pub stitch_dir_flags: u32, // 4 lowest bits are 1 if we need to stitch in that direction. 0 = x+, 1 = y+, 2 = x-, 3 = y-
}
u8slice_impl!(HeightmapInstance);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HeightmapChunkData {
    lod: u32,                 // 0 = highest resolution, 1 = half resolution, etc.*
    lod_pow2: u32,            // 2^lod
    resolution: u32,          // width of the vertex grid
    distance_lod_cutoff: f32, // max distance at which to switch to the next lod to have smooth transitions
    cell_size: f32,
    inv_cell_size: f32,
}
u8slice_impl!(HeightmapChunkData);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];

impl HeightmapInstance {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

impl PipelineKey for HeightmapPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let heightmaplayout = gfx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &Texture::bindgroup_layout_entries(
                    0,
                    [TL::UInt, TL::UInt, TL::Float, TL::Float].into_iter(),
                )
                .chain(std::iter::once(
                    Uniform::<HeightmapChunkData>::bindgroup_layout_entry(8),
                ))
                .collect::<Vec<_>>(),
                label: Some("heightmap bindgroup layout"),
            });
        let vert = &mk_module("heightmap/heightmap.vert", &[]);

        if !self.depth {
            let frag = &mk_module("heightmap/heightmap.frag", &[]);

            return PipelineBuilder::color(
                "heightmap",
                &[
                    &gfx.render_params.layout,
                    &heightmaplayout,
                    &bg_layout_litmesh(&gfx.device),
                ],
                &[HeightmapInstance::desc()],
                vert,
                frag,
                gfx.sc_desc.format,
            )
            .with_samples(gfx.samples)
            .build(&gfx.device);
        }

        gfx.depth_pipeline_bglayout(
            &[HeightmapInstance::desc()],
            vert,
            None,
            self.smap,
            &[&gfx.render_params.layout, &heightmaplayout],
        )
    }
}

impl Drawable for HeightmapPrepared {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = gfx.get_pipeline(HeightmapPipeline {
            depth: false,
            smap: false,
        });

        rp.set_pipeline(pipeline);

        rp.set_bind_group(2, &gfx.simplelit_bg, &[]);

        self.set_buffers(rp);

        for lod in 0..LOD {
            let (_, n_instances) = &self.instances[lod];
            let (_, n_indices) = &self.indices[lod];

            gfx.perf
                .heightmap_drawcall(*n_indices as usize / 3 * *n_instances as usize)
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        if shadow_cascade.is_some() {
            // Heightmap don't cast shadows for now as they are hard to do properly
            // It needs separate frustrum culling + actual good shadow acne fix
            return;
        }
        rp.set_pipeline(gfx.get_pipeline(HeightmapPipeline {
            depth: true,
            smap: shadow_cascade.is_some(),
        }));

        self.set_buffers(rp);

        for lod in 0..LOD {
            let (_, n_instances) = &self.instances[lod];
            let (_, n_indices) = &self.indices[lod];

            gfx.perf.heightmap_depth_drawcall(
                *n_indices as usize / 3 * *n_instances as usize,
                shadow_cascade.is_some(),
            );
        }
    }
}

impl HeightmapPrepared {
    fn set_buffers<'a>(&'a self, rp: &mut RenderPass<'a>) {
        for lod in 0..LOD {
            let (instances, n_instances) = &self.instances[lod];
            if *n_instances == 0 {
                continue;
            }

            let (ind, n_indices) = &self.indices[lod];

            rp.set_bind_group(1, &self.heightmapbgs[lod], &[]);
            rp.set_vertex_buffer(0, instances.slice().unwrap());
            rp.set_index_buffer(ind.slice().unwrap(), IndexFormat::Uint32);
            rp.draw_indexed(0..*n_indices, 0, 0..*n_instances);
        }
    }
}

fn collect_arrlod<T>(x: impl IntoIterator<Item = T>) -> [T; LOD] {
    let mut iter = x.into_iter();
    [(); LOD].map(move |_| iter.next().expect("iterator too short"))
}
