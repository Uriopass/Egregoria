use std::collections::BTreeSet;

use geom::{pack_height, Shape, Vec2, AABB, NO_OVERRIDE};

use crate::map::terrain::CELL_SIZE;
use crate::map::{
    Map, ProjectFilter, ProjectKind, SubscriberChunkID, TerrainChunkID, UpdateType, ROAD_Z_OFFSET,
    TERRAIN_CHUNK_RESOLUTION,
};

struct OverrideSetter {
    chunk: TerrainChunkID,
    overrides: [[u16; TERRAIN_CHUNK_RESOLUTION]; TERRAIN_CHUNK_RESOLUTION],
    chunk_bound: AABB,
}

impl OverrideSetter {
    fn new(chunk: TerrainChunkID) -> Self {
        let chunk_bound = chunk.bbox();

        let overrides = [[NO_OVERRIDE; TERRAIN_CHUNK_RESOLUTION]; TERRAIN_CHUNK_RESOLUTION];

        Self {
            chunk,
            overrides,
            chunk_bound,
        }
    }

    fn set_override(&mut self, obj_bounds: AABB, filter: impl Fn(Vec2) -> Option<f32>) {
        let b = obj_bounds.intersection(self.chunk_bound);
        if b.size() == Vec2::ZERO {
            return;
        }

        let b = b.offset(-self.chunk_bound.ll);

        let start = (b.ll / CELL_SIZE).floor();
        let end = (b.ur / CELL_SIZE).ceil();

        for y in start.y as usize..end.y as usize {
            for x in start.x as usize..end.x as usize {
                let pos = Vec2::new(x as f32, y as f32) * CELL_SIZE + self.chunk_bound.ll;
                let Some(h) = filter(pos) else {
                    continue;
                };
                let mut h = pack_height(h);
                if h == NO_OVERRIDE {
                    h += 1;
                }
                let v = self.overrides[y][x];
                self.overrides[y][x] = if v == NO_OVERRIDE { h } else { v.min(h) }
            }
        }
    }

    fn finish(self, map: &mut Map) {
        map.environment.set_overrides(self.chunk, self.overrides);
        map.subscribers
            .dispatch_chunk(UpdateType::Terrain, self.chunk);
    }
}

/// Updates the overrides on the map.
/// Proceeds in 3 steps:
///  - Find all objects that could have changed (from chunk)
///  - Find all terrain chunks that can be affected
///  - Update each chunk individually
pub fn find_overrides(map: &mut Map, chunk: SubscriberChunkID) {
    let mut terrain_affected = BTreeSet::new();

    let sub_chunk_bbox = chunk.bbox();

    for obj in map.spatial_map.query(
        sub_chunk_bbox,
        ProjectFilter::ROAD | ProjectFilter::INTER | ProjectFilter::BUILDING,
    ) {
        // ensure the object is only processed once
        if !sub_chunk_bbox.contains(obj.canonical_position(map)) {
            continue;
        }

        let bbox = match obj {
            ProjectKind::Intersection(i) => {
                let i = map.get(i).unwrap();

                let mut bounds = i.bcircle();
                bounds.radius *= 2.0;

                bounds.bbox()
            }
            ProjectKind::Road(r) => {
                let r = map.get(r).unwrap();

                let expand = 10.0 + r.width * 3.0;

                r.points.bbox().flatten().expand(expand + 3.0)
            }
            ProjectKind::Building(b) => {
                let b = map.get(b).unwrap();

                let obb = b.obb.expand(25.0);

                obb.bbox()
            }
            _ => continue,
        };

        for chunk in map.environment.covered_chunks(bbox) {
            terrain_affected.insert(chunk);
        }
    }

    for chunk in terrain_affected {
        let mut setter = OverrideSetter::new(chunk);

        for obj in map.spatial_map.query(
            chunk.bbox(),
            ProjectFilter::ROAD | ProjectFilter::INTER | ProjectFilter::BUILDING,
        ) {
            match obj {
                ProjectKind::Intersection(i) => {
                    let i = map.get(i).unwrap();

                    let mut bounds = i.bcircle();
                    bounds.radius *= 2.0;

                    setter
                        .set_override(bounds.bbox(), |pos| bounds.contains(pos).then_some(i.pos.z));
                }
                ProjectKind::Road(r) => {
                    let r = map.get(r).unwrap();

                    let expand = 10.0 + r.width * 3.0;

                    setter.set_override(r.points.bbox().flatten().expand(expand + 3.0), |pos| {
                        let proj = r.points.project_2d(pos);
                        proj.xy()
                            .is_close(pos, expand)
                            .then_some(proj.z - ROAD_Z_OFFSET)
                    })
                }
                ProjectKind::Building(b) => {
                    let b = map.get(b).unwrap();

                    let obb = b.obb.expand(25.0);

                    setter.set_override(obb.bbox(), |pos| obb.contains(pos).then_some(b.height));
                }
                _ => {}
            }
        }

        setter.finish(map);
    }
}
