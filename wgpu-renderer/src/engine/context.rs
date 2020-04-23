use winit::window::Window;

use crate::engine::{Draweable, PreparedPipeline, Texture};
use std::any::TypeId;
use std::collections::HashMap;
use wgpu::{
    Adapter, CommandBuffer, CommandEncoderDescriptor, Device, Queue, Surface, SwapChain,
    SwapChainDescriptor, SwapChainOutput,
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
    pub pipelines: HashMap<TypeId, PreparedPipeline>,
    pub cur_frame: Option<SwapChainOutput>,
    pub queue_buffer: Vec<CommandBuffer>,
}

pub struct FrameContext<'a> {
    pub encoder: wgpu::CommandEncoder,
    pub frame: wgpu::SwapChainOutput,
    pub gfx: &'a GfxContext,
}

impl FrameContext<'_> {
    pub fn finish(self) {
        self.gfx.queue.submit(&[self.encoder.finish()]);
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

    pub fn begin_frame(&mut self) -> FrameContext {
        let tex = self
            .swapchain
            .get_next_texture()
            .expect("Timeout getting texture");

        FrameContext {
            gfx: self,
            encoder: self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                }),
            frame: tex,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = (new_size.width, new_size.height);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.cur_frame = None;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.sc_desc);
    }

    pub fn get_pipeline<T: Draweable>(&self) -> &PreparedPipeline {
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
