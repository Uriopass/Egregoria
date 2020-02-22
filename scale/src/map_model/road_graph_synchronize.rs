use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{MovedEvent, SelectedEntity};
use crate::map_model::road_graph_synchronize::ConnectState::{First, Inactive, Unselected};
use crate::map_model::{make_inter_entity, IntersectionComponent, LanePattern, Map};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{LineRender, MeshRender, MeshRenderEnum};
use crate::rendering::RED;
use specs::prelude::*;
use specs::shred::{DynamicSystemData, PanicHandler};
use specs::shrev::{EventChannel, ReaderId};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConnectState {
    Inactive,
    Unselected,
    First(Entity),
}

pub struct RoadGraphSynchronize;

pub struct RoadGraphSynchronizeState {
    reader: ReaderId<MovedEvent>,
    pub connect_state: ConnectState,
    pub show_connect: Entity,
    pub pattern: LanePattern,
}

impl RoadGraphSynchronizeState {
    pub fn new(world: &mut World) -> Self {
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
                    thickness: 1.0,
                },
                9,
            ))
            .build();
        Self {
            reader,
            connect_state: Inactive,
            show_connect: e,
            pattern: LanePattern::two_way(1),
        }
    }
}

#[derive(SystemData)]
pub struct RGSData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    self_state: Write<'a, RoadGraphSynchronizeState, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    meshrenders: WriteStorage<'a, MeshRender>,
    transforms: WriteStorage<'a, Transform>,
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = RGSData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Moved events
        for event in data.moved.read(&mut data.self_state.reader) {
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.map.move_intersection(rnc.id, event.new_pos);
            }
        }

        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data.map.add_intersection(data.mouseinfo.unprojected);
            let intersections = &data.intersections;
            if let Some(x) = data.selected.0.and_then(|x| intersections.get(x)) {
                data.map.connect(id, x.id, &data.self_state.pattern);
            }
            let e = make_inter_entity(
                &data.map.intersections[id],
                data.mouseinfo.unprojected,
                &data.lazy,
                &data.entities,
            );
            *data.selected = SelectedEntity(Some(e));
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.0 {
                if let Some(inter) = data.intersections.get(e) {
                    data.map.remove_intersection(inter.id);
                    data.intersections.remove(e);
                    data.entities.delete(e).unwrap();
                }
            }
            data.self_state.deactive_connect(&mut data.meshrenders);
        }

        // Connection handling
        if data.kbinfo.just_pressed.contains(&KeyCode::C) {
            match data.self_state.connect_state {
                First(_) => data.self_state.deactive_connect(&mut data.meshrenders),
                _ => data.self_state.connect_state = Unselected,
            }
        }

        if let Some(x) = data.selected.0 {
            if let Some(interc) = data.intersections.get(x) {
                data.map.set_intersection_radius(interc.id, interc.radius);
                match data.self_state.connect_state {
                    Unselected => {
                        data.self_state.connect_state = First(x);
                        data.meshrenders
                            .get_mut(data.self_state.show_connect)
                            .unwrap()
                            .hide = false;
                    }
                    First(y) => {
                        let interc2 = data.intersections.get(y).unwrap();
                        if y != x {
                            if !data.map.is_neigh(interc.id, interc2.id) {
                                data.map
                                    .connect(interc.id, interc2.id, &data.self_state.pattern);
                            } else {
                                data.map.disconnect(interc.id, interc2.id);
                            }
                            data.self_state.deactive_connect(&mut data.meshrenders);
                        }
                    }
                    _ => (),
                }
            } else {
                data.self_state.deactive_connect(&mut data.meshrenders);
            }
        }

        if let First(x) = data.self_state.connect_state {
            let trans = data.transforms.get(x).unwrap().clone();
            data.transforms
                .get_mut(data.self_state.show_connect)
                .unwrap()
                .set_position(trans.position());
            if let Some(MeshRenderEnum::Line(x)) = data
                .meshrenders
                .get_mut(data.self_state.show_connect)
                .and_then(|x| x.orders.get_mut(0))
            {
                x.offset = data.mouseinfo.unprojected - trans.position();
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        let state = RoadGraphSynchronizeState::new(world);
        world.insert(state);
    }
}

impl RoadGraphSynchronizeState {
    fn deactive_connect(&mut self, meshrenders: &mut WriteStorage<MeshRender>) {
        self.connect_state = Inactive;
        meshrenders.get_mut(self.show_connect).unwrap().hide = true;
    }
}
