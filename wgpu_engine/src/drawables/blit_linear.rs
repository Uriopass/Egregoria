use crate::{GfxContext, Texture, UvVertex, VBDesc};
use wgpu::{BlendFactor, BlendOperation, MultisampleState, TextureSampleType};

pub struct BlitLinear;

impl BlitLinear {
    pub fn setup(gfx: &mut GfxContext) {
        return;
        gfx.register_pipeline::<Self>(
            &["blit_linear"],
            Box::new(move |m, gfx| {
                let render_pipeline_layout =
                    gfx.device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("blit pipeline"),
                            bind_group_layouts: &[&Texture::bindgroup_layout_complex(
                                &gfx.device,
                                TextureSampleType::Float { filterable: true },
                                1,
                                true,
                            )],
                            push_constant_ranges: &[],
                        });

                let blitlinearm = &m[0];

                let color_states = [Some(wgpu::ColorTargetState {
                    format: gfx.sc_desc.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })];

                let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: blitlinearm,
                        entry_point: "vert",
                        buffers: &[UvVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: blitlinearm,
                        entry_point: "frag",
                        targets: &color_states,
                    }),
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: MultisampleState {
                        count: 4,
                        ..Default::default()
                    },
                    multiview: None,
                };

                gfx.device.create_render_pipeline(&render_pipeline_desc)
            }),
        );
    }
}
