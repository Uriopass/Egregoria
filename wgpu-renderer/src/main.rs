#![allow(dead_code)]

use crate::engine::{Texture, TexturedMesh, TexturedMeshBuilder, UvVertex};
use engine::{ClearScreen, Draweable, GfxContext, IndexType, Mesh, RainbowMesh, Vertex};
use futures::executor;
use wgpu::Color;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod engine;

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
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[2, 1, 0, 3, 2, 0];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(500, 500))
        .build(&event_loop)
        .expect("Failed to create window");
    let mut ctx = executor::block_on(GfxContext::new(window));

    ctx.register_pipeline::<Mesh>();
    ctx.register_pipeline::<RainbowMesh>();
    ctx.register_pipeline::<TexturedMesh>();
    ctx.register_pipeline::<ClearScreen>();

    let tex = Texture::from_path(&ctx, "resources/car.png").expect("couldn't load car");

    let mut mb = TexturedMeshBuilder::new();
    mb.extend(&UV_VERTICES, &UV_INDICES);
    let mesh = mb.build(&ctx, tex);

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
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    ctx.resize(physical_size);
                }
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed. stopping");
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let mut frame = ctx.begin_frame();
                clear_screen.draw(&mut frame);
                mesh.draw(&mut frame);
                frame.finish();
            }
            _ => (),
        }
    })
}
