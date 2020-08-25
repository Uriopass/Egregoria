use crate::{HouseID, IntersectionID, LotID, RoadID};
use flat_spatial::shape::AABB;
use flat_spatial::shapegrid::ShapeGridHandle;
use flat_spatial::ShapeGrid;
use geom::rect::Rect;
use geom::Vec2;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    House(HouseID),
    Lot(LotID),
    Ground,
}

macro_rules! impl_from_pk {
    ($t: ty, $e: expr) => {
        impl From<$t> for ProjectKind {
            fn from(x: $t) -> Self {
                $e(x)
            }
        }
    };
}

impl_from_pk!(IntersectionID, ProjectKind::Inter);
impl_from_pk!(RoadID, ProjectKind::Road);
impl_from_pk!(HouseID, ProjectKind::House);
impl_from_pk!(LotID, ProjectKind::Lot);

impl ProjectKind {
    pub fn to_lot(self) -> Option<LotID> {
        if let ProjectKind::Lot(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

pub struct SpatialMap {
    grid: ShapeGrid<ProjectKind, AABB>,
    ids: HashMap<ProjectKind, ShapeGridHandle>,
}

impl Default for SpatialMap {
    fn default() -> Self {
        Self {
            grid: ShapeGrid::new(50),
            ids: Default::default(),
        }
    }
}

impl SpatialMap {
    pub fn insert<T: Into<ProjectKind>>(&mut self, p: T, bbox: Rect) {
        let kind = p.into();
        let handle = self.grid.insert(rect_to_aabb(bbox), kind);
        self.ids.insert(kind, handle);
    }

    pub fn remove<T: Into<ProjectKind>>(&mut self, p: T) {
        let kind = p.into();
        if let Some(id) = self.ids.remove(&kind) {
            self.grid.remove(id);
        } else {
            warn!(
                "trying to remove {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn update<T: Into<ProjectKind>>(&mut self, p: T, bbox: Rect) {
        let kind = p.into();
        if let Some(id) = self.ids.get(&kind) {
            self.grid.set_shape(*id, rect_to_aabb(bbox));
        } else {
            warn!(
                "trying to update shape {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn query_rect(&self, r: Rect) -> impl Iterator<Item = ProjectKind> + '_ {
        self.grid.query(rect_to_aabb(r)).map(|(_, _, k)| *k)
    }

    pub fn query_point(&self, p: Vec2) -> impl Iterator<Item = ProjectKind> + '_ {
        self.grid.query([p.x, p.y]).map(|(_, _, k)| *k)
    }

    pub fn debug_grid(&self) -> impl Iterator<Item = Rect> + '_ {
        self.grid
            .handles()
            .filter_map(move |x| self.grid.get(x))
            .map(|(aabb, _)| aabb_to_rect(*aabb))
    }
}

fn rect_to_aabb(r: Rect) -> AABB {
    AABB::new([r.x, r.y].into(), [r.x + r.w, r.y + r.h].into())
}

fn aabb_to_rect(r: AABB) -> Rect {
    Rect::new(r.ll.x, r.ll.y, r.ur.x - r.ll.x, r.ur.y - r.ll.y)
}
