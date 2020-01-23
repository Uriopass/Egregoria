use super::{Intersection, RoadNode};
use crate::cars::map::TrafficLight::Always;
use crate::cars::map::{TrafficLight, TrafficLightSchedule};
use crate::graphs::graph::{Edge, Graph, NodeID};
use cgmath::Vector2;
use cgmath::{InnerSpace, MetricSpace};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::Sub;

#[derive(Serialize, Deserialize)]
pub struct RoadGraph {
    nodes: Graph<RoadNode>,
    intersections: Graph<Intersection>,
}

impl RoadGraph {
    pub fn empty() -> RoadGraph {
        RoadGraph {
            intersections: Graph::empty(),
            nodes: Graph::empty(),
        }
    }

    pub fn nodes(&self) -> &Graph<RoadNode> {
        &self.nodes
    }
    pub fn intersections(&self) -> &Graph<Intersection> {
        &self.intersections
    }

    pub fn set_node_position(&mut self, i: NodeID, pos: Vector2<f32>) {
        if let Some(x) = self.nodes.get_mut(i) {
            x.pos = pos
        }
    }

    pub fn set_intersection_position(&mut self, i: NodeID, pos: Vector2<f32>) {
        if let Some(x) = self.intersections.get_mut(i) {
            x.pos = pos
        }
    }

    pub fn add_intersection(&mut self, i: Intersection) -> NodeID {
        self.intersections.push(i)
    }

    fn pseudo_angle(v: Vector2<f32>) -> f32 {
        debug_assert!(v.magnitude2().sub(1.0).abs() <= 1e-5);
        let dx = v.x;
        let dy = v.y;
        let p = dx / (dx.abs() + dy.abs());

        if dy < 0.0 {
            p - 1.0
        } else {
            1.0 - p
        }
    }

    pub fn update_traffic_lights(&mut self, i: NodeID) {
        let inter = &self.intersections[&i];
        if inter.in_nodes.len() <= 2 {
            for (_, id) in inter.in_nodes.iter() {
                self.nodes[id].light = Always;
            }
            return;
        }
        let mut in_nodes: Vec<&NodeID> = inter.in_nodes.values().collect();
        in_nodes.sort_by_key(|x| {
            OrderedFloat(Self::pseudo_angle(
                (self.nodes[*x].pos - inter.pos).normalize(),
            ))
        });

        println!("inter: {:?}", inter.pos);

        let cycle_size = 10;
        let orange_length = 5;
        for (i, id) in in_nodes.into_iter().enumerate() {
            println!(
                "| {:?} {:?}",
                self.nodes[id].pos,
                Self::pseudo_angle((self.nodes[id].pos - inter.pos).normalize())
            );
            self.nodes[id].light = TrafficLight::Periodic(TrafficLightSchedule::from_basic(
                cycle_size,
                orange_length,
                cycle_size + orange_length,
                if i % 2 == 0 {
                    cycle_size + orange_length
                } else {
                    0
                },
            ));
        }
    }

    pub fn calculate_nodes_positions(&mut self, i: NodeID) {
        let inter = &self.intersections[i];
        let center = inter.pos;

        for (to, node_id) in &inter.out_nodes {
            let inter2 = &self.intersections[to];
            let center2 = inter2.pos;

            let diff = center2 - center;
            let inter_length = diff.magnitude().max(1e-8);
            let dir = (center2 - center) / inter_length;

            let inter_length = (inter_length / 2.0).min(25.0);

            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            let rn = self.nodes.get_mut(*node_id).unwrap();
            rn.pos = center + dir * inter_length - nor * 4.0;

            let rn2 = self.nodes.get_mut(inter2.in_nodes[&i]).unwrap();
            rn2.pos = center2 - dir * inter_length - nor * 4.0;
        }

        for (to, node_id) in &inter.in_nodes {
            let inter2 = &self.intersections[to];
            let center2 = inter2.pos;

            let diff = center2 - center;
            let inter_length = diff.magnitude();
            let dir = (center2 - center) / inter_length;

            let inter_length = (inter_length / 2.0).min(25.0);

            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            let rn = self.nodes.get_mut(*node_id).unwrap();
            rn.pos = center + dir * inter_length + nor * 4.0;

            let rn2 = self.nodes.get_mut(inter2.out_nodes[&i]).unwrap();
            rn2.pos = center2 - dir * inter_length + nor * 4.0;
        }
    }

    pub fn closest_node(&self, pos: Vector2<f32>) -> NodeID {
        let mut id: NodeID = *self.nodes.ids().next().unwrap();
        let mut min_dist = self.nodes.get(id).unwrap().pos.distance2(pos);

        for (key, value) in &self.nodes {
            let dist = pos.distance2(value.pos);
            if dist < min_dist {
                id = *key;
                min_dist = dist;
            }
        }
        id
    }

    pub fn delete_inter(&mut self, id: NodeID) {
        for Edge { to, .. } in self.intersections.get_neighs(id).clone() {
            self.disconnect_directional(id, to);
        }
        for Edge { to, .. } in self.intersections.get_backward_neighs(id).clone() {
            self.disconnect_directional(to, id);
        }
        self.intersections.remove_node(id);
    }

    pub fn disconnect(&mut self, a: NodeID, b: NodeID) {
        self.disconnect_directional(a, b);
        self.disconnect_directional(b, a);
    }

    pub fn disconnect_directional(&mut self, from: NodeID, to: NodeID) {
        self.intersections.remove_neigh(from, to);
        let inter_from_node = &self.intersections[&from].out_nodes[&to];
        let inter_to_node = &self.intersections[&to].in_nodes[&from];

        self.nodes.remove_node(*inter_from_node);
        self.nodes.remove_node(*inter_to_node);

        self.intersections
            .get_mut(from)
            .unwrap()
            .out_nodes
            .remove(&to);

        self.intersections
            .get_mut(to)
            .unwrap()
            .in_nodes
            .remove(&from);
        self.update_traffic_lights(to);
    }

    pub fn connect(&mut self, a: NodeID, b: NodeID) {
        self.connect_directional(a, b);
        self.connect_directional(b, a);
    }

    pub fn connect_directional(&mut self, from: NodeID, to: NodeID) {
        if self.intersections[from].pos == self.intersections[to].pos {
            println!("Couldn't connect two intersections because they are at the same place.");
            return;
        }
        self.intersections.add_neigh(from, to, 1.0);

        let rn_out = RoadNode::new([0.0, 0.0].into());
        let rn_in = RoadNode::new([0.0, 0.0].into());

        let out_id = self.nodes.push(rn_out);
        let in_id = self.nodes.push(rn_in);
        self.nodes.add_neigh(out_id, in_id, 0.0);

        let inter = self.intersections.get_mut(from).unwrap();
        inter.out_nodes.insert(to, out_id);
        for (from_id, in_id) in &inter.in_nodes {
            if *from_id == to {
                continue;
            }
            self.nodes.add_neigh(*in_id, out_id, 1.0); // FIXME: Use actual internal road length
        }

        let inter2 = self.intersections.get_mut(to).unwrap();
        inter2.in_nodes.insert(from, in_id);
        for (to_id, out) in &inter2.out_nodes {
            if *to_id == from {
                continue;
            }
            self.nodes.add_neigh(in_id, *out, 1.0);
        }

        self.calculate_nodes_positions(from);
        self.update_traffic_lights(to);
    }
}
