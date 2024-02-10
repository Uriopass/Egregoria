use std::ops::Mul;

use common::FastMap;
use engine::wgpu::RenderPass;
use engine::{
    Drawable, FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, MeshInstance,
};
use geom::{vec3, vec4, Camera, HeightmapChunk, Intersect3, LinearColor, Matrix4, Vec3, AABB3};
use simulation::map::{Map, MapSubscriber, SubscriberChunkID, UpdateType};

pub struct TreesRender {
    tree_builder: InstancedMeshBuilder<false>,
    trees_cache: FastMap<SubscriberChunkID, InstancedMesh>,
    tree_sub: MapSubscriber,
}

impl TreesRender {
    pub fn new(gfx: &mut GfxContext, map: &Map) -> Self {
        let mesh = gfx.mesh("pine.glb".as_ref()).expect("could not load pine");

        let tree_sub = map.subscribe(UpdateType::Terrain);
        Self {
            tree_builder: InstancedMeshBuilder::new_ref(&mesh),
            trees_cache: FastMap::default(),
            tree_sub,
        }
    }

    fn build(&mut self, map: &Map, ctx: &mut FrameContext<'_>) {
        for chunkid in self.tree_sub.take_updated_chunks() {
            self.tree_builder.instances.clear();

            let aabb = chunkid.bbox();
            map.environment
                .trees
                .query_aabb_visitor(aabb.ll, aabb.ur, |obj| {
                    let Some((_, t)) = map.environment.trees.get(obj.0) else {
                        return;
                    };
                    self.tree_builder.instances.push(MeshInstance {
                        pos: t.pos.z(map.environment.height(t.pos).unwrap_or_default()),
                        dir: t.dir.z0() * t.size * 0.2,
                        tint: ((1.0 - t.size * 0.05) * t.col * LinearColor::WHITE).a(1.0),
                    });
                });

            if let Some(m) = self.tree_builder.build(ctx.gfx) {
                self.trees_cache.insert(chunkid, m);
            } else {
                self.trees_cache.remove(&chunkid);
            }
        }
    }

    pub fn draw(&mut self, map: &Map, cam: &Camera, ctx: &mut FrameContext<'_>) {
        profiling::scope!("draw trees");
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
                        SubscriberChunkID::SIZE_F32 * 1.5,
                        SubscriberChunkID::SIZE_F32 * 1.5,
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
            let chunkcenter = cid.center().z0();
            let max_height = cid
                .convert()
                .filter_map(|c| map.environment.get_chunk(c))
                .map(HeightmapChunk::max_height)
                .fold(0.0, f32::max);

            if !ctx.gfx.frustrum.intersects(&AABB3::new_size(
                cid.corner().z(-40.0),
                vec3(
                    5.0 + SubscriberChunkID::SIZE_F32,
                    5.0 + SubscriberChunkID::SIZE_F32,
                    40.0 + max_height + 100.0,
                ),
            )) || camcenter.distance(chunkcenter.xy()) > 5000.0
            {
                continue;
            }

            ctx.draw(TreeMesh(mesh.clone(), chunkcenter));
        }
    }
}
