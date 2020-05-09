use crate::engine::{
    Drawable, HasPipeline, Mesh, PreparedPipeline, SpriteBatch, Texture, TexturedMesh, Uniform,
};
use cgmath::SquareMatrix;
use std::any::TypeId;
use std::collections::HashMap;
use wgpu::{
    Adapter, CommandBuffer, Device, Queue, ShaderStage, Surface, SwapChain, SwapChainDescriptor,
};
use winit::window::Window;

#[allow(dead_code)]
pub struct GfxContext {
    pub surface: Surface,
    pub size: (u32, u32),
    pub window: Window,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub swapchain: SwapChain,
    pub depth_texture: Texture,
    pub sc_desc: SwapChainDescriptor,
    pub pipelines: HashMap<TypeId, PreparedPipeline>,
    pub queue_buffer: Vec<CommandBuffer>,
    pub projection: Uniform<cgmath::Matrix4<f32>>,
    pub projection_layout: wgpu::BindGroupLayout,
    pub samples: u32,
    pub multi_frame: wgpu::TextureView,
}

pub struct FrameContext<'a> {
    pub gfx: &'a GfxContext,
    pub objs: &'a mut Vec<Box<dyn Drawable>>,
}

impl<'a> FrameContext<'a> {
    pub fn draw(&mut self, v: impl Drawable + 'static) {
        self.objs.push(Box::new(v))
    }
}

impl GfxContext {
    pub async fn new(window: Window) -> Self {
        let (win_width, win_height) = (window.inner_size().width, window.inner_size().height);
        let surface = Surface::create(&window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Failed to find a suitable adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let samples = 4;
        let swapchain = device.create_swap_chain(&surface, &sc_desc);
        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, samples);

        let projection_layout =
            Uniform::<cgmath::Matrix4<f32>>::bindgroup_layout(&device, 0, ShaderStage::VERTEX);
        let projection = Uniform::new(cgmath::Matrix4::identity(), &device, &projection_layout);

        let multi_frame = Self::create_multisampled_framebuffer(&sc_desc, &device, samples);

        let mut me = Self {
            size: (win_width, win_height),
            swapchain,
            device,
            queue,
            sc_desc,
            adapter,
            depth_texture,
            surface,
            pipelines: HashMap::new(),
            window,
            queue_buffer: vec![],
            projection,
            projection_layout,
            samples,
            multi_frame,
        };

        me.register_pipeline::<Mesh>();
        me.register_pipeline::<TexturedMesh>();
        me.register_pipeline::<SpriteBatch>();

        me
    }

    pub fn set_proj(&mut self, proj: cgmath::Matrix4<f32>) {
        self.projection.value = proj;
    }

    fn create_multisampled_framebuffer(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("multisampled frame descriptor"),
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_default_view()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = (new_size.width, new_size.height);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.sc_desc, self.samples);
        self.multi_frame =
            Self::create_multisampled_framebuffer(&self.sc_desc, &self.device, self.samples);
    }

    pub fn get_pipeline<T: 'static + Drawable>(&self) -> &PreparedPipeline {
        &self
            .pipelines
            .get(&std::any::TypeId::of::<T>())
            .expect("Pipeline was not registered in context")
    }

    pub fn register_pipeline<T: HasPipeline>(&mut self) {
        self.pipelines
            .insert(std::any::TypeId::of::<T>(), T::create_pipeline(self));
    }
}
