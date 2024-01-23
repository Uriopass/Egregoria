use crate::{CompiledModule, GfxContext, PipelineBuilder, Texture, UvVertex, TL};
use wgpu::{
    BlendComponent, BlendState, CommandEncoder, FragmentState, IndexFormat,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, VertexState,
};

#[derive(Copy, Clone, Hash)]
pub struct SSAOPipeline;

pub fn render_ssao(gfx: &GfxContext, enc: &mut CommandEncoder) {
    profiling::scope!("ssao");
    let pipeline = gfx.get_pipeline(SSAOPipeline);
    let bg = gfx
        .fbos
        .depth
        .bindgroup(&gfx.device, &pipeline.get_bind_group_layout(1));

    let mut ssao_pass = enc.begin_render_pass(&RenderPassDescriptor {
        label: Some("ssao pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &gfx.fbos.ssao.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    ssao_pass.set_pipeline(pipeline);
    ssao_pass.set_bind_group(0, &gfx.render_params.bindgroup, &[]);
    ssao_pass.set_bind_group(1, &bg, &[]);
    ssao_pass.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
    ssao_pass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
    ssao_pass.draw_indexed(0..6, 0, 0..1);
}

impl PipelineBuilder for SSAOPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        let render_pipeline_layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("ssao pipeline"),
                bind_group_layouts: &[
                    &gfx.render_params.layout,
                    &Texture::bindgroup_layout(
                        &gfx.device,
                        [if gfx.samples > 1 {
                            TL::NonfilterableFloatMultisampled
                        } else {
                            TL::NonfilterableFloat
                        }],
                    ),
                ],
                push_constant_ranges: &[],
            });

        let color_states = [Some(wgpu::ColorTargetState {
            format: gfx.fbos.ssao.format,
            write_mask: wgpu::ColorWrites::ALL,
            blend: Some(BlendState {
                color: BlendComponent::REPLACE,
                alpha: BlendComponent::REPLACE,
            }),
        })];

        let ssao = mk_module("ssao");

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &ssao,
                entry_point: "vert",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &ssao,
                entry_point: "frag",
                targets: &color_states,
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        };

        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }
}
