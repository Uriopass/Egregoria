use super::{Intersection, RoadNode};
use crate::cars::{IntersectionComponent, RoadNodeComponent};
use crate::graphs::graph::{Edge, Graph, NodeID};
use crate::interaction::{Movable, Selectable};
use crate::physics::physics_components::Transform;
use cgmath::Vector2;
use cgmath::{InnerSpace, MetricSpace};
use specs::{Entities, WriteStorage};

pub struct RoadGraph {
    nodes: Graph<RoadNode>,
    intersections: Graph<Intersection>,
    pub dirty: bool,
}

impl RoadGraph {
    pub fn new() -> RoadGraph {
        RoadGraph {
            intersections: Graph::new(),
            nodes: Graph::new(),
            dirty: true,
        }
    }

    pub fn nodes(&self) -> &Graph<RoadNode> {
        &self.nodes
    }
    pub fn intersections(&self) -> &Graph<Intersection> {
        &self.intersections
    }

    pub fn set_node_position(&mut self, i: NodeID, pos: Vector2<f32>) {
        self.nodes.nodes.entry(i).and_modify(|x| x.pos = pos);
    }

    pub fn set_intersection_position(&mut self, i: NodeID, pos: Vector2<f32>) {
        self.intersections
            .nodes
            .entry(i)
            .and_modify(|x| x.pos = pos);
    }

    pub fn add_intersection(&mut self, i: Intersection) -> NodeID {
        self.dirty = true;
        self.intersections.add_node(i)
    }

    pub fn recalculate_inter(&mut self, i: NodeID, transforms: &mut WriteStorage<Transform>) {
        let inter = &self.intersections.nodes[&i];
        let center = inter.pos;

        for (to, node_id) in &inter.out_nodes {
            let inter2 = &self.intersections.nodes[to];
            let center2 = inter2.pos;

            let dir = (center2 - center).normalize();
            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);
            {
                let rn = self.nodes.nodes.get_mut(node_id).unwrap();
                rn.pos = center + dir * 25.0 - nor * 4.0;
                transforms
                    .get_mut(rn.e.unwrap())
                    .unwrap()
                    .set_position(rn.pos);
            }
            {
                let rn2 = self.nodes.nodes.get_mut(&inter2.in_nodes[&i]).unwrap();
                rn2.pos = center2 - dir * 25.0 - nor * 4.0;
                transforms
                    .get_mut(rn2.e.unwrap())
                    .unwrap()
                    .set_position(rn2.pos);
            }
        }

        for (to, node_id) in &inter.in_nodes {
            let inter2 = &self.intersections.nodes[to];
            let center2 = inter2.pos;

            let dir = (center2 - center).normalize();
            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            {
                let rn = self.nodes.nodes.get_mut(node_id).unwrap();
                rn.pos = center + dir * 25.0 + nor * 4.0;
                transforms
                    .get_mut(rn.e.unwrap())
                    .unwrap()
                    .set_position(rn.pos);
            }
            {
                let rn2 = self.nodes.nodes.get_mut(&inter2.out_nodes[&i]).unwrap();
                rn2.pos = center2 - dir * 25.0 + nor * 4.0;
                transforms
                    .get_mut(rn2.e.unwrap())
                    .unwrap()
                    .set_position(rn2.pos);
            }
        }
    }

    pub fn populate_entities<'a>(
        &mut self,
        entities: &Entities<'a>,
        rnc: &mut WriteStorage<'a, RoadNodeComponent>,
        inters: &mut WriteStorage<'a, IntersectionComponent>,
        transforms: &mut WriteStorage<'a, Transform>,
        movable: &mut WriteStorage<'a, Movable>,
        selectable: &mut WriteStorage<'a, Selectable>,
    ) {
        for (n, rn) in &mut self.nodes.nodes {
            if rn.e.is_none() {
                rn.e = Some(
                    entities
                        .build_entity()
                        .with(RoadNodeComponent { id: *n }, rnc)
                        .with(Transform::new(rn.pos), transforms)
                        .with(Movable, movable)
                        .with(Selectable, selectable)
                        .build(),
                );
            }
        }

        for (n, rn) in &mut self.intersections.nodes {
            if rn.e.is_none() {
                rn.e = Some(
                    entities
                        .build_entity()
                        .with(IntersectionComponent { id: *n }, inters)
                        .with(Transform::new(rn.pos), transforms)
                        .with(Movable, movable)
                        .with(Selectable, selectable)
                        .build(),
                );
            }
        }
    }

    pub fn closest_node(&self, pos: Vector2<f32>) -> NodeID {
        let mut id: NodeID = *self.nodes.ids().next().unwrap();
        let mut min_dist = self.nodes.nodes.get(&id).unwrap().pos.distance2(pos);

        for (key, value) in &self.nodes.nodes {
            let dist = pos.distance2(value.pos);
            if dist < min_dist {
                id = *key;
                min_dist = dist;
            }
        }
        id
    }

    pub fn connect(&mut self, a: NodeID, b: NodeID) {
        self.connect_directional(a, b);
        self.connect_directional(b, a);
    }

    pub fn connect_directional(&mut self, from: NodeID, to: NodeID) {
        self.intersections.add_neigh(from, to, 1.0);

        let inter = &self.intersections.nodes[&from];
        let center = inter.pos;

        let inter2 = &self.intersections.nodes[&to];
        let center2 = inter2.pos;

        let dir = (center2 - center).normalize();
        let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

        let rn_out = RoadNode::new(center + dir * 25.0 - nor * 4.0);
        let rn_in = RoadNode::new(center2 - dir * 25.0 - nor * 4.0);

        let out_id = self.nodes.add_node(rn_out);
        let in_id = self.nodes.add_node(rn_in);
        self.nodes
            .add_neigh(out_id, in_id, rn_out.pos.distance(rn_in.pos));

        let inter = self.intersections.nodes.get_mut(&from).unwrap();
        inter.out_nodes.insert(to, out_id);
        for (_, in_id) in &inter.in_nodes {
            self.nodes.add_neigh(*in_id, out_id, 1.0); // FIXME: Use actual internal road length
        }

        let inter2 = self.intersections.nodes.get_mut(&to).unwrap();
        inter2.in_nodes.insert(from, in_id);
        for (_, out) in &inter2.out_nodes {
            self.nodes.add_neigh(in_id, *out, 1.0);
        }

        self.dirty = true;
    }
}
