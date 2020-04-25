use crate::engine::{
    ColoredUvVertex, Context, Drawable, FrameContext, IndexType, InstanceRaw, SpriteBatch,
    SpriteBatchBuilder, Texture, TexturedMesh, TexturedMeshBuilder, Vertex,
};
use crate::rendering::CameraHandler;
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Zero};
use scale::physics::Transform;
use winit::dpi::PhysicalSize;

pub struct State {
    camera: CameraHandler,
    mesh: TexturedMesh,
    sb: SpriteBatch,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler::new(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32, 10.0);

        let tex = Texture::from_path(&ctx.gfx, "resources/car.png").expect("couldn't load car");

        let mut mb = TexturedMeshBuilder::new();
        mb.extend(&UV_VERTICES, &UV_INDICES);
        let mesh = mb.build(&ctx.gfx, tex.clone());

        let mut sb = SpriteBatchBuilder::new(tex);

        let mut pos = Transform::new(Vector2::<f32>::new(10.0, 0.0));
        pos.set_angle(0.5);

        sb.instances.push(InstanceRaw::new(
            pos.to_matrix4(),
            Vector3::new(1.0, 1.0, 1.0),
            4.5,
        ));

        let sbb = sb.build(&ctx.gfx);

        Self {
            camera,
            mesh,
            sb: sbb,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera
            .easy_camera_movement(ctx, 1.0 / 30.0, true, true);
        self.camera.update(ctx);
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        //self.mesh.draw(ctx);
        self.sb.draw(ctx);
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

const UV_VERTICES: &[ColoredUvVertex] = &[
    ColoredUvVertex {
        position: [-50.0, -50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 1.0],
    },
    ColoredUvVertex {
        position: [150.0, -50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 1.0],
    },
    ColoredUvVertex {
        position: [150.0, 50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [1.0, 0.0],
    },
    ColoredUvVertex {
        position: [-50.0, 50.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[2, 1, 0, 3, 2, 0];
