#[path = "../src/shader.rs"]
mod shader;
#[path = "../src/surface.rs"]
mod surface;
#[path = "../src/vertex.rs"]
mod vertex;

use std::path::PathBuf;

use futures::executor;
use wgpu::{Color, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::shader::ShaderDescriptor;
use crate::surface::{BufferUsageDescriptor, PipelineDescriptor, RenderPassDescriptor, Surface};
use crate::vertex::Vertex;

fn wgpu_color(r: f64, g: f64, b: f64, a: f64) -> Color {
    Color { r, g, b, a }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, -0.49240386, 0.0],
        color: [0.5, 0.0, 0.5, 1.0],
    }, // A
    Vertex {
        position: [-0.49513406, -0.06958647, 0.0],
        color: [0.5, 0.0, 0.5, 1.0],
    }, // B
    Vertex {
        position: [-0.21918549, 0.44939706, 0.0],
        color: [0.5, 0.0, 0.5, 1.0],
    }, // C
    Vertex {
        position: [0.35966998, 0.3473291, 0.0],
        color: [0.5, 0.0, 0.5, 1.0],
    }, // D
    Vertex {
        position: [0.44147372, -0.2347359, 0.0],
        color: [0.5, 0.0, 0.5, 1.0],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(500, 500))
        .build(&event_loop)
        .expect("Failed to create window");
    let mut surface = executor::block_on(Surface::new(window));
    let shader_desc = ShaderDescriptor {
        vert_shader: PathBuf::from("src/shaders/old_shader.vert"),
        frag_shader: PathBuf::from("src/shaders/old_shader.frag"),
    };
    let pipeline_desc = PipelineDescriptor {
        shader_desc,
        vertex_buffer_number: 0,
        alpha_blending: false,
        primitive_topo: PrimitiveTopology::TriangleList,
    };
    surface.create_pipeline(pipeline_desc);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    surface.resize(physical_size);
                }
                WindowEvent::CloseRequested => {
                    println!("The close button was pressed. stopping");
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::MainEventsCleared => surface.request_redraw(),
            Event::RedrawRequested(_) => {
                let render_pass_desc = RenderPassDescriptor {
                    clear_color: wgpu_color(0.5, 0.5, 0.5, 1.0),
                    buffer_usage_desc: None,
                    vertices: Some(0..3),
                };
                surface.redraw(render_pass_desc);
            }
            _ => (),
        }
    })
}
