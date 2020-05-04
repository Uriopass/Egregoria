use crate::geometry::Vec2;
use crate::graphs::SecondaryGraph;
use crate::map_model::{Intersection, IntersectionID};
use cgmath::MetricSpace;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Pathfinder {
    g: SecondaryGraph<IntersectionID, Vec2, NotNan<f32>>,
}

impl Pathfinder {
    pub fn add(&mut self, inter: &Intersection) {
        self.g.push(inter.id, inter.pos);
    }

    pub fn remove(&mut self, inter: IntersectionID) {
        self.g.remove_node(inter);
    }

    pub fn connect(&mut self, from: &Intersection, to: &Intersection) {
        let dist = from.pos.distance(to.pos);

        self.g.add_neigh(from.id, to.id, NotNan::new(dist).unwrap());
    }

    pub fn disconnect(&mut self, from: IntersectionID, to: IntersectionID) {
        self.g.remove_neigh(from, to);
    }

    pub fn update(&mut self, a: &Intersection) {
        self.g[a.id] = a.pos;

        for (id, w) in &mut self.g.edges[a.id] {
            let his_pos = self.g.nodes[*id];
            *w = NotNan::new(his_pos.distance2(a.pos)).unwrap()
        }

        for (id, w) in &mut self.g.backward_edges[a.id] {
            let his_pos = self.g.nodes[*id];
            *w = NotNan::new(his_pos.distance2(a.pos)).unwrap()
        }
    }

    pub fn inner_ref(&self) -> &SecondaryGraph<IntersectionID, Vec2, NotNan<f32>> {
        &self.g
    }

    pub fn path(
        &self,
        start: &Intersection,
        end: &Intersection,
    ) -> Option<(Vec<IntersectionID>, f32)> {
        let end_pos = end.pos;
        let end_id = end.id;

        pathfinding::directed::astar::astar(
            &start.id,
            |p| self.g.iter_neighs_owned(*p),
            |p| NotNan::new(self.g[*p].distance(end_pos)).unwrap(),
            |p| *p == end_id,
        )
        .map(|(v, d)| (v, d.into_inner()))
    }
}
