use crate::map_model::{IntersectionID, Intersections, NavMesh, NavNode, NavNodeID, Road, RoadID};
use cgmath::InnerSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LaneType {
    Driving,
    Biking,
    Bus,
    Construction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,
    pub lane_type: LaneType,

    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,

    pub src_node: Option<NavNodeID>,
    pub dst_node: Option<NavNodeID>,

    pub direction: LaneDirection,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LanePattern {
    pub name: String,
    pub lanes_forward: Vec<LaneType>,
    pub lanes_backward: Vec<LaneType>,
}
impl PartialEq for LanePattern {
    fn eq(&self, other: &Self) -> bool {
        self.lanes_forward == other.lanes_forward && self.lanes_backward == other.lanes_backward
    }
}
impl Eq for LanePattern {}

impl LanePattern {
    pub fn one_way(n_lanes: usize) -> Self {
        assert!(n_lanes > 0);
        LanePattern {
            lanes_backward: vec![],
            lanes_forward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
            name: if n_lanes == 1 {
                "One way".to_owned()
            } else {
                format!("One way {} lanes", n_lanes)
            },
        }
    }

    pub fn two_way(n_lanes: usize) -> Self {
        assert!(n_lanes > 0);
        LanePattern {
            lanes_backward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
            lanes_forward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
            name: if n_lanes == 1 {
                "Two way".to_owned()
            } else {
                format!("Two way {} lanes", n_lanes)
            },
        }
    }
}

impl Lane {
    pub fn get_inter_node(&self, id: IntersectionID) -> NavNodeID {
        if id == self.src_i {
            self.src_node
        } else if id == self.dst_i {
            self.dst_node
        } else {
            panic!("Trying to get node to not corresponding intersection");
        }
        .unwrap_or_else(|| {
            let v: &'static String =
                Box::leak(Box::new(format!("Lane {:?} not generated yet", self.id)));
            panic!(v)
        })
    }

    fn get_node_pos(
        &self,
        inter_id: IntersectionID,
        incoming: bool,
        inters: &Intersections,
        parent_road: &Road,
    ) -> Vector2<f32> {
        let inter = &inters[inter_id];

        let mut lane_dist = parent_road.idx_unchecked(self.id) as f32;
        let dir = parent_road.dir_from(inter);
        let dir_normal: Vector2<f32> = if incoming {
            [-dir.y, dir.x].into()
        } else {
            [dir.y, -dir.x].into()
        };

        if parent_road.is_one_way() {
            lane_dist -= 0.5 + parent_road.n_lanes() as f32 / 2.0;
        }

        let mindist = parent_road.length() / 2.0 - 1.0;

        inter.pos + dir * inter.interface_radius.min(mindist) + dir_normal * lane_dist as f32 * 8.0
    }

    pub fn gen_navmesh(
        &mut self,
        intersections: &Intersections,
        parent_road: &Road,
        mesh: &mut NavMesh,
    ) {
        let pos = self.get_node_pos(
            self.src_i,
            self.direction == LaneDirection::Backward,
            intersections,
            parent_road,
        );
        match self.src_node {
            None => {
                self.src_node = Some(mesh.push(NavNode::new(pos)));
            }
            Some(id) => mesh[id].pos = pos,
        }

        let pos = self.get_node_pos(
            self.dst_i,
            self.direction == LaneDirection::Forward,
            intersections,
            parent_road,
        );
        match self.dst_node {
            None => {
                self.dst_node = Some(mesh.push(NavNode::new(pos)));
                if self.direction == LaneDirection::Forward {
                    mesh.add_neigh(self.src_node.unwrap(), self.dst_node.unwrap(), 1.0);
                } else {
                    mesh.add_neigh(self.dst_node.unwrap(), self.src_node.unwrap(), 1.0);
                }
            }
            Some(id) => mesh[id].pos = pos,
        }
    }

    pub fn clean(&mut self, mesh: &mut NavMesh) {
        mesh.remove_node(self.src_node.take().expect("Lane not generated"));
        mesh.remove_node(self.dst_node.take().expect("Lane not generated"));
    }

    pub fn get_orientation_vec(&self, mesh: &NavMesh) -> Vector2<f32> {
        let src = mesh[self.src_node.unwrap()].pos;
        let dst = mesh[self.dst_node.unwrap()].pos;

        assert_ne!(dst, src);

        let vec = (dst - src).normalize();
        match self.direction {
            LaneDirection::Forward => vec,
            LaneDirection::Backward => -vec,
        }
    }
}
