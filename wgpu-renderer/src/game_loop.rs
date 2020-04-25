use crate::engine::{
    Context, Drawable, FrameContext, IndexType, Texture, TexturedMesh, TexturedMeshBuilder,
    UvVertex, Vertex,
};
use crate::rendering::CameraHandler;
use cgmath::Vector2;
use winit::dpi::PhysicalSize;

pub struct State {
    camera: CameraHandler,
    mesh: TexturedMesh,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32);
        let tex = Texture::from_path(&ctx.gfx, "resources/car.png").expect("couldn't load car");

        let mut mb = TexturedMeshBuilder::new();
        mb.extend(&UV_VERTICES, &UV_INDICES);
        let mesh = mb.build(&ctx.gfx, tex);

        Self { camera, mesh }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera
            .easy_camera_movement(ctx, 1.0 / 30.0, true, true);
        self.camera.update(ctx);
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        self.mesh.draw(ctx);
    }

    pub fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }

    pub fn unproject(&mut self, pos: Vector2<f32>) -> Vector2<f32> {
        self.camera.unproject_mouse_click(pos)
    }
}

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
