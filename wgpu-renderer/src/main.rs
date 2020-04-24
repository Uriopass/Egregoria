#![allow(dead_code)]

use crate::engine::{FrameContext, Texture, TexturedMesh, TexturedMeshBuilder, UvVertex};
use crate::rendering::CameraHandler;
use cgmath::Vector2;
use engine::{ClearScreen, Context, Draweable, IndexType, Mesh, RainbowMesh, Vertex};
use futures::executor;
use wgpu::Color;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod engine;
mod geometry;
mod rendering;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.2],
        color: [0.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.2],
        color: [0.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.2],
        color: [0.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.0, 0.1],
        color: [0.5, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.1],
        color: [0.5, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.5, 0.1],
        color: [0.5, 0.0, 0.0, 1.0],
    },
];

const INDICES: &[IndexType] = &[2, 1, 0, 5, 4, 3, 8, 7, 6];

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-50.0, -50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [150.0, -50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [150.0, 50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [-50.0, 50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[2, 1, 0, 3, 2, 0];

struct State {
    camera: CameraHandler,
    mesh: TexturedMesh,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.size.0 as f32, ctx.size.1 as f32);
        let tex = Texture::from_path(&ctx, "resources/car.png").expect("couldn't load car");

        let mut mb = TexturedMeshBuilder::new();
        mb.extend(&UV_VERTICES, &UV_INDICES);
        let mesh = mb.build(&ctx, tex);

        Self { camera, mesh }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.camera
            .easy_camera_movement(ctx, 1.0 / 30.0, true, true);
        self.camera.update(ctx);
    }

    fn render(&mut self, ctx: &mut FrameContext) {
        self.mesh.draw(ctx);
    }

    fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }

    fn unproject(&mut self, pos: Vector2<f32>) -> Vector2<f32> {
        self.camera.unproject_mouse_click(pos)
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(500, 500))
        .build(&event_loop)
        .expect("Failed to create window");

    let mut ctx = executor::block_on(Context::new(window));

    ctx.register_pipeline::<Mesh>();
    ctx.register_pipeline::<RainbowMesh>();
    ctx.register_pipeline::<TexturedMesh>();
    ctx.register_pipeline::<ClearScreen>();

    let mut state = State::new(&mut ctx);

    let clear_screen = ClearScreen {
        clear_color: Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        },
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                let managed = ctx.input.handle(&event);

                if !managed {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            ctx.resize(physical_size);
                            state.resized(&mut ctx, physical_size);
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
                ctx.input.mouse.unprojected = state.unproject(ctx.input.mouse.screen);

                state.update(&mut ctx);
                let mut frame = ctx.begin_frame();
                clear_screen.draw(&mut frame);
                state.render(&mut frame);
                frame.finish();

                ctx.input.end_frame();
            }
            _ => (),
        }
    })
}
