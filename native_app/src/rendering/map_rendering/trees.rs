use common::FastMap;
use egregoria::map::{ChunkID, Map, MapSubscriber, UpdateType, CHUNK_SIZE};
use engine::meshload::load_mesh;
use engine::wgpu::RenderPass;
use engine::{
    Drawable, FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, MeshInstance,
};
use geom::{vec3, vec4, Camera, InfiniteFrustrum, Intersect3, LinearColor, Matrix4, Vec3, AABB3};
use std::ops::Mul;

pub struct TreesRender {
    tree_builder: InstancedMeshBuilder<false>,
    trees_cache: FastMap<ChunkID, InstancedMesh>,
    tree_sub: MapSubscriber,
}

impl TreesRender {
    pub fn new(gfx: &mut GfxContext, map: &Map) -> Self {
        let mesh = load_mesh(gfx, "pine.glb").expect("could not load pine");

        let tree_sub = map.subscribe(UpdateType::Terrain);
        Self {
            tree_builder: InstancedMeshBuilder::new(mesh),
            trees_cache: FastMap::default(),
            tree_sub,
        }
    }

    fn build(&mut self, map: &Map, ctx: &mut FrameContext<'_>) {
        for chunkid in self.tree_sub.take_updated_chunks() {
            let chunk = if let Some(x) = map.terrain.chunks.get(&chunkid) {
                x
            } else {
                continue;
            };

            self.tree_builder.instances.clear();

            for t in &chunk.trees {
                self.tree_builder.instances.push(MeshInstance {
                    pos: t.pos.z(map.terrain.height(t.pos).unwrap_or_default()),
                    dir: t.dir.z0() * t.size * 0.2,
                    tint: ((1.0 - t.size * 0.05) * t.col * LinearColor::WHITE).a(1.0),
                });
            }

            if let Some(m) = self.tree_builder.build(ctx.gfx) {
                self.trees_cache.insert(chunkid, m);
            } else {
                self.trees_cache.remove(&chunkid);
            }
        }
    }

    pub fn draw(
        &mut self,
        map: &Map,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        ctx: &mut FrameContext<'_>,
    ) {
        self.build(map, ctx);

        let camcenter = cam.pos.xy();

        struct TreeMesh(InstancedMesh, Vec3);

        impl Drawable for TreeMesh {
            fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
                self.0.draw(gfx, rp);
            }
            fn draw_depth<'a>(
                &'a self,
                gfx: &'a GfxContext,
                rp: &mut RenderPass<'a>,
                shadow_cascade: Option<&Matrix4>,
            ) {
                if let Some(v) = shadow_cascade {
                    let pos = v.mul(self.1.w(1.0));

                    let margin = v.mul(vec4(
                        CHUNK_SIZE as f32 * 1.5,
                        CHUNK_SIZE as f32 * 1.5,
                        100.0,
                        0.0,
                    )) * pos.w;

                    if pos.x.abs() > pos.w + margin.x.abs()
                        || pos.y.abs() > pos.w + margin.y.abs()
                        || pos.z < -margin.z.abs()
                        || pos.z > pos.w + margin.z.abs()
                    {
                        return;
                    }
                }
                self.0.draw_depth(gfx, rp, shadow_cascade);
            }
        }

        for (cid, mesh) in self.trees_cache.iter() {
            let chunkcenter = vec3(
                (cid.0 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                (cid.1 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                0.0,
            );

            if !frustrum.intersects(&AABB3::centered(
                chunkcenter,
                vec3(5.0 + CHUNK_SIZE as f32, 5.0 + CHUNK_SIZE as f32, 100.0),
            )) || camcenter.distance(chunkcenter.xy()) > 5000.0
            {
                continue;
            }

            ctx.draw(TreeMesh(mesh.clone(), chunkcenter));
        }
    }
}
