use crate::{
    compile_shader, CompiledModule, GfxContext, PipelineKey, Texture, TextureBuilder, Uniform, TL,
};
use common::FastMap;
use geom::{Vec3, Vec4};
use wgpu::{
    BindGroup, BlendState, CommandEncoder, CommandEncoderDescriptor, Device, FragmentState, LoadOp,
    Operations, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerDescriptor, TextureFormat, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};

pub struct Pbr {
    pub environment_cube: Texture,
    environment_uniform: Uniform<Vec4>,
    environment_bg: BindGroup,
    pub specular_prefilter_cube: Texture,
    cube_views: Vec<TextureView>,
    specular_views: Vec<TextureView>,
    specular_uniforms: Vec<Uniform<SpecularParams>>,
    pub diffuse_irradiance_cube: Texture,
    diffuse_uniform: Uniform<DiffuseParams>,
    diffuse_views: Vec<TextureView>,
    pub split_sum_brdf_lut: Texture,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PbrPipeline {
    Environment,
    DiffuseIrradiance,
    SpecularPrefilter,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct SpecularParams {
    roughness: f32,
    time97: u32,
    sample_count: u32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
struct DiffuseParams {
    time97: u32,
    sample_count: u32,
}

u8slice_impl!(SpecularParams);
u8slice_impl!(DiffuseParams);

impl Pbr {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let environment_cube = TextureBuilder::empty(128, 128, 6, TextureFormat::Rgba16Float)
            .with_label("environment cubemap")
            .with_srgb(false)
            .with_sampler(Texture::linear_sampler())
            .build(device, queue);

        let diffuse_irradiance_cube = TextureBuilder::empty(16, 16, 6, TextureFormat::Rgba16Float)
            .with_label("irradiance cubemap")
            .with_srgb(false)
            .with_sampler(Texture::linear_sampler())
            .build(device, queue);

        let specular_prefilter_cube = TextureBuilder::empty(64, 64, 6, TextureFormat::Rgba16Float)
            .with_label("specular prefilter cubemap")
            .with_srgb(false)
            .with_sampler(Texture::linear_sampler())
            .with_fixed_mipmaps(5)
            .build(device, queue);

        let diffuse_uniform = Uniform::new(DiffuseParams::default(), device);
        let environment_uniform = Uniform::new(Vec4::default(), device);

        let mut cube_views = Vec::new();
        let mut diffuse_views = Vec::new();
        for face in 0..6 {
            cube_views.push(
                environment_cube
                    .texture
                    .create_view(&TextureViewDescriptor {
                        label: Some("environment cube view"),
                        format: None,
                        dimension: Some(TextureViewDimension::D2),
                        aspect: Default::default(),
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: face,
                        array_layer_count: None,
                    }),
            );
            diffuse_views.push(diffuse_irradiance_cube.texture.create_view(
                &TextureViewDescriptor {
                    label: Some("irradiance cube view"),
                    format: None,
                    dimension: Some(TextureViewDimension::D2),
                    aspect: Default::default(),
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: face,
                    array_layer_count: Some(1),
                },
            ));
        }

        let mut specular_views =
            Vec::with_capacity((specular_prefilter_cube.n_mips() * 6) as usize);
        let mut specular_uniforms = Vec::with_capacity(specular_prefilter_cube.n_mips() as usize);
        for mip in 0..specular_prefilter_cube.n_mips() {
            let params = Uniform::new(SpecularParams::default(), device);
            specular_uniforms.push(params);
            for face in 0..6 {
                specular_views.push(specular_prefilter_cube.texture.create_view(
                    &TextureViewDescriptor {
                        label: None,
                        format: None,
                        dimension: Some(TextureViewDimension::D2),
                        aspect: Default::default(),
                        base_mip_level: mip,
                        mip_level_count: Some(1),
                        base_array_layer: face,
                        array_layer_count: Some(1),
                    },
                ));
            }
        }

        let split_sum_brdf_lut = Self::make_split_sum_brdf_lut(device, queue);

        Self {
            environment_bg: environment_cube
                .bindgroup(device, &Texture::bindgroup_layout(device, [TL::Cube])),
            environment_cube,
            diffuse_irradiance_cube,
            specular_prefilter_cube,
            split_sum_brdf_lut,
            specular_uniforms,
            specular_views,
            cube_views,
            environment_uniform,
            diffuse_uniform,
            diffuse_views,
        }
    }

    pub fn update(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        self.write_environment_cubemap(gfx, gfx.render_params.value().sun, enc);
        self.write_diffuse_irradiance(gfx, enc);
        self.write_specular_prefilter(gfx, enc);
    }

    fn write_environment_cubemap(&self, gfx: &GfxContext, sun_pos: Vec3, enc: &mut CommandEncoder) {
        profiling::scope!("gfx::write_environment_cubemap");
        self.environment_uniform
            .write_direct(&gfx.queue, &sun_pos.normalize().w(0.0));
        let pipe = gfx.get_pipeline(PbrPipeline::Environment);

        for face in 0..6u32 {
            let view = &self.cube_views[face as usize];
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some(format!("environment cubemap face {face}").as_str()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Default::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &self.environment_uniform.bg, &[]);
            pass.draw(face * 6..face * 6 + 6, 0..1);
        }
    }

    pub fn write_diffuse_irradiance(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        profiling::scope!("gfx::write_diffuse_irradiance");
        let pipe = gfx.get_pipeline(PbrPipeline::DiffuseIrradiance);
        self.diffuse_uniform.write_direct(
            &gfx.queue,
            &DiffuseParams {
                time97: ((gfx.tick * 7) % 97) as u32,
                sample_count: if gfx.tick == 0 { 1024 } else { 32 },
            },
        );
        for face in 0..6u32 {
            let view = &self.diffuse_views[face as usize];
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some(format!("diffuse irradiance face {face}").as_str()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &self.environment_bg, &[]);
            pass.set_bind_group(1, &self.diffuse_uniform.bg, &[]);
            pass.draw(face * 6..face * 6 + 6, 0..1);
        }
    }

    pub fn write_specular_prefilter(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        profiling::scope!("gfx::write_specular_prefilter");
        let pipe = gfx.get_pipeline(PbrPipeline::SpecularPrefilter);
        for mip in 0..self.specular_prefilter_cube.n_mips() {
            let roughness = mip as f32 / (self.specular_prefilter_cube.n_mips() - 1) as f32;
            let uni = &self.specular_uniforms[mip as usize];
            uni.write_direct(
                &gfx.queue,
                &SpecularParams {
                    roughness,
                    time97: ((gfx.tick * 7) % 97) as u32,
                    sample_count: if gfx.tick == 0 { 1024 } else { 60 },
                },
            );
            for face in 0..6 {
                let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                    label: Some(format!("specular prefilter face {face} mip {mip}").as_str()),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &self.specular_views[mip as usize * 6 + face as usize],
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &self.environment_bg, &[]);
                pass.set_bind_group(1, &uni.bg, &[]);
                pass.draw(face * 6..face * 6 + 6, 0..1);
            }
        }
    }

    fn make_split_sum_brdf_lut(device: &Device, queue: &Queue) -> Texture {
        let brdf_tex = TextureBuilder::empty(512, 512, 1, TextureFormat::Rg16Float)
            .with_label("brdf split sum lut")
            .with_srgb(false)
            .with_sampler(SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                ..Texture::linear_sampler()
            })
            .build(device, queue);

        let pipelayout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let brdf_convolution_module =
            compile_shader(device, "pbr/brdf_convolution", &FastMap::default());

        let cubemapline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipelayout),
            vertex: VertexState {
                module: &brdf_convolution_module,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &brdf_convolution_module,
                entry_point: "frag",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Rg16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let mut enc = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("brd lut encoder"),
        });

        let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
            label: Some("brdf lut"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &brdf_tex.view,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&cubemapline);
        pass.draw(0..4, 0..1);
        drop(pass);

        queue.submit(Some(enc.finish()));

        brdf_tex
    }
}

impl PipelineKey for PbrPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        match self {
            PbrPipeline::Environment => {
                let cubemap_vert = &mk_module("to_cubemap.vert", &[]);
                let cubemap_frag = &mk_module("atmosphere_cubemap.frag", &[]);
                let cubemappipelayout =
                    gfx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[&Uniform::<()>::bindgroup_layout(&gfx.device)],
                            push_constant_ranges: &[],
                        });

                gfx.device
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&cubemappipelayout),
                        vertex: VertexState {
                            module: cubemap_vert,
                            entry_point: "vert",
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: TextureFormat::Rgba16Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    })
            }
            PbrPipeline::DiffuseIrradiance => {
                let cubemap_vert = &mk_module("to_cubemap.vert", &[]);
                let cubemap_frag = &mk_module("pbr/convolute_diffuse_irradiance.frag", &[]);
                let bg_layout = Texture::bindgroup_layout(&gfx.device, [TL::Cube]);
                let params_layout = Uniform::<()>::bindgroup_layout(&gfx.device);

                let cubemappipelayout =
                    &gfx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[&bg_layout, &params_layout],
                            push_constant_ranges: &[],
                        });

                gfx.device
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        label: None,
                        layout: Some(cubemappipelayout),
                        vertex: VertexState {
                            module: cubemap_vert,
                            entry_point: "vert",
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: TextureFormat::Rgba16Float,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    })
            }
            PbrPipeline::SpecularPrefilter => {
                let cubemap_vert = &mk_module("to_cubemap.vert", &[]);
                let cubemap_frag = &mk_module("pbr/specular_prefilter.frag", &[]);
                let bg_layout = Texture::bindgroup_layout(&gfx.device, [TL::Cube]);
                let params_layout = Uniform::<()>::bindgroup_layout(&gfx.device);

                let cubemappipelayout =
                    &gfx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[&bg_layout, &params_layout],
                            push_constant_ranges: &[],
                        });

                gfx.device
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        label: None,
                        layout: Some(cubemappipelayout),
                        vertex: VertexState {
                            module: cubemap_vert,
                            entry_point: "vert",
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: TextureFormat::Rgba16Float,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    })
            }
        }
    }
}
