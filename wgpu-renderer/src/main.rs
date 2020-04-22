mod depth;
mod shader;
mod surface;
mod vertex;

use std::path::PathBuf;

use futures::executor;
use wgpu::{Color, CompareFunction, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use shader::ShaderDescriptor;
use surface::{BufferUsageDescriptor, PipelineDescriptor, RenderPassDescriptor, Surface};
use vertex::Vertex;

fn wgpu_color(r: f64, g: f64, b: f64, a: f64) -> Color {
    Color { r, g, b, a }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.1],
        color: [1.0, 1.0, 1.0, 0.5],
    },
    Vertex {
        position: [0.5, -0.5, 0.1],
        color: [1.0, 1.0, 1.0, 0.5],
    },
    Vertex {
        position: [0.5, 0.5, 0.1],
        color: [1.0, 1.0, 1.0, 0.5],
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

const INDICES: &[u16] = &[0, 1, 2, 3, 4, 5];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(500, 500))
        .build(&event_loop)
        .expect("Failed to create window");
    let mut surface = executor::block_on(Surface::new(window));
    let shader_desc = ShaderDescriptor {
        vert_shader: PathBuf::from("wgpu-renderer/src/shaders/shader.vert"),
        frag_shader: PathBuf::from("wgpu-renderer/src/shaders/shader.frag"),
    };
    let pipeline_desc = PipelineDescriptor {
        shader_desc,
        vertex_buffer_number: 1,
        compare_depth_function: CompareFunction::Less,
        alpha_blending: true,
        primitive_topo: PrimitiveTopology::TriangleList,
    };
    let vertex_buffer =
        surface.create_buffer(bytemuck::cast_slice(VERTICES), wgpu::BufferUsage::VERTEX);
    let index_buffer =
        surface.create_buffer(bytemuck::cast_slice(INDICES), wgpu::BufferUsage::INDEX);
    surface.create_pipeline(pipeline_desc);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
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
                let buffer_usage_desc = Some(BufferUsageDescriptor {
                    vertex_buffer: &vertex_buffer,
                    index_buffer: &index_buffer,
                    indices: 0..INDICES.len() as u32,
                    base_vertex: 0,
                });
                let render_pass_desc = RenderPassDescriptor {
                    clear_color: wgpu_color(0.5, 0.5, 0.5, 1.0),
                    buffer_usage_desc,
                    vertices: None,
                };
                surface.redraw(render_pass_desc);
            }
            _ => (),
        }
    })
}
