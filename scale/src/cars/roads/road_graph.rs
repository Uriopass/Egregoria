use crate::cars::{IntersectionComponent, RoadNodeComponent};
use crate::graphs::graph::{Edge, Graph, NodeID};
use cgmath::Vector2;
use specs::prelude::*;

use super::{Intersection, RoadNode};
use crate::interaction::{Movable, MovedEvent, Selectable};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::{CircleRender, LineToRender, MeshRender};
use crate::rendering::{GREEN, RED, WHITE};
use cgmath::{InnerSpace, MetricSpace};
use specs::shred::PanicHandler;
use specs::shrev::{EventChannel, ReaderId};

pub struct RoadGraph {
    nodes: Graph<RoadNode>,
    intersections: Graph<Intersection>,
    pub dirty: bool,
}

pub struct RoadGraphSynchronize {
    reader: ReaderId<MovedEvent>,
}

impl RoadGraphSynchronize {
    pub fn new(world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<MovedEvent>>()
            .register_reader();
        Self { reader }
    }
}

#[derive(SystemData)]
pub struct RGSData<'a> {
    entities: Entities<'a>,
    rg: Write<'a, RoadGraph, PanicHandler>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    roadnodescomponents: WriteStorage<'a, RoadNodeComponent>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    meshrenders: WriteStorage<'a, MeshRender>,
    transforms: WriteStorage<'a, Transform>,
    movable: WriteStorage<'a, Movable>,
    selectable: WriteStorage<'a, Selectable>,
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = RGSData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for event in data.moved.read(&mut self.reader) {
            if let Some(rnc) = data.roadnodescomponents.get(event.entity) {
                data.rg
                    .nodes
                    .nodes
                    .entry(rnc.id)
                    .and_modify(|x| x.pos = event.new_pos);
            }
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.rg
                    .intersections
                    .nodes
                    .entry(rnc.id)
                    .and_modify(|x| x.pos = event.new_pos);

                data.rg.recalculate_inter(rnc.id, &mut data.transforms);
            }
        }

        if data.rg.dirty {
            data.rg.dirty = false;
            data.rg.populate_entities(
                &mut data.entities,
                &mut data.roadnodescomponents,
                &mut data.intersections,
                &mut data.transforms,
                &mut data.movable,
                &mut data.selectable,
            );

            {
                for (n, r) in &data.rg.nodes.nodes {
                    let e = r.e;
                    if e.is_none() {
                        continue;
                    }
                    let e = e.unwrap();

                    let mut meshb = MeshRender::from(CircleRender {
                        radius: 1.0,
                        color: GREEN,
                        filled: true,
                        ..Default::default()
                    });

                    for nei in data.rg.nodes.get_neighs(*n) {
                        let e_nei = data.rg.nodes.nodes[&nei.to].e;
                        if e_nei.is_none() {
                            continue;
                        }
                        let e_nei = e_nei.unwrap();
                        meshb.add(LineToRender {
                            color: WHITE,
                            to: e_nei,
                        });
                    }

                    data.meshrenders
                        .insert(e, meshb)
                        .expect("Error inserting mesh for graph");
                }
            }

            {
                for (n, r) in &data.rg.intersections.nodes {
                    let e = r.e;
                    if e.is_none() {
                        continue;
                    }
                    let e = e.unwrap();

                    let mut meshb = MeshRender::from(CircleRender {
                        radius: 3.0,
                        color: RED,
                        filled: true,
                        ..Default::default()
                    });

                    for nei in data.rg.intersections.get_neighs(*n) {
                        let e_nei = data.rg.intersections.nodes[&nei.to].e;
                        if e_nei.is_none() {
                            continue;
                        }
                        let e_nei = e_nei.unwrap();
                        meshb.add(LineToRender {
                            color: RED,
                            to: e_nei,
                        });
                    }

                    data.meshrenders
                        .insert(e, meshb)
                        .expect("Error inserting mesh for graph");
                }
            }
        }
    }
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

    pub fn add_intersection(&mut self, i: Intersection) -> NodeID {
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
        entities: &mut Entities<'a>,
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
        self.intersections.add_neigh(a, b, 1.0);
        self.intersections.add_neigh(b, a, 1.0);
    }

    pub fn build_nodes(&mut self) {
        self.nodes.clear();

        let g = &mut self.nodes;

        let mut inserts: Vec<(NodeID, NodeID, NodeID, NodeID)> = vec![];

        for (from, Edge { to, .. }) in &self.intersections.neighs() {
            let inter = &self.intersections.nodes[from];
            let center = inter.pos;

            let inter2 = &self.intersections.nodes[to];
            let center2 = inter2.pos;

            let dir = (center2 - center).normalize();
            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            let rn_out = RoadNode::new(center + dir * 25.0 - nor * 4.0);
            let rn_in = RoadNode::new(center2 - dir * 25.0 - nor * 4.0);

            let out_id = g.add_node(rn_out);
            let in_id = g.add_node(rn_in);
            g.add_neigh(out_id, in_id, rn_out.pos.distance(rn_in.pos));

            inserts.push((*from, *to, in_id, out_id));
        }
        println!("{} intersections added", inserts.len());

        for (from, to, in_id, out_id) in inserts {
            self.intersections
                .nodes
                .get_mut(&to)
                .unwrap()
                .in_nodes
                .insert(from, in_id);

            self.intersections
                .nodes
                .get_mut(&from)
                .unwrap()
                .out_nodes
                .insert(to, out_id);
        }

        for (_, inter) in &mut self.intersections.nodes {
            let l = inter.out_nodes.len();
            for (id, in_node) in &inter.in_nodes {
                for (id2, out_node) in &inter.out_nodes {
                    if id != id2 || l == 1 {
                        g.add_neigh(*in_node, *out_node, 1.0);
                    }
                }
            }
        }
    }
}
