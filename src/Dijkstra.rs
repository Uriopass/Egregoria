#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct Node {
    value: i32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: i32,
    position: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Edge {
    to: usize,
    cost: i32,
}

pub fn dijsktra(nodes: &Vec<Node>, edges: &Vec<Vec<Edge>>, start: usize) -> Vec<i32> {
    let mut heap = BinaryHeap::new();

    heap.push(State {
        position: start,
        cost: 0,
    });

    let mut dist = vec![std::i32::MAX; edges.len()];

    dist[start] = 0;

    while let Some(State { cost, position }) = heap.pop() {
        if cost > dist[position] {
            continue;
        }

        for nei in &edges[position] {
            let next = State {
                position: nei.to,
                cost: dist[position] + nei.cost,
            };

            if next.cost < dist[nei.to] {
                dist[nei.to] = next.cost;
                heap.push(next);
            }
        }
    }
    dist
}
