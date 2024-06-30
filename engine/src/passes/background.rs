use crate::{
    CompiledModule, GfxContext, PipelineKey, RenderParams, Texture, Uniform, UvVertex, TL,
};
use wgpu::{
    BindGroupLayout, BlendState, CommandEncoder, DepthBiasState, FragmentState, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, TextureFormat, TextureView,
    VertexState,
};

pub fn render_background(gfx: &GfxContext, enc: &mut CommandEncoder, frame: &TextureView) {
    profiling::scope!("bg pass");
    let ops = wgpu::Operations {
        load: wgpu::LoadOp::Load, // Don't clear! We're drawing after main pass
        store: wgpu::StoreOp::Store,
    };

    let attachment = if gfx.samples > 1 {
        RenderPassColorAttachment {
            view: &gfx.fbos.color_msaa,
            resolve_target: Some(frame),
            ops,
        }
    } else {
        RenderPassColorAttachment {
            view: frame,
            resolve_target: None,
            ops,
        }
    };

    let mut bg_pass = enc.begin_render_pass(&RenderPassDescriptor {
        label: Some("bg pass"),
        color_attachments: &[Some(attachment)],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &gfx.fbos.depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    bg_pass.set_pipeline(gfx.get_pipeline(BackgroundPipeline));
    bg_pass.set_bind_group(0, &gfx.render_params.bg, &[]);
    bg_pass.set_bind_group(1, &gfx.bnoise_bg, &[]);
    bg_pass.set_bind_group(2, &gfx.sky_bg, &[]);
    bg_pass.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
    bg_pass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
    bg_pass.draw_indexed(0..6, 0, 0..1);
}

#[derive(Hash)]
pub struct BackgroundPipeline;

impl BackgroundPipeline {
    pub fn bglayout_texs(gfx: &GfxContext) -> BindGroupLayout {
        Texture::bindgroup_layout(&gfx.device, [TL::Float, TL::Float, TL::Cube])
    }
}

impl PipelineKey for BackgroundPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let bg = &mk_module("background", &[]);

        let render_pipeline_layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("background"),
                bind_group_layouts: &[
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    &Texture::bindgroup_layout(&gfx.device, [TL::Float]),
                    &Self::bglayout_texs(gfx),
                ],
                push_constant_ranges: &[],
            });

        let color_states = [Some(wgpu::ColorTargetState {
            format: gfx.sc_desc.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::COLOR,
        })];

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: Some("background pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: bg,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: bg,
                entry_point: "frag",
                compilation_options: Default::default(),
                targets: &color_states,
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: gfx.samples,
                ..Default::default()
            },
            multiview: None,
        };
        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }
}
