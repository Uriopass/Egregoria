use super::RoadGraph;
use crate::cars::roads::road_graph_synchronize::ConnectState::{First, Inactive, Unselected};
use crate::cars::roads::Intersection;
use crate::cars::IntersectionComponent;
use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{Movable, MovedEvent, Selectable, SelectedEntity};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::{LineRender, MeshRender, MeshRenderEnum};
use crate::rendering::RED;
use specs::prelude::System;
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
                data.rg.set_intersection_position(&rnc.id, event.new_pos);
                data.rg.calculate_nodes_positions(&rnc.id);
            }
        }

        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data
                .rg
                .add_intersection(Intersection::new(data.mouseinfo.unprojected));
            let intersections = &data.intersections;
            if let Some(x) = data.selected.0.and_then(|x| intersections.get(x)) {
                data.rg.connect(&id, &x.id);
            }
            data.rg.make_entity(
                &id,
                &data.entities,
                &mut data.intersections,
                &mut data.meshrenders,
                &mut data.transforms,
                &mut data.movable,
                &mut data.selectable,
            );

            *data.selected = SelectedEntity(data.rg.intersections()[&id].e);
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.0 {
                if let Some(inter) = data.intersections.get(e) {
                    data.rg.delete_inter(&inter.id, &data.entities);
                }
            }
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
                            if !data.rg.intersections().is_neigh(&interc.id, &interc2.id) {
                                data.rg.connect(&interc.id, &interc2.id);
                            } else {
                                data.rg.disconnect(&interc.id, &interc2.id);
                            }
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
            match data
                .meshrenders
                .get_mut(self.show_connect)
                .and_then(|x| x.orders.get_mut(0))
            {
                Some(MeshRenderEnum::Line(x)) => {
                    x.offset = data.mouseinfo.unprojected - trans.position();
                }
                _ => (),
            }
        }
    }
}

impl RoadGraphSynchronize {
    fn deactive_connect(&mut self, data: &mut RGSData) {
        self.connect_state = Inactive;
        data.meshrenders.get_mut(self.show_connect).unwrap().hide = true;
    }
}
