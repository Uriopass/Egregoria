use crate::map::{BuildingID, IntersectionID, LotID, Map, RoadID};
use derive_more::From;
use flat_spatial::aabbgrid::AABBGridHandle;
use flat_spatial::AABBGrid;
use geom::{Circle, Intersect, Shape, ShapeEnum, Vec2, AABB};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::{BitOr, Neg, Sub};

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    Building(BuildingID),
    Lot(LotID),
    Ground,
}

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

    pub fn as_building(&self) -> Option<BuildingID> {
        match *self {
            Self::Building(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_ground(&self) -> bool {
        matches!(self, ProjectKind::Ground)
    }
}

pub struct SpatialMap {
    broad: AABBGrid<ProjectKind, AABB>,
    near: BTreeMap<ProjectKind, ShapeEnum>,
    ids: BTreeMap<ProjectKind, AABBGridHandle>,
}

impl Default for SpatialMap {
    fn default() -> Self {
        Self {
            broad: AABBGrid::new(50),
            near: Default::default(),
            ids: Default::default(),
        }
    }
}

impl SpatialMap {
    pub fn insert(&mut self, kind: impl Into<ProjectKind>, shape: impl Into<ShapeEnum>) {
        let kind = kind.into();
        let shape = shape.into();
        let handle = self.broad.insert(shape.bbox(), kind);
        self.ids.insert(kind, handle);
        self.near.insert(kind, shape);
    }

    pub fn remove(&mut self, kind: impl Into<ProjectKind>) {
        let kind = kind.into();
        if let Some(id) = self.ids.remove(&kind) {
            self.broad.remove(id);
            self.near.remove(&kind);
        } else {
            warn!(
                "trying to remove {:?} from spatial map but it wasn't present",
                kind
            )
        }
    }

    pub fn update(&mut self, kind: impl Into<ProjectKind>, shape: impl Into<ShapeEnum>) {
        let kind = kind.into();
        let shape = shape.into();
        if let Some(id) = self.ids.get(&kind) {
            self.broad.set_aabb(*id, shape.bbox());
            self.near.insert(kind, shape);
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
        filter: ProjectFilter,
    ) -> impl Iterator<Item = ProjectKind> + '_ {
        self.query(Circle { center, radius }, filter)
    }

    pub fn query<'a>(
        &'a self,
        shape: impl Intersect<ShapeEnum> + Intersect<AABB> + Clone + 'a,
        filter: ProjectFilter,
    ) -> impl Iterator<Item = ProjectKind> + 'a {
        self.broad
            .query(shape.bbox())
            .filter(move |&(_, _, p)| filter.test(p))
            .filter_map(move |(_, _, p)| shape.intersects(self.near.get(p)?).then_some(*p))
    }

    pub fn debug_grid(&self) -> impl Iterator<Item = AABB> + '_ {
        self.broad
            .handles()
            .filter_map(move |x| self.broad.get(x))
            .map(|obj| obj.aabb)
    }

    pub fn contains<T: Into<ProjectKind>>(&self, p: T) -> bool {
        let kind = p.into();

        self.ids.contains_key(&kind);
        let v = unwrap_ret!(self.ids.get(&kind), false);
        self.broad.get(*v).is_some() && self.near.get(&kind).is_some()
    }

    pub fn objects(&self) -> impl Iterator<Item = &ProjectKind> + '_ {
        self.ids.keys()
    }
}

#[derive(Copy, Clone)]
pub struct ProjectFilter(u8);

impl ProjectFilter {
    pub const INTER: Self = Self(1);
    pub const ROAD: Self = Self(2);
    pub const BUILDING: Self = Self(4);
    pub const LOT: Self = Self(8);
    pub const ALL: Self = Self(!0);

    pub fn test(self, p: &ProjectKind) -> bool {
        match p {
            ProjectKind::Inter(_) => (self.0 & Self::INTER.0) != 0,
            ProjectKind::Road(_) => (self.0 & Self::ROAD.0) != 0,
            ProjectKind::Building(_) => (self.0 & Self::BUILDING.0) != 0,
            ProjectKind::Lot(_) => (self.0 & Self::LOT.0) != 0,
            ProjectKind::Ground => true,
        }
    }
}

impl BitOr for ProjectFilter {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl Sub for ProjectFilter {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl Neg for ProjectFilter {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(!self.0)
    }
}
