use crate::engine::{AudioContext, FrameContext, GfxContext, InputContext};
use crate::game_loop;
use crate::rendering::imgui_wrapper::GuiRenderContext;
use futures::executor;
use wgpu::{Color, CommandEncoderDescriptor, SwapChainOutput};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(dead_code)]
pub struct Context {
    pub gfx: GfxContext,
    pub input: InputContext,
    pub audio: AudioContext,
}

impl Context {
    pub fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let size = event_loop.primary_monitor().size();

        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(
                size.width as f32 * 0.8,
                size.height as f32 * 0.8,
            ))
            .build(&event_loop)
            .expect("Failed to create window");

        let gfx = executor::block_on(GfxContext::new(window));
        let input = InputContext::default();
        let audio = AudioContext::new(2);

        (Self { gfx, input, audio }, event_loop)
    }

    pub fn start(mut self, mut state: game_loop::State<'static>, el: EventLoop<()>) {
        let clear_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let mut frame: Option<SwapChainOutput> = None;
        let mut new_size: Option<PhysicalSize<u32>> = None;

        el.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            state.event(&self.gfx, &event);
            match event {
                Event::WindowEvent { event, .. } => {
                    let managed = self.input.handle(&event);

                    if !managed {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                new_size = Some(physical_size);
                            }
                            WindowEvent::CloseRequested => {
                                println!("The close button was pressed. stopping");
                                *control_flow = ControlFlow::Exit
                            }
                            _ => (),
                        }
                    }
                }
                Event::MainEventsCleared => {
                    if frame.is_none() {
                        if let Some(new_size) = new_size.take() {
                            self.gfx.resize(new_size);
                            state.resized(&mut self, new_size);
                        }
                        frame = Some(
                            self.gfx
                                .swapchain
                                .get_next_texture()
                                .expect("Timeout getting texture"),
                        );
                    } else {
                        self.input.mouse.unprojected = state.unproject(self.input.mouse.screen);

                        state.update(&mut self);

                        let mut encoder =
                            self.gfx
                                .device
                                .create_command_encoder(&CommandEncoderDescriptor {
                                    label: Some("Render encoder"),
                                });

                        self.gfx
                            .projection
                            .upload_to_gpu(&self.gfx.device, &mut encoder);

                        let mut objs = vec![];

                        let frame = frame.take().unwrap();
                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    color_attachments: &[
                                        wgpu::RenderPassColorAttachmentDescriptor {
                                            attachment: &self.gfx.multi_frame,
                                            resolve_target: Some(&frame.view),
                                            load_op: wgpu::LoadOp::Clear,
                                            store_op: wgpu::StoreOp::Store,
                                            clear_color: wgpu::Color {
                                                r: clear_color.r,
                                                g: clear_color.g,
                                                b: clear_color.b,
                                                a: clear_color.a,
                                            },
                                        },
                                    ],
                                    depth_stencil_attachment: Some(
                                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                            attachment: &self.gfx.depth_texture.view,
                                            depth_load_op: wgpu::LoadOp::Clear,
                                            depth_store_op: wgpu::StoreOp::Store,
                                            clear_depth: 0.0,
                                            stencil_load_op: wgpu::LoadOp::Clear,
                                            stencil_store_op: wgpu::StoreOp::Store,
                                            clear_stencil: 0,
                                        },
                                    ),
                                });

                            let mut fc = FrameContext {
                                objs: &mut objs,
                                gfx: &self.gfx,
                            };
                            state.render(&mut fc);
                            for obj in fc.objs {
                                obj.draw(&self.gfx, &mut render_pass);
                            }
                        }

                        state.render_gui(GuiRenderContext {
                            device: &self.gfx.device,
                            encoder: &mut encoder,
                            frame_view: &frame.view,
                            window: &self.gfx.window,
                        });

                        self.gfx.queue.submit(&[encoder.finish()]);

                        self.input.end_frame();
                    }
                }
                _ => (),
            }
        })
    }
}
