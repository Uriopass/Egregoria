use serde::{Deserialize, Serialize};
use slotmap::secondary::Keys;
use slotmap::{Key, SecondaryMap};
use std::ops::{Index, IndexMut};

type Edge<N, W> = (N, W);

type EdgeList<N, W> = Vec<Edge<N, W>>;

#[derive(Serialize, Deserialize)]
pub struct SecondaryGraph<N: Key, T, W> {
    pub nodes: SecondaryMap<N, T>,
    pub edges: SecondaryMap<N, EdgeList<N, W>>,
    pub backward_edges: SecondaryMap<N, EdgeList<N, W>>,
}

impl<N: Key, T, W> Default for SecondaryGraph<N, T, W> {
    fn default() -> Self {
        SecondaryGraph {
            nodes: SecondaryMap::new(),
            edges: SecondaryMap::new(),
            backward_edges: SecondaryMap::new(),
        }
    }
}

impl<N: Key + Copy + Eq, T: Copy, W: Copy + Clone> SecondaryGraph<N, T, W> {
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
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: N) -> Option<&mut T> {
        self.nodes.get_mut(id)
    }

    pub fn push(&mut self, n: N, value: T) {
        self.nodes.insert(n, value);
        self.edges.insert(n, Vec::new());
        self.backward_edges.insert(n, Vec::new());
    }

    pub fn neighs(&self) -> Vec<(N, &Edge<N, W>)> {
        self.edges
            .iter()
            .map(|(from, el)| el.iter().map(move |x| (from, x)))
            .flatten()
            .collect()
    }

    pub fn get_neighs(&self, id: N) -> &EdgeList<N, W> {
        &self.edges[id]
    }

    pub fn get_neighs_mut(&mut self, id: N) -> &mut EdgeList<N, W> {
        &mut self.edges[id]
    }

    pub fn iter_neighs_owned(&self, id: N) -> impl IntoIterator<Item = (N, W)> + '_ {
        self.edges[id].iter().copied()
    }

    pub fn get_backward_neighs(&self, id: N) -> &EdgeList<N, W> {
        &self.backward_edges[id]
    }

    pub fn set_neighs(&mut self, id: N, neighs: EdgeList<N, W>) {
        self.edges.insert(id, neighs);
    }

    pub fn add_neigh(&mut self, from: N, to: N, weight: W) {
        self.edges[from].push((to, weight));
        self.backward_edges[to].push((from, weight));
    }

    pub fn is_neigh(&self, from: N, to: N) -> bool {
        self.edges[from].iter().any(|x| x.0 == to)
    }

    pub fn remove_neigh(&mut self, from: N, to: N) {
        remove_from_list(&mut self.edges, from, to);
        remove_from_list(&mut self.backward_edges, to, from);
    }

    pub fn remove_outbounds(&mut self, id: N) {
        for (to, _) in &self.edges[id] {
            remove_from_list(&mut self.backward_edges, *to, id);
        }
        self.edges[id].clear();
    }

    pub fn remove_inbounds(&mut self, id: N) {
        for (to, _) in &self.backward_edges[id] {
            remove_from_list(&mut self.edges, *to, id);
        }
        self.backward_edges[id].clear();
    }

    pub fn remove_node(&mut self, id: N) {
        self.nodes.remove(id);
        for (to, _) in self.backward_edges.remove(id).expect("Invalid node id") {
            remove_from_list(&mut self.edges, to, id);
        }
        for (to, _) in self.edges.remove(id).expect("Invalid node id") {
            remove_from_list(&mut self.backward_edges, to, id);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.backward_edges.clear();
        self.edges.clear();
    }
}

impl<T: Copy, N: Key, W> Index<N> for SecondaryGraph<N, T, W> {
    type Output = T;

    fn index(&self, index: N) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T: Copy, N: Key, W> IndexMut<N> for SecondaryGraph<N, T, W> {
    fn index_mut(&mut self, index: N) -> &mut Self::Output {
        self.nodes.get_mut(index).unwrap()
    }
}

impl<T: Copy, N: Key, W> Index<&N> for SecondaryGraph<N, T, W> {
    type Output = T;

    fn index(&self, index: &N) -> &Self::Output {
        &self.nodes[index.clone()]
    }
}

impl<T: Copy, N: Key, W> IndexMut<&N> for SecondaryGraph<N, T, W> {
    fn index_mut(&mut self, index: &N) -> &mut Self::Output {
        self.nodes.get_mut(index.clone()).unwrap()
    }
}

impl<'a, T: 'a + Copy, N: Key, W> IntoIterator for &'a mut SecondaryGraph<N, T, W> {
    type Item = (N, &'a mut T);
    type IntoIter = slotmap::secondary::IterMut<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.nodes).iter_mut()
    }
}

impl<'a, T: 'a + Copy, N: Key, W> IntoIterator for &'a SecondaryGraph<N, T, W> {
    type Item = (N, &'a T);
    type IntoIter = slotmap::secondary::Iter<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.nodes).iter()
    }
}

fn remove_from_list<N: Key + Eq, W>(hash: &mut SecondaryMap<N, EdgeList<N, W>>, id: N, elem: N) {
    hash.get_mut(id)
        .expect("Invalid node id")
        .retain(|(to, _)| *to != elem);
}
