use winit::window::Window;

use crate::engine::render_pass::Draweable;
use crate::engine::texture::Texture;
use std::any::TypeId;
use std::collections::HashMap;
use wgpu::{
    Adapter, CommandBuffer, Device, Queue, RenderPipeline, Surface, SwapChain, SwapChainDescriptor,
    SwapChainOutput,
};

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
    pub pipelines: HashMap<TypeId, RenderPipeline>,
    pub cur_frame: Option<SwapChainOutput>,
    pub queue_buffer: Vec<CommandBuffer>,
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
        let swapchain = device.create_swap_chain(&surface, &sc_desc);
        let depth_texture = Texture::create_depth_texture(&device, &sc_desc);

        Self {
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
            cur_frame: None,
            queue_buffer: vec![],
        }
    }

    pub fn begin_frame(&mut self) {
        self.cur_frame = Some(
            self.swapchain
                .get_next_texture()
                .expect("Timeout getting texture"),
        );
    }

    pub fn end_frame(&mut self) {
        self.queue.submit(&self.queue_buffer);
        self.queue_buffer.clear();
        self.cur_frame = None; // drops the old swapchain texture
    }

    pub fn draw(&mut self, x: &impl Draweable) {
        self.queue_buffer.push(x.draw(self))
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = (new_size.width, new_size.height);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.cur_frame = None;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.sc_desc);
    }

    pub fn get_pipeline<T: Draweable>(&self) -> &wgpu::RenderPipeline {
        &self
            .pipelines
            .get(&std::any::TypeId::of::<T>())
            .expect("Pipeline was not registered in context")
    }

    pub fn register_pipeline<T: Draweable>(&mut self) {
        self.pipelines
            .insert(std::any::TypeId::of::<T>(), T::create_pipeline(self));
    }
}
