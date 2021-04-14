use crate::{BuildingID, IntersectionID, LotID, Map, RoadID};
use common::FastMap;
use flat_spatial::shapegrid::ShapeGridHandle;
use flat_spatial::ShapeGrid;
use geom::{Circle, Intersect, Vec2, AABB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    Building(BuildingID),
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
impl_from_pk!(BuildingID, ProjectKind::Building);
impl_from_pk!(LotID, ProjectKind::Lot);

impl ProjectKind {
    pub fn to_lot(self) -> Option<LotID> {
        if let ProjectKind::Lot(id) = self {
            Some(id)
        } else {
            None
        }
    }

    pub fn check_valid(&self, map: &Map) -> bool {
        match *self {
            ProjectKind::Inter(id) => map.intersections.contains_key(id),
            ProjectKind::Road(id) => map.roads.contains_key(id),
            ProjectKind::Building(id) => map.buildings.contains_key(id),
            ProjectKind::Lot(id) => map.lots.contains_key(id),
            ProjectKind::Ground => true,
        }
    }
}

#[derive(Clone)]
pub struct SpatialMap {
    broad: ShapeGrid<ProjectKind, AABB>,
    ids: FastMap<ProjectKind, ShapeGridHandle>,
}

impl Default for SpatialMap {
    fn default() -> Self {
        Self {
            broad: ShapeGrid::new(50),
            ids: Default::default(),
        }
    }
}

impl SpatialMap {
    pub fn insert<T: Into<ProjectKind>>(&mut self, p: T, bbox: AABB) {
        let kind = p.into();
        let handle = self.broad.insert(bbox, kind);
        self.ids.insert(kind, handle);
    }

    pub fn remove<T: Into<ProjectKind>>(&mut self, p: T) {
        let kind = p.into();
        if let Some(id) = self.ids.remove(&kind) {
            self.broad.remove(id);
        } else {
            warn!(
                "trying to remove {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn update<T: Into<ProjectKind>>(&mut self, p: T, bbox: AABB) {
        let kind = p.into();
        if let Some(id) = self.ids.get(&kind) {
            self.broad.set_shape(*id, bbox);
        } else {
            warn!(
                "trying to update shape {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn query_around(
        &self,
        center: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = ProjectKind> + '_ {
        self.query(Circle { center, radius })
    }

    pub fn query<'a>(
        &'a self,
        r: impl Intersect<AABB> + Clone + 'a,
    ) -> impl Iterator<Item = ProjectKind> + 'a {
        self.broad.query(r).map(|(_, _, k)| *k)
    }

    pub fn debug_grid(&self) -> impl Iterator<Item = AABB> + '_ {
        self.broad
            .handles()
            .filter_map(move |x| self.broad.get(x))
            .map(|(aabb, _)| *aabb)
    }

    pub fn contains<T: Into<ProjectKind>>(&self, p: T) -> bool {
        let kind = p.into();

        let v = unwrap_ret!(self.ids.get(&kind), false);
        self.broad.get(*v).is_some()
    }

    pub fn objects(&self) -> impl Iterator<Item = &ProjectKind> + '_ {
        self.ids.keys()
    }
}
