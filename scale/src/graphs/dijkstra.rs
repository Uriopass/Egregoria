#![allow(dead_code)]
use crate::graphs::graph::Graph;
use ordered_float::NotNan;
use rand::distributions::weighted::alias_method::Weight;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::hash_map::RandomState;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

#[derive(Copy, Eq, Clone, PartialEq)]
struct State<N> {
    cost: NotNan<f32>,
    position: N,
}

impl<N: Ord> Ord for State<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Equal)
    }
}

impl<N: Ord> PartialOrd for State<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn dijsktra<N: Copy + Ord + Hash + From<usize>, T>(
    graph: &Graph<N, T>,
    start: N,
) -> HashMap<N, f32> {
    let mut heap = BinaryHeap::new();

    heap.push(State {
        position: start,
        cost: NotNan::from(0.0),
    });

    let mut dist: HashMap<N, f32, RandomState> = HashMap::with_capacity(graph.len());

    for id in graph.ids() {
        dist.insert(*id, f32::MAX);
    }
    dist.insert(start, 0.0);

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
        let mut g: Graph<usize, usize> = Graph::empty();

        let id = g.push(0);
        let id2 = g.push(1);
        let id3 = g.push(2);

        g.set_neighs(
            id,
            vec![
                Edge {
                    to: id2,
                    weight: 1.0,
                },
                Edge {
                    to: id3,
                    weight: 3.0,
                },
            ],
        );
        g.set_neighs(
            id2,
            vec![Edge {
                to: id3,
                weight: 5.0,
            }],
        );

        let dists = dijsktra(&g, id);
        assert_eq!(dists[&id], 0.0);
        assert_eq!(dists[&id2], 1.0);
        assert_eq!(dists[&id3], 3.0);
    }
}
