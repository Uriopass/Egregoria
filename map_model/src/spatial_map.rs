use crate::{House, HouseID, Intersection, IntersectionID, ProjectKind, Road, RoadID};
use flat_spatial::shape::AABB;
use flat_spatial::shapegrid::ShapeGridHandle;
use flat_spatial::ShapeGrid;
use geom::rect::Rect;
use geom::Vec2;
use slotmap::SecondaryMap;

pub struct SpatialMap {
    grid: ShapeGrid<ProjectKind, AABB>,

    house_ids: SecondaryMap<HouseID, ShapeGridHandle>,
    road_ids: SecondaryMap<RoadID, ShapeGridHandle>,
    intersection_ids: SecondaryMap<IntersectionID, ShapeGridHandle>,
}

impl Default for SpatialMap {
    fn default() -> Self {
        Self {
            grid: ShapeGrid::new(50),
            house_ids: Default::default(),
            road_ids: Default::default(),
            intersection_ids: Default::default(),
        }
    }
}

impl SpatialMap {
    pub fn insert_house(&mut self, h: &House) {
        let bbox = h.exterior.bbox();
        let handle = self
            .grid
            .insert(rect_to_aabb(bbox), ProjectKind::House(h.id));
        self.house_ids.insert(h.id, handle);
    }

    pub fn remove_house(&mut self, h: &House) {
        if let Some(id) = self.house_ids.remove(h.id) {
            self.grid.remove(id);
        } else {
            println!(
                "Trying to remove {:?} from spatial map but it wasn't present",
                h.id
            )
        }
    }

    pub fn insert_road(&mut self, r: &Road) {
        let mut bbox = r.generated_points.bbox();
        bbox.x -= r.width * 0.5;
        bbox.y -= r.width * 0.5;
        bbox.w += r.width;
        bbox.h += r.width;

        let handle = self
            .grid
            .insert(rect_to_aabb(bbox), ProjectKind::Road(r.id));
        self.road_ids.insert(r.id, handle);
    }

    pub fn remove_road(&mut self, r: &Road) {
        if let Some(id) = self.road_ids.remove(r.id) {
            self.grid.remove(id);
        } else {
            println!(
                "Trying to remove {:?} from spatial map but it wasn't present",
                r.id
            )
        }
    }

    pub fn update_inter(&mut self, inter: &Intersection) {
        if inter.roads.len() == 0 {
            self.intersection_ids
                .remove(inter.id)
                .and_then(|h| self.grid.remove(h));
            return;
        }
        let bbox = rect_to_aabb(inter.polygon.bbox());
        match self.intersection_ids.get(inter.id) {
            Some(x) => self.grid.set_shape(*x, bbox),
            None => {
                let h = self.grid.insert(bbox, ProjectKind::Inter(inter.id));
                self.intersection_ids.insert(inter.id, h);
            }
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
