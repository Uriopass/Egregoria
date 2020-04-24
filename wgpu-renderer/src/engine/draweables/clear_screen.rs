use crate::engine::{
    compile_shader, CompiledShader, Context, Draweable, FrameContext, PreparedPipeline,
};

use lazy_static::*;
use wgpu::Color;

pub struct ClearScreen {
    pub clear_color: Color,
}

lazy_static! {
    static ref VERT_SHADER: CompiledShader = compile_shader("resources/shaders/empty_shader.vert");
}

impl Draweable for ClearScreen {
    fn create_pipeline(gfx: &Context) -> PreparedPipeline {
        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            });

        let module = gfx.device.create_shader_module(&VERT_SHADER.0);

        PreparedPipeline {
            pipeline: gfx
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    layout: &layout,
                    vertex_stage: wgpu::ProgrammableStageDescriptor {
                        module: &module,
                        entry_point: "main",
                    },
                    fragment_stage: None,
                    rasterization_state: None,
                    primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                    color_states: &[wgpu::ColorStateDescriptor {
                        format: gfx.sc_desc.format,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                    depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                        stencil_read_mask: 0,
                        stencil_write_mask: 0,
                    }),
                    vertex_state: wgpu::VertexStateDescriptor {
                        index_format: wgpu::IndexFormat::Uint16,
                        vertex_buffers: &[],
                    },
                    sample_count: 1,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                }),
            bindgroupslayouts: vec![],
        }
    }

    fn draw(&self, ctx: &mut FrameContext) {
        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &ctx.frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &ctx.gfx.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });
        render_pass.set_pipeline(&ctx.gfx.get_pipeline::<Self>().pipeline);
        render_pass.draw(0..0, 0..0);
    }
}
