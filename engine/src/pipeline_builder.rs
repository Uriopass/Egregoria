use wgpu::{
    BindGroupLayout, BlendState, DepthBiasState, Device, Face, FragmentState, FrontFace,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, TextureFormat, VertexBufferLayout, VertexState,
};

pub struct PipelineBuilder<'a> {
    pub descr: RenderPipelineDescriptor<'a>,
    pub layout: PipelineLayoutDescriptor<'a>,
    pub frag_shader: &'a ShaderModule,
    pub target_format: TextureFormat,
    pub blend: BlendState,
}

impl<'a> PipelineBuilder<'a> {
    pub fn color(
        label: &'static str,
        layouts: &'a [&'a BindGroupLayout],
        vertex_buffers: &'a [VertexBufferLayout<'a>],
        vert_shader: &'a ShaderModule,
        frag_shader: &'a ShaderModule,
        target_format: TextureFormat,
    ) -> Self {
        let render_pipeline_layout = PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: layouts,
            push_constant_ranges: &[],
        };

        Self {
            descr: RenderPipelineDescriptor {
                label: Some(label),
                layout: None,
                vertex: VertexState {
                    module: vert_shader,
                    entry_point: "vert",
                    compilation_options: Default::default(),
                    buffers: vertex_buffers,
                },
                fragment: None,
                primitive: PrimitiveState {
                    cull_mode: Some(Face::Back),
                    front_face: FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: DepthBiasState {
                        constant: 0,
                        slope_scale: 0.0,
                        clamp: 0.0,
                    },
                }),
                multisample: MultisampleState::default(),
                multiview: None,
            },
            layout: render_pipeline_layout,
            frag_shader,
            target_format,
            blend: BlendState::ALPHA_BLENDING,
        }
    }

    pub fn with_samples(mut self, samples: u32) -> Self {
        self.descr.multisample.count = samples;
        self
    }

    pub fn with_blend(mut self, blend: BlendState) -> Self {
        self.blend = blend;
        self
    }

    pub fn without_depth(mut self) -> Self {
        self.descr.depth_stencil = None;
        self
    }

    pub fn with_depth_write(mut self) -> Self {
        self.descr
            .depth_stencil
            .as_mut()
            .unwrap()
            .depth_write_enabled = true;
        self
    }

    pub fn build(self, device: &Device) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&self.layout);

        let color_states = [Some(wgpu::ColorTargetState {
            format: self.target_format,
            write_mask: wgpu::ColorWrites::ALL,
            blend: Some(self.blend),
        })];

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: self.descr.label,
            layout: Some(&render_pipeline_layout),
            vertex: self.descr.vertex,
            fragment: Some(FragmentState {
                module: self.frag_shader,
                entry_point: "frag",
                compilation_options: Default::default(),
                targets: &color_states,
            }),
            primitive: self.descr.primitive,
            depth_stencil: self.descr.depth_stencil,
            multisample: self.descr.multisample,
            multiview: self.descr.multiview,
        };

        device.create_render_pipeline(&render_pipeline_desc)
    }
}
