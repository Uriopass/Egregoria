use std::collections::hash_map::Keys;
use std::collections::HashMap;

pub struct Edge {
    pub to: NodeID,
    pub weight: f32,
}

type EdgeList = Vec<Edge>;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct NodeID(usize);

pub struct Graph<T> {
    pub nodes: HashMap<NodeID, T>,
    pub edges: HashMap<NodeID, EdgeList>,
    pub backward_edges: HashMap<NodeID, EdgeList>,
    uuid: usize,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
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

    pub fn ids(&self) -> Keys<NodeID, T> {
        self.nodes.keys()
    }

    pub fn add_node(&mut self, data: T) -> NodeID {
        let uuid = NodeID(self.uuid);
        self.nodes.insert(uuid, data);
        self.edges.insert(uuid, vec![]);
        self.backward_edges.insert(uuid, vec![]);
        self.uuid += 1;
        uuid
    }

    pub fn get_neighs(&self, id: NodeID) -> &EdgeList {
        self.edges.get(&id).expect("Invalid node id")
    }

    pub fn set_neighs(&mut self, id: NodeID, neighs: EdgeList) {
        self.edges.insert(id, neighs);
    }

    pub fn add_neigh(&mut self, id: NodeID, to: NodeID, weight: f32) {
        self.edges
            .get_mut(&id)
            .expect("Invalid node id")
            .push(Edge { to, weight })
    }

    pub fn remove_neigh(&mut self, id: NodeID, to: NodeID) {
        self.edges
            .get_mut(&id)
            .expect("Invalid node id")
            .retain(|e| e.to != to);
    }

    pub fn remove_node(&mut self, id: NodeID) {
        self.nodes.remove(&id);
        for x in self.backward_edges.remove(&id).expect("Invalid node id") {
            self.remove_neigh(x.to, id)
        }
    }
}
