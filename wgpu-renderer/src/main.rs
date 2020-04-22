use futures::executor;
use wgpu::Color;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::engine::draweables::{ClearScreen, IndexType, Mesh, MeshBuilder};
use engine::context::GfxContext;
use engine::vertex::Vertex;

mod engine;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.1],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.1],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.1],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        color: [0.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 0.0, 0.0, 1.0],
    },
];

const INDICES: &[IndexType] = &[0, 1, 2, 3, 4, 5];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(500, 500))
        .build(&event_loop)
        .expect("Failed to create window");
    let mut ctx = executor::block_on(GfxContext::new(window));

    ctx.register_pipeline::<Mesh>();
    ctx.register_pipeline::<ClearScreen>();

    let mut mb = MeshBuilder::new();
    mb.extend(&VERTICES, &INDICES);
    let mesh = mb.build(&ctx);

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
                ctx.begin_frame();
                ctx.draw(&clear_screen);
                ctx.draw(&mesh);
                ctx.end_frame();
            }
            _ => (),
        }
    })
}
