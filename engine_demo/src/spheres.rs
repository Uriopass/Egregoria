use engine::{
    Context, FrameContext, InstancedMesh, InstancedMeshBuilder, Material, MeshInstance,
    MetallicRoughness,
};
use geom::{vec3, Camera, LinearColor, Vec3};

use crate::DemoElement;

pub struct Spheres {
    meshes: Vec<InstancedMesh>,
}

impl DemoElement for Spheres {
    fn name(&self) -> &'static str {
        "Spheres"
    }

    fn init(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        let mesh = gfx.mesh("sphere.glb".as_ref()).unwrap();
        let mut meshes = vec![];

        const N_MET: i32 = 5;
        const N_ROUGH: i32 = 10;

        for x in 0..N_ROUGH {
            for z in 0..N_MET {
                let mut c = (*mesh).clone();

                let mat = Material::new_raw(
                    &gfx.device,
                    &gfx.null_texture,
                    MetallicRoughness {
                        metallic: z as f32 / (N_MET as f32 - 1.0),
                        roughness: x as f32 / (N_ROUGH as f32 - 1.0),
                        tex: None,
                    },
                    None,
                    &gfx.palette(),
                );

                c.lods[0].primitives[0].0 = gfx.register_material(mat);
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

    fn update(&mut self, _ctx: &mut Context, _cam: &Camera) {}

    fn render(&mut self, fc: &mut FrameContext, _cam: &Camera) {
        fc.draw(self.meshes.clone());
    }
}
