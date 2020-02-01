use serde::{Deserialize, Serialize};
use std::collections::hash_map::Keys;
use std::collections::{hash_map, HashMap};
use std::hash::Hash;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Edge<N> {
    pub to: N,
    pub weight: f32,
}

type EdgeList<N> = Vec<Edge<N>>;

#[derive(Serialize, Deserialize)]
pub struct Graph<N: Ord + Hash + Eq, T> {
    nodes: HashMap<N, T>,
    edges: HashMap<N, EdgeList<N>>,
    backward_edges: HashMap<N, EdgeList<N>>,
    uuid: usize,
}

impl<N: Ord + Hash + Eq + Copy + From<usize>, T> Graph<N, T> {
    pub fn empty() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            backward_edges: HashMap::new(),
            uuid: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn ids(&self) -> Keys<N, T> {
        self.nodes.keys()
    }

    pub fn get(&self, id: N) -> Option<&T> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: N) -> Option<&mut T> {
        self.nodes.get_mut(&id)
    }

    pub fn push(&mut self, value: T) -> N {
        let uuid = N::from(self.uuid);
        self.nodes.insert(uuid, value);
        self.edges.insert(uuid, vec![]);
        self.backward_edges.insert(uuid, vec![]);
        self.uuid += 1;
        uuid
    }

    pub fn neighs(&self) -> Vec<(&N, &Edge<N>)> {
        self.edges
            .iter()
            .map(|(from, el)| el.iter().map(move |x| (from, x)))
            .flatten()
            .collect()
    }

    pub fn get_neighs(&self, id: N) -> &EdgeList<N> {
        self.edges.get(&id).expect("Invalid node id")
    }

    pub fn get_backward_neighs(&self, id: N) -> &EdgeList<N> {
        self.backward_edges.get(&id).expect("Invalid node id")
    }

    pub fn set_neighs(&mut self, id: N, neighs: EdgeList<N>) {
        self.edges.insert(id, neighs);
    }

    pub fn add_neigh(&mut self, from: N, to: N, weight: f32) {
        self.edges
            .get_mut(&from)
            .expect("Invalid node id")
            .push(Edge { to, weight });
        self.backward_edges
            .get_mut(&to)
            .expect("Invalid node id")
            .push(Edge { to: from, weight });
    }

    pub fn is_neigh(&self, from: N, to: N) -> bool {
        self.edges[&from].iter().any(|x| x.to == to)
    }

    pub fn remove_neigh(&mut self, from: N, to: N) {
        remove_from_list(&mut self.edges, from, to);
        remove_from_list(&mut self.backward_edges, to, from);
    }

    pub fn remove_outbounds(&mut self, id: N) {
        for Edge { to, .. } in &self.edges[&id] {
            remove_from_list(&mut self.backward_edges, *to, id);
        }
        self.edges.get_mut(&id).unwrap().clear();
    }

    pub fn remove_inbounds(&mut self, id: N) {
        for Edge { to, .. } in &self.backward_edges[&id] {
            remove_from_list(&mut self.edges, *to, id);
        }
        self.backward_edges.get_mut(&id).unwrap().clear();
    }

    pub fn remove_node(&mut self, id: N) {
        self.nodes.remove(&id);
        for x in self.backward_edges.remove(&id).expect("Invalid node id") {
            remove_from_list(&mut self.edges, x.to, id);
        }
        for x in self.edges.remove(&id).expect("Invalid node id") {
            remove_from_list(&mut self.backward_edges, x.to, id);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.backward_edges.clear();
        self.edges.clear();
        self.uuid = 0;
    }
}

impl<T, N: Hash + Ord> Index<N> for Graph<N, T> {
    type Output = T;

    fn index(&self, index: N) -> &Self::Output {
        &self.nodes[&index]
    }
}

impl<T, N: Hash + Ord> IndexMut<N> for Graph<N, T> {
    fn index_mut(&mut self, index: N) -> &mut Self::Output {
        self.nodes.get_mut(&index).unwrap()
    }
}

impl<T, N: Hash + Ord> Index<&N> for Graph<N, T> {
    type Output = T;

    fn index(&self, index: &N) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T, N: Hash + Ord> IndexMut<&N> for Graph<N, T> {
    fn index_mut(&mut self, index: &N) -> &mut Self::Output {
        self.nodes.get_mut(index).unwrap()
    }
}

impl<'a, T: 'a, N: Hash + Ord> IntoIterator for &'a mut Graph<N, T> {
    type Item = (&'a N, &'a mut T);
    type IntoIter = hash_map::IterMut<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.nodes).iter_mut()
    }
}

impl<'a, T: 'a, N: Hash + Ord> IntoIterator for &'a Graph<N, T> {
    type Item = (&'a N, &'a T);
    type IntoIter = hash_map::Iter<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.nodes).iter()
    }
}

fn remove_from_list<N: Eq + Hash>(hash: &mut HashMap<N, EdgeList<N>>, id: N, elem: N) {
    hash.get_mut(&id)
        .expect("Invalid node id")
        .retain(|e| e.to != elem);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        let mut g = Graph::empty();
        let a = g.push(0);
        let b = g.push(1);
        let c = g.push(2);

        g.add_neigh(a, b, 1.0);

        assert_eq!(g.get_neighs(a).len(), 1);
        assert_eq!(g.get_backward_neighs(b).get(0).unwrap().to, a);

        g.add_neigh(b, c, 1.0);

        g.remove_node(b);

        assert_eq!(g.len(), 2);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.backward_edges.len(), 2);
        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);

        g.add_neigh(a, c, 1.0);
        g.remove_neigh(a, c);

        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);
    }
}
