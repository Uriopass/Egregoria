use crate::map::{Map, ProjectFilter, ProjectKind, RoadID, UpdateType};
use geom::Vec2;
use geom::OBB;
use geom::{Circle, Vec3};
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;
use std::collections::BTreeSet;

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
    pub height: f32,
}

impl Lot {
    pub fn try_make(
        map: &mut Map,
        parent: RoadID,
        at: Vec3,
        axis: Vec2,
        size: f32,
    ) -> Option<LotID> {
        let height = map.environment.height(at.xy())?;
        if (height - at.z).abs() > 1.0 {
            return None;
        }

        let shape = OBB::new(at.xy() + axis * size * 0.5, axis, size, size);

        let proj = map.project(shape.center().z0(), size * 0.5 - 0.5, ProjectFilter::ALL);
        if !matches!(proj.kind, ProjectKind::Ground) {
            return None;
        }

        let id = map.lots.insert_with_key(move |id| Lot {
            id,
            parent,
            kind: LotKind::Unassigned,
            shape,
            height,
        });
        map.spatial_map.insert(&map.lots[id]);
        Some(id)
    }

    pub fn generate_along_road(map: &mut Map, road: RoadID) {
        if !map.roads.contains_key(road) {
            log::error!("trying to generate along invalid road");
            return;
        }
        fn gen_side(map: &mut Map, road: RoadID, side: f32) {
            let r = unwrap_ret!(map.roads.get(road));

            let w = r.width * 0.5;
            let mut ri = 0.0;
            let r1 = r.points.first().x + r.points.last().x;
            let r2 = r.points.last().y + r.points.first().y;
            let r3 = side * r.length();
            let mut picksize = || {
                ri += 1.0;
                match (common::rand::rand4(r1, r2, r3, ri) * 3.0) as usize {
                    0 => 20.0,
                    1 => 30.0,
                    _ => 40.0,
                }
            };

            let points = r.points.clone();
            let mut along = points.points_dirs_manual();
            let mut size = picksize();
            let mut d = size * 0.5;

            let mut lots = vec![];
            while let Some((pos, dir)) = along.next(d) {
                let axis = side * dir.perp_up();
                let l = Lot::try_make(map, road, pos + axis * (w + 1.0), axis.xy(), size);
                if let Some(id) = l {
                    lots.push(id);
                    map.subscribers.dispatch(UpdateType::Road, &map.lots[id]);

                    d += size * 0.5 + 2.0;
                    size = picksize();
                    d += size * 0.5;
                } else {
                    d += 2.0;
                }
            }
        }

        let r = unwrap_ret!(map.roads.get(road));
        let pair = r.sidewalks(r.src);
        if pair.outgoing.is_some() {
            gen_side(map, road, 1.0);
        }
        if pair.incoming.is_some() {
            gen_side(map, road, -1.0);
        }
    }

    pub fn remove_intersecting_lots(map: &mut Map, road: RoadID) {
        let r = unwrap_retlog!(map.roads.get(road), "{:?} does not exist", road);
        let mut to_remove: BTreeSet<_> = map
            .spatial_map
            .query(r.boldline(), ProjectFilter::LOT)
            .collect();

        let mut rp = |p: Circle| to_remove.extend(map.spatial_map.query(p, ProjectFilter::LOT));
        rp(unwrap_ret!(map.intersections.get(r.src)).bcircle());
        rp(unwrap_ret!(map.intersections.get(r.dst)).bcircle());

        for lot in to_remove {
            if let ProjectKind::Lot(lot) = lot {
                map.lots.remove(lot);
                map.spatial_map.remove(lot);
            }
        }
    }
}
