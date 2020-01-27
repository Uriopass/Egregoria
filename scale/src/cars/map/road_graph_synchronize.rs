use super::RoadGraph;
use crate::cars::map::road_graph_synchronize::ConnectState::{First, Inactive, Unselected};
use crate::cars::map::Intersection;
use crate::cars::IntersectionComponent;
use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::graphs::graph::NodeID;
use crate::interaction::{Movable, MovedEvent, Selectable, SelectedEntity};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::{
    CircleRender, LineRender, MeshRender, MeshRenderEnum,
};
use crate::rendering::RED;
use cgmath::Vector2;
use specs::prelude::*;
use specs::shred::PanicHandler;
use specs::shrev::{EventChannel, ReaderId};

#[derive(PartialEq, Eq, Clone, Copy)]
enum ConnectState {
    Inactive,
    Unselected,
    First(Entity),
}

pub struct RoadGraphSynchronize {
    reader: ReaderId<MovedEvent>,
    connect_state: ConnectState,
    show_connect: Entity,
}

impl RoadGraphSynchronize {
    pub fn new(world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);

        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        let e = world
            .create_entity()
            .with(Transform::new([0.0, 0.0]))
            .with(MeshRender::simple(
                LineRender {
                    offset: [0.0, 0.0].into(),
                    color: RED,
                    thickness: 0.2,
                },
                9,
            ))
            .build();

        Self {
            reader,
            connect_state: Inactive,
            show_connect: e,
        }
    }
}

#[derive(SystemData)]
pub struct RGSData<'a> {
    entities: Entities<'a>,
    rg: Write<'a, RoadGraph, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    meshrenders: WriteStorage<'a, MeshRender>,
    transforms: WriteStorage<'a, Transform>,
    movable: WriteStorage<'a, Movable>,
    selectable: WriteStorage<'a, Selectable>,
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = RGSData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Moved events
        for event in data.moved.read(&mut self.reader) {
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.rg.set_intersection_position(rnc.id, event.new_pos);
                data.rg.calculate_nodes_positions(rnc.id);
                data.rg.save("graph");
            }
        }
        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data
                .rg
                .add_intersection(Intersection::new(data.mouseinfo.unprojected));
            let intersections = &data.intersections;
            if let Some(x) = data.selected.0.and_then(|x| intersections.get(x)) {
                data.rg.connect(id, x.id);
            }
            let e = make_inter_entity(id, data.mouseinfo.unprojected, &mut data);
            *data.selected = SelectedEntity(Some(e));

            data.rg.save("graph");
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.0 {
                if let Some(inter) = data.intersections.get(e) {
                    data.rg.delete_inter(inter.id);
                    data.entities.delete(e).unwrap();
                }
            }
            data.rg.save("graph");
        }

        // Connection handling
        if data.kbinfo.just_pressed.contains(&KeyCode::C) {
            self.connect_state = Unselected;
        }

        if let Some(x) = data.selected.0 {
            if let Some(interc) = data.intersections.get(x) {
                match self.connect_state {
                    Unselected => {
                        self.connect_state = First(x);
                        data.meshrenders.get_mut(self.show_connect).unwrap().hide = false;
                    }
                    First(y) => {
                        let interc2 = data.intersections.get(y).unwrap();
                        if y != x {
                            if !data.rg.intersections().is_neigh(interc.id, interc2.id) {
                                data.rg.connect(interc.id, interc2.id);
                            } else {
                                data.rg.disconnect(interc.id, interc2.id);
                            }
                            data.rg.save("graph");
                            self.deactive_connect(&mut data);
                        }
                    }
                    _ => (),
                }
            } else {
                self.deactive_connect(&mut data);
            }
        } else {
            self.deactive_connect(&mut data);
        }

        if let First(x) = self.connect_state {
            let trans = data.transforms.get(x).unwrap().clone();
            data.transforms
                .get_mut(self.show_connect)
                .unwrap()
                .set_position(trans.position());
            if let Some(MeshRenderEnum::Line(x)) = data
                .meshrenders
                .get_mut(self.show_connect)
                .and_then(|x| x.orders.get_mut(0))
            {
                x.offset = data.mouseinfo.unprojected - trans.position();
            }
        }
    }
}

pub fn make_inter_entity(inter_id: NodeID, inter_pos: Vector2<f32>, data: &mut RGSData) -> Entity {
    data.entities
        .build_entity()
        .with(
            IntersectionComponent { id: inter_id },
            &mut data.intersections,
        )
        .with(
            MeshRender::simple(
                CircleRender {
                    radius: 2.0,
                    color: RED,
                    filled: true,
                    ..CircleRender::default()
                },
                2,
            ),
            &mut data.meshrenders,
        )
        .with(Transform::new(inter_pos), &mut data.transforms)
        .with(Movable, &mut data.movable)
        .with(Selectable, &mut data.selectable)
        .build()
}

impl RoadGraphSynchronize {
    fn deactive_connect(&mut self, data: &mut RGSData) {
        self.connect_state = Inactive;
        data.meshrenders.get_mut(self.show_connect).unwrap().hide = true;
    }
}
