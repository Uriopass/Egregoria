use crate::{Buildings, Intersections, Lots, Map, ProjectKind, RoadID, Roads, SpatialMap};
use geom::Polygon;
use geom::Vec2;
use geom::OBB;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LotID;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LotKind {
    Residential,
    Commercial,
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
        lots: &mut Lots,
        spatial: &mut SpatialMap,
        roads: &Roads,
        inters: &Intersections,
        buildings: &Buildings,
        parent: RoadID,
        at: Vec2,
        axis: Vec2,
        size: f32,
    ) -> Option<LotID> {
        let shape = OBB::new(at + axis * size * 0.5, axis, size, size);

        for obj in spatial.query_rect(shape.bbox()) {
            match obj {
                ProjectKind::Road(r) => {
                    let r = &roads[r];
                    if r.intersects(shape) {
                        return None;
                    }
                }
                ProjectKind::Inter(i) => {
                    let i = &inters[i];
                    if i.polygon.intersects(&Polygon(shape.corners.to_vec())) {
                        return None;
                    }
                }
                ProjectKind::Lot(h) => {
                    let h = &lots[h];
                    if h.shape.intersects(shape) {
                        return None;
                    }
                }
                ProjectKind::Building(id) => {
                    let b = &buildings[id];
                    if b.draw
                        .iter()
                        .any(|(p, _)| p.intersects(&Polygon(shape.corners.to_vec())))
                    {
                        return None;
                    }
                }
                ProjectKind::Ground => {}
            }
        }

        let r = common::rand::rand2(at.x, at.y);

        let kind = if r < 0.2 {
            LotKind::Commercial
        } else {
            LotKind::Residential
        };

        let id = lots.insert_with_key(move |id| Lot {
            id,
            parent,
            kind,
            shape,
            size,
        });
        spatial.insert(id, shape.bbox());
        Some(id)
    }

    pub fn generate_along_road(map: &mut Map, road: RoadID) {
        fn gen_side(map: &mut Map, road: RoadID, side: f32) {
            let r = &map.roads[road];

            let w = r.width * 0.5;

            fn picksize() -> f32 {
                *[20.0f32, 30.0, 40.0]
                    .choose(&mut rand::thread_rng())
                    .unwrap()
            }

            let mut along = r.generated_points.points_dirs_manual();
            let mut size = picksize();
            let mut d = size * 0.5;

            let mut lots = vec![];
            while let Some((pos, dir)) = along.next(d) {
                let axis = side * dir.perpendicular();
                let l = Lot::try_make(
                    &mut map.lots,
                    &mut map.spatial_map,
                    &map.roads,
                    &map.intersections,
                    &map.buildings,
                    road,
                    pos + axis * (w + 1.0),
                    axis,
                    size,
                );
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
            .query_rect(r.generated_points.bbox())
            .filter_map(|kind| {
                let id = kind.to_lot()?;
                if r.intersects(map.lots[id].shape) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut rp = |p: &Polygon| {
            to_remove.extend(map.spatial_map.query_rect(p.bbox()).filter_map(|kind| {
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
