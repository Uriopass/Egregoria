use serde::{Deserialize, Serialize};
use slotmap::{Key, Keys, SecondaryMap, SlotMap};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Edge<N, W> {
    pub to: N,
    pub weight: W,
}

type EdgeList<N, W> = Vec<Edge<N, W>>;

#[derive(Serialize, Deserialize)]
pub struct Graph<N: Key, T: Copy, W> {
    nodes: SlotMap<N, T>,
    edges: SecondaryMap<N, EdgeList<N, W>>,
    backward_edges: SecondaryMap<N, EdgeList<N, W>>,
}

impl<N: Key + Copy + Eq, T: Copy, W: Clone> Graph<N, T, W> {
    pub fn empty() -> Self {
        Graph {
            nodes: SlotMap::with_key(),
            edges: SecondaryMap::new(),
            backward_edges: SecondaryMap::new(),
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
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: N) -> Option<&mut T> {
        self.nodes.get_mut(id)
    }

    pub fn push(&mut self, value: T) -> N {
        let uuid = self.nodes.insert(value);
        self.edges.insert(uuid, vec![]);
        self.backward_edges.insert(uuid, vec![]);
        uuid
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

    pub fn get_backward_neighs(&self, id: N) -> &EdgeList<N, W> {
        &self.backward_edges[id]
    }

    pub fn set_neighs(&mut self, id: N, neighs: EdgeList<N, W>) {
        self.edges.insert(id, neighs);
    }

    pub fn add_neigh(&mut self, from: N, to: N, weight: W) {
        self.edges[from].push(Edge {
            to,
            weight: weight.clone(),
        });
        self.backward_edges[to].push(Edge { to: from, weight });
    }

    pub fn is_neigh(&self, from: N, to: N) -> bool {
        self.edges[from].iter().any(|x| x.to == to)
    }

    pub fn remove_neigh(&mut self, from: N, to: N) {
        remove_from_list(&mut self.edges, from, to);
        remove_from_list(&mut self.backward_edges, to, from);
    }

    pub fn remove_outbounds(&mut self, id: N) {
        for Edge { to, .. } in &self.edges[id] {
            remove_from_list(&mut self.backward_edges, *to, id);
        }
        self.edges[id].clear();
    }

    pub fn remove_inbounds(&mut self, id: N) {
        for Edge { to, .. } in &self.backward_edges[id] {
            remove_from_list(&mut self.edges, *to, id);
        }
        self.backward_edges[id].clear();
    }

    pub fn remove_node(&mut self, id: N) {
        self.nodes.remove(id);
        for x in self.backward_edges.remove(id).expect("Invalid node id") {
            remove_from_list(&mut self.edges, x.to, id);
        }
        for x in self.edges.remove(id).expect("Invalid node id") {
            remove_from_list(&mut self.backward_edges, x.to, id);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.backward_edges.clear();
        self.edges.clear();
    }
}

impl<T: Copy, N: Key, W> Index<N> for Graph<N, T, W> {
    type Output = T;

    fn index(&self, index: N) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T: Copy, N: Key, W> IndexMut<N> for Graph<N, T, W> {
    fn index_mut(&mut self, index: N) -> &mut Self::Output {
        self.nodes.get_mut(index).unwrap()
    }
}

impl<T: Copy, N: Key, W> Index<&N> for Graph<N, T, W> {
    type Output = T;

    fn index(&self, index: &N) -> &Self::Output {
        &self.nodes[index.clone()]
    }
}

impl<T: Copy, N: Key, W> IndexMut<&N> for Graph<N, T, W> {
    fn index_mut(&mut self, index: &N) -> &mut Self::Output {
        self.nodes.get_mut(index.clone()).unwrap()
    }
}

impl<'a, T: 'a + Copy, N: Key, W> IntoIterator for &'a mut Graph<N, T, W> {
    type Item = (N, &'a mut T);
    type IntoIter = slotmap::IterMut<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.nodes).iter_mut()
    }
}

impl<'a, T: 'a + Copy, N: Key, W> IntoIterator for &'a Graph<N, T, W> {
    type Item = (N, &'a T);
    type IntoIter = slotmap::Iter<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.nodes).iter()
    }
}

fn remove_from_list<N: Key + Eq, W>(hash: &mut SecondaryMap<N, EdgeList<N, W>>, id: N, elem: N) {
    hash.get_mut(id)
        .expect("Invalid node id")
        .retain(|e| e.to != elem);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::new_key_type;

    new_key_type! {
        pub struct NodeID;
    }

    #[test]
    fn test_graph() {
        let mut g: Graph<NodeID, i32, ()> = Graph::empty();
        let a = g.push(0);
        let b = g.push(1);
        let c = g.push(2);

        g.add_neigh(a, b, ());

        assert_eq!(g.get_neighs(a).len(), 1);
        assert_eq!(g.get_backward_neighs(b).get(0).unwrap().to, a);

        g.add_neigh(b, c, ());

        g.remove_node(b);

        assert_eq!(g.len(), 2);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.backward_edges.len(), 2);
        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);

        g.add_neigh(a, c, ());
        g.remove_neigh(a, c);

        assert_eq!(g.get_neighs(a).len(), 0);
        assert_eq!(g.get_backward_neighs(c).len(), 0);
    }
}
