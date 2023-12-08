use crate::DemoElement;
use engine::meshload::load_mesh;
use engine::{
    Context, FrameContext, InstancedMesh, InstancedMeshBuilder, Material, MeshInstance,
    MetallicRoughness,
};
use geom::{vec3, Camera, InfiniteFrustrum, LinearColor, Vec3};

pub struct Spheres {
    meshes: Vec<InstancedMesh>,
}

impl DemoElement for Spheres {
    fn name(&self) -> &'static str {
        "Spheres"
    }

    fn init(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        let mesh = load_mesh(gfx, "sphere.glb").unwrap();
        let alb = gfx.material(mesh.materials[0].0).albedo.clone();

        let mut meshes = vec![];

        const N_MET: i32 = 5;
        const N_ROUGH: i32 = 10;

        for x in 0..N_ROUGH {
            for z in 0..N_MET {
                let mut c = mesh.clone();

                c.materials[0].0 = gfx.register_material(Material::new_raw(
                    &gfx.device,
                    alb.clone(),
                    MetallicRoughness {
                        metallic: z as f32 / (N_MET as f32 - 1.0),
                        roughness: x as f32 / (N_ROUGH as f32 - 1.0),
                        tex: None,
                    },
                    None,
                    &gfx.palette(),
                ));
                let mut i = InstancedMeshBuilder::<true>::new(c);
                i.instances.push(MeshInstance {
                    pos: 2.3 * vec3(x as f32, 0.0, z as f32),
                    dir: Vec3::X,
                    tint: LinearColor::WHITE,
                });
                meshes.push(i.build(gfx).unwrap());
            }
        }

        Self { meshes }
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, fc: &mut FrameContext, _cam: &Camera, _frustrum: &InfiniteFrustrum) {
        fc.draw(self.meshes.clone());
    }
}
