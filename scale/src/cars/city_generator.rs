use crate::cars::car_graph::{RoadGraph, RoadNode};
use crate::graphs::graph::{Edge, Graph, NodeID};
use cgmath::Vector2;
use cgmath::{InnerSpace, MetricSpace};
use std::collections::HashMap;

pub struct Intersection {
    pos: Vector2<f32>,
}

impl Intersection {
    pub fn new(pos: Vector2<f32>) -> Self {
        Intersection { pos }
    }
}

pub struct CityGenerator {
    intersections: Graph<Intersection>,
}

impl CityGenerator {
    pub fn new() -> Self {
        CityGenerator {
            intersections: Graph::new(),
        }
    }

    pub fn add_intersection(&mut self, i: Intersection) -> NodeID {
        self.intersections.add_node(i)
    }

    pub fn connect(&mut self, a: NodeID, b: NodeID) {
        self.intersections.add_neigh(a, b, 1.0);
        self.intersections.add_neigh(b, a, 1.0);
    }

    pub fn build(self) -> RoadGraph {
        let mut g: Graph<RoadNode> = Graph::new();

        let mut out_nodes = HashMap::<NodeID, HashMap<NodeID, NodeID>>::new();
        let mut in_nodes = HashMap::<NodeID, HashMap<NodeID, NodeID>>::new();

        for (id, inter) in &self.intersections.nodes {
            let center = inter.pos;

            for Edge { to, .. } in self.intersections.get_neighs(*id) {
                let inter2 = &self.intersections.nodes[to];
                let dir = (inter2.pos - center).normalize();
                let nor = Vector2::new(-dir.y, dir.x);

                let out_id = g.add_node(RoadNode::new(center + dir * 25.0 - nor * 4.0));
                let in_id = g.add_node(RoadNode::new(inter2.pos - dir * 25.0 - nor * 4.0));

                out_nodes
                    .entry(*id)
                    .or_insert_with(HashMap::new)
                    .insert(*to, out_id);
                in_nodes
                    .entry(*to)
                    .or_insert_with(HashMap::new)
                    .insert(*id, in_id);
            }
        }

        for id in self.intersections.nodes.keys() {
            let outs = &out_nodes[id];
            let ins = &in_nodes[id];

            for Edge { to, .. } in self.intersections.get_neighs(*id) {
                let out_node = outs[to];
                let in_node = in_nodes[to][id];

                g.add_neigh(
                    out_node,
                    in_node,
                    g.nodes[&in_node].pos.distance(g.nodes[&out_node].pos),
                );
            }

            for (in_from, in_node) in ins {
                for (out_from, out_node) in outs {
                    if in_from == out_from && outs.len() > 2 {
                        continue;
                    }
                    g.add_neigh(
                        *in_node,
                        *out_node,
                        g.nodes[&in_node].pos.distance(g.nodes[&out_node].pos),
                    )
                }
            }
        }

        RoadGraph(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_cg() {
        let mut cb = CityGenerator::new();
        let center = cb.add_intersection(Intersection::new([0.0, 0.0].into()));
        let a = cb.add_intersection(Intersection::new([100.0, 0.0].into()));
        let b = cb.add_intersection(Intersection::new([-100.0, 0.0].into()));
        let c = cb.add_intersection(Intersection::new([0.0, 100.0].into()));
        let d = cb.add_intersection(Intersection::new([0.0, -100.0].into()));

        cb.connect(a, center);
        cb.connect(b, center);
        cb.connect(c, center);
        cb.connect(d, center);

        cb.connect(a, c);
        cb.connect(c, b);
        cb.connect(b, d);
        cb.connect(d, a);

        let g = cb.build();

        for (id, _) in g.0.nodes {
            assert_ne!(0, g.0.edges[&id].len());
        }
    }
}
