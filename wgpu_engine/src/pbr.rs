use crate::{
    compile_shader, CompiledModule, GfxContext, PipelineBuilder, Texture, TextureBuilder, Uniform,
    TL,
};
use geom::{Vec3, Vec4};
use std::num::NonZeroU32;
use wgpu::{
    BlendState, CommandEncoder, CommandEncoderDescriptor, Device, FragmentState, LoadOp,
    Operations, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerDescriptor, TextureFormat, TextureViewDescriptor, TextureViewDimension, VertexState,
};

pub struct PBR {
    pub environment_cube: Texture,
    pub specular_prefilter_cube: Texture,
    pub diffuse_irradiance_cube: Texture,
    pub split_sum_brdf_lut: Texture,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PBRPipeline {
    Environment,
    DiffuseIrradiance,
    SpecularPrefilter,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct SpecularParams {
    roughness: f32,
    time100: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct DiffuseParams {
    time100: u32,
}

u8slice_impl!(SpecularParams);
u8slice_impl!(DiffuseParams);

impl PBR {
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

        let split_sum_brdf_lut = Self::make_split_sum_brdf_lut(device, queue);

        Self {
            environment_cube,
            diffuse_irradiance_cube,
            specular_prefilter_cube,
            split_sum_brdf_lut,
        }
    }

    pub fn update(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        self.write_environment_cubemap(gfx, gfx.render_params.value().sun, enc);
        self.write_diffuse_irradiance(gfx, enc);
        self.write_specular_prefilter(gfx, enc);
    }

    fn write_environment_cubemap(&self, gfx: &GfxContext, sun_pos: Vec3, enc: &mut CommandEncoder) {
        let sun_pos: Uniform<Vec4> = Uniform::new(sun_pos.normalize().w(0.0), &gfx.device);
        let pipe = gfx.get_pipeline(PBRPipeline::Environment);

        for face in 0..6 {
            let view = self
                .environment_cube
                .texture
                .create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: Some(TextureViewDimension::D2),
                    aspect: Default::default(),
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: face,
                    array_layer_count: None,
                });
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some(format!("environment cubemap face {face}").as_str()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Default::default(),
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &sun_pos.bindgroup, &[]);
            pass.draw(face * 6..face * 6 + 6, 0..1);
        }
    }

    pub fn write_diffuse_irradiance(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        let pipe = gfx.get_pipeline(PBRPipeline::DiffuseIrradiance);
        let bg = self
            .environment_cube
            .bindgroup(&gfx.device, &pipe.get_bind_group_layout(0));
        let params = Uniform::new(
            DiffuseParams {
                time100: (gfx.tick % 100) as u32,
            },
            &gfx.device,
        );
        for face in 0..6 {
            let view = self
                .diffuse_irradiance_cube
                .texture
                .create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: Some(TextureViewDimension::D2),
                    aspect: Default::default(),
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: face,
                    array_layer_count: None,
                });
            let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                label: Some(format!("diffuse irradiance face {face}").as_str()),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &bg, &[]);
            pass.set_bind_group(1, &params.bindgroup, &[]);
            pass.draw(face * 6..face * 6 + 6, 0..1);
        }
    }

    pub fn write_specular_prefilter(&self, gfx: &GfxContext, enc: &mut CommandEncoder) {
        let pipe = gfx.get_pipeline(PBRPipeline::SpecularPrefilter);
        let bg = self
            .environment_cube
            .bindgroup(&gfx.device, &pipe.get_bind_group_layout(0));
        for mip in 0..self.specular_prefilter_cube.n_mips() {
            let roughness = mip as f32 / (self.specular_prefilter_cube.n_mips() - 1) as f32;
            for face in 0..6 {
                let params = Uniform::new(
                    SpecularParams {
                        roughness,
                        time100: (gfx.tick % 100) as u32,
                    },
                    &gfx.device,
                );
                let view =
                    self.specular_prefilter_cube
                        .texture
                        .create_view(&TextureViewDescriptor {
                            label: None,
                            format: None,
                            dimension: Some(TextureViewDimension::D2),
                            aspect: Default::default(),
                            base_mip_level: mip,
                            mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                            base_array_layer: face,
                            array_layer_count: None,
                        });
                let mut pass = enc.begin_render_pass(&RenderPassDescriptor {
                    label: Some(format!("specular prefilter face {face} mip {mip}").as_str()),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &bg, &[]);
                pass.set_bind_group(1, &params.bindgroup, &[]);
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

        let brdf_convolution_module = compile_shader(device, "pbr/brdf_convolution");

        let cubemapline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipelayout),
            vertex: VertexState {
                module: &brdf_convolution_module,
                entry_point: "vert",
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
        });
        pass.set_pipeline(&cubemapline);
        pass.draw(0..4, 0..1);
        drop(pass);

        queue.submit(Some(enc.finish()));

        brdf_tex
    }
}

impl PipelineBuilder for PBRPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        match self {
            PBRPipeline::Environment => {
                let cubemap_vert = &mk_module("to_cubemap.vert");
                let cubemap_frag = &mk_module("atmosphere_cubemap.frag");
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
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: TextureFormat::Rgba16Float,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    })
            }
            PBRPipeline::DiffuseIrradiance => {
                let cubemap_vert = &mk_module("to_cubemap.vert");
                let cubemap_frag = &mk_module("pbr/convolute_diffuse_irradiance.frag");
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
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: TextureFormat::Rgba16Float,
                                blend: Some(BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview: None,
                    })
            }
            PBRPipeline::SpecularPrefilter => {
                let cubemap_vert = &mk_module("to_cubemap.vert");
                let cubemap_frag = &mk_module("pbr/specular_prefilter.frag");
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
                            buffers: &[],
                        },
                        primitive: Default::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(FragmentState {
                            module: cubemap_frag,
                            entry_point: "frag",
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
