use crate::procgen::heightmap::height;
use crate::{Map, ProjectKind, RoadID};
use geom::OBB;
use geom::{Intersect, Polygon};
use geom::{Shape, Vec2};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LotID;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LotKind {
    Unassigned,
    Residential,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lot {
    pub id: LotID,
    pub parent: RoadID,
    pub kind: LotKind,
    pub shape: OBB,
    pub size: f32,
}

impl Lot {
    pub fn try_make(
        map: &mut Map,
        parent: RoadID,
        at: Vec2,
        axis: Vec2,
        size: f32,
    ) -> Option<LotID> {
        let shape = OBB::new(at + axis * size * 0.5, axis, size, size);

        if height(at) < 0.12 {
            return None;
        }

        let proj = map.project(shape.center(), size * 0.5 - 0.5);
        if !matches!(proj.kind, ProjectKind::Ground) {
            return None;
        }

        let bbox = shape.bbox();
        let id = map.lots.insert_with_key(move |id| Lot {
            id,
            parent,
            kind: LotKind::Unassigned,
            shape,
            size,
        });
        map.spatial_map.insert(id, bbox);
        Some(id)
    }

    pub fn generate_along_road(map: &mut Map, road: RoadID) {
        fn gen_side(map: &mut Map, road: RoadID, side: f32) {
            let r = &map.roads[road];

            let w = r.width * 0.5;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(
                common::rand::rand3(
                    r.src_point.x + r.dst_point.x,
                    r.dst_point.y + r.src_point.y,
                    side * r.length,
                )
                .to_bits() as u64,
            );

            let mut picksize = || *[20.0f32, 30.0, 40.0].choose(&mut rng).unwrap();

            let points = r.generated_points.clone();
            let mut along = points.points_dirs_manual();
            let mut size = picksize();
            let mut d = size * 0.5;

            let mut lots = vec![];
            while let Some((pos, dir)) = along.next(d) {
                let axis = side * dir.perpendicular();
                let l = Lot::try_make(map, road, pos + axis * (w + 1.0), axis, size);
                if let Some(id) = l {
                    lots.push(id);

                    d += size * 0.5 + 2.0;
                    size = picksize();
                    d += size * 0.5;
                } else {
                    d += 2.0;
                }
            }

            map.roads[road].lots.extend_from_slice(&lots);
        }

        let pair = map.roads[road].sidewalks(map.roads[road].src);
        if pair.outgoing.is_some() {
            gen_side(map, road, 1.0);
        }
        if pair.incoming.is_some() {
            gen_side(map, road, -1.0);
        }
    }

    pub fn remove_intersecting_lots(map: &mut Map, road: RoadID) {
        let r = &map.roads[road];
        let mut to_remove = map
            .spatial_map
            .query(r.generated_points.bbox())
            .filter_map(|kind| {
                let id = kind.to_lot()?;
                if r.intersects(&map.lots[id].shape) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut rp = |p: &Polygon| {
            to_remove.extend(map.spatial_map.query(p.bbox()).filter_map(|kind| {
                let id = kind.to_lot()?;
                if p.intersects(&Polygon(map.lots[id].shape.corners.to_vec())) {
                    Some(id)
                } else {
                    None
                }
            }));
        };
        rp(&map.intersections[r.src].polygon);
        rp(&map.intersections[r.dst].polygon);

        for lot in to_remove {
            if let Some(l) = map.lots.remove(lot) {
                let r = &mut map.roads[l.parent].lots;
                if let Some(v) = r.iter().position(|&x| x == l.id) {
                    r.swap_remove(v);
                }
                map.spatial_map.remove(lot);
            }
        }
    }
}
