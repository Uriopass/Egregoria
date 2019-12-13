#![allow(dead_code)]

use crate::graphs::graph::{Graph, NodeID};
use ordered_float::NotNan;
use rand::distributions::weighted::alias_method::Weight;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::hash_map::RandomState;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Eq, Clone, PartialEq)]
struct State {
    cost: NotNan<f32>,
    position: NodeID,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Equal)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn dijsktra<T>(graph: &Graph<T>, start: NodeID) -> HashMap<NodeID, f32> {
    let mut heap = BinaryHeap::new();

    heap.push(State {
        position: start,
        cost: NotNan::from(0.),
    });

    let mut dist: HashMap<NodeID, f32, RandomState> = HashMap::with_capacity(graph.len());

    for id in graph.ids() {
        dist.insert(*id, f32::MAX);
    }
    dist.insert(start, 0.);

    while let Some(State { cost, position }) = heap.pop() {
        if cost.into_inner() > dist[&position] {
            continue;
        }

        for nei in graph.get_neighs(position) {
            let v = dist[&position] + nei.weight;
            if v < dist[&nei.to] {
                dist.insert(nei.to, v);
                heap.push(State {
                    position: nei.to,
                    cost: NotNan::from(v),
                });
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::graph::Edge;

    #[test]
    fn test_dijkstra() {
        let mut g: Graph<usize> = Graph::new();

        let id = g.add_node(0);
        let id2 = g.add_node(1);
        let id3 = g.add_node(2);

        g.set_neighs(
            id,
            vec![
                Edge {
                    to: id2,
                    weight: 1.,
                },
                Edge {
                    to: id3,
                    weight: 3.,
                },
            ],
        );
        g.set_neighs(
            id2,
            vec![Edge {
                to: id3,
                weight: 5.,
            }],
        );

        let dists = dijsktra(&g, id);
        assert_eq!(dists[&id], 0.);
        assert_eq!(dists[&id2], 1.);
        assert_eq!(dists[&id3], 3.);
    }
}
