use crate::{compile_shader, Drawable, GfxContext, PreparedPipeline, Texture, UvVertex, VBDesc};
use wgpu::{BlendFactor, BlendOperation, RenderPass, TextureComponentType};

pub struct BlitLinear;

impl Drawable for BlitLinear {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline
    where
        Self: Sized,
    {
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("blit pipeline"),
                    bind_group_layouts: &[&Texture::bindgroup_layout(
                        &gfx.device,
                        TextureComponentType::Uint,
                    )],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx
            .device
            .create_shader_module(compile_shader("assets/shaders/blit_linear.vert", None).0);
        let fs_module = gfx
            .device
            .create_shader_module(compile_shader("assets/shaders/blit_linear.frag", None).0);

        let color_states = [wgpu::ColorStateDescriptor {
            format: gfx.sc_desc.format,
            color_blend: wgpu::BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &color_states,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[UvVertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        PreparedPipeline(gfx.device.create_render_pipeline(&render_pipeline_desc))
    }

    fn draw<'a>(&'a self, _gfx: &'a GfxContext, _rp: &mut RenderPass<'a>) {
        unimplemented!()
    }
}
