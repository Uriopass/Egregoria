use crate::{
    CompiledModule, GfxContext, PipelineKey, RenderParams, Texture, Uniform, UvVertex, TL,
};
use wgpu::{
    BlendComponent, BlendState, CommandEncoder, FragmentState, IndexFormat,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, VertexState,
};

#[derive(Copy, Clone, Hash)]
pub struct FogPipeline;

pub fn render_fog(gfx: &GfxContext, enc: &mut CommandEncoder) {
    //if !gfx.defines.contains_key("FOG") {
    //    return;
    //}
    profiling::scope!("fog");
    let pipeline = gfx.get_pipeline(FogPipeline);

    let mut fog_pass = enc.begin_render_pass(&RenderPassDescriptor {
        label: Some("fog pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &gfx.fbos.fog.view,
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

    fog_pass.set_pipeline(pipeline);
    fog_pass.set_bind_group(0, &gfx.render_params.bg, &[]);
    fog_pass.set_bind_group(1, &gfx.fbos.depth_bg, &[]);
    fog_pass.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
    fog_pass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
    fog_pass.draw_indexed(0..6, 0, 0..1);
}

impl PipelineKey for FogPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let render_pipeline_layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("fog pipeline"),
                bind_group_layouts: &[
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
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
            format: gfx.fbos.fog.format,
            write_mask: wgpu::ColorWrites::ALL,
            blend: Some(BlendState {
                color: BlendComponent::REPLACE,
                alpha: BlendComponent::REPLACE,
            }),
        })];

        let fog = mk_module("fog", &[]);

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &fog,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &fog,
                entry_point: "frag",
                compilation_options: Default::default(),
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
