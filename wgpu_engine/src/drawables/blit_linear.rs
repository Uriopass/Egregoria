use crate::{compile_shader, GfxContext, Texture, UvVertex, VBDesc};
use wgpu::{BlendFactor, BlendOperation};

pub struct BlitLinear;

impl BlitLinear {
    pub fn setup(gfx: &mut GfxContext) {
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("blit pipeline"),
                    bind_group_layouts: &[&Texture::bindgroup_layout(&gfx.device)],
                    push_constant_ranges: &[],
                });

        let vs_module = compile_shader(&gfx.device, "assets/shaders/blit_linear.vert", None).0;
        let fs_module = compile_shader(&gfx.device, "assets/shaders/blit_linear.frag", None).0;

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
                module: &vs_module,
                entry_point: "main",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &color_states,
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        };
        let pipe = gfx.device.create_render_pipeline(&render_pipeline_desc);
        gfx.register_pipeline::<Self>(pipe);
    }
}
