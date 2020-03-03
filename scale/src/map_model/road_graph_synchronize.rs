use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{MovedEvent, SelectedEntity};
use crate::map_model::road_graph_synchronize::ConnectState::{First, Inactive, Unselected};
use crate::map_model::{make_inter_entity, IntersectionComponent, LanePattern, Map};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{LineRender, MeshRender, MeshRenderEnum};
use crate::rendering::{Color, BLUE};
use specs::prelude::*;
use specs::shred::{DynamicSystemData, PanicHandler};
use specs::shrev::{EventChannel, ReaderId};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConnectState {
    Inactive,
    Unselected,
    First(Entity),
}

impl ConnectState {
    pub fn is_first(&self) -> bool {
        match self {
            First(_) => true,
            _ => false,
        }
    }
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

        let color = Color { a: 0.5, ..BLUE };
        let e = world
            .create_entity()
            .with(Transform::new([0.0, 0.0]))
            .with(MeshRender::simple(
                LineRender {
                    offset: [0.0, 0.0].into(),
                    color,
                    thickness: 4.0,
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
                data.map.connect(x.id, id, &data.self_state.pattern);
            }
            let e = make_inter_entity(
                &data.map.intersections()[id],
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

        if data.kbinfo.just_pressed.contains(&KeyCode::Escape) {
            data.self_state.deactive_connect(&mut data.meshrenders);
        }

        if let Some(x) = data.selected.0 {
            if let Some(interc) = data.intersections.get(x) {
                data.map.set_intersection_radius(interc.id, interc.radius);
                data.map
                    .set_intersection_turn_policy(interc.id, interc.policy);

                match data.self_state.connect_state {
                    Unselected | Inactive => {
                        data.self_state.connect_state = First(x);
                        data.meshrenders
                            .get_mut(data.self_state.show_connect)
                            .unwrap()
                            .hide = false;
                    }
                    First(y) => {
                        let interc2 = data.intersections.get(y).unwrap();
                        if y != x {
                            let road = data.map.find_road(interc.id, interc2.id);

                            match road {
                                None => {
                                    data.map.connect(
                                        interc2.id,
                                        interc.id,
                                        &data.self_state.pattern,
                                    );
                                }
                                Some(_) => {
                                    let old_road = data.map.disconnect(interc.id, interc2.id);
                                    if data.self_state.pattern != old_road.unwrap().pattern {
                                        data.map.connect(
                                            interc2.id,
                                            interc.id,
                                            &data.self_state.pattern,
                                        );
                                    }
                                }
                            }

                            data.self_state.deactive_connect(&mut data.meshrenders);
                        }
                    }
                }
            } else {
                data.self_state.deactive_connect(&mut data.meshrenders);
            }
        } else if let First(x) = data.self_state.connect_state {
            data.self_state.deactive_connect(&mut data.meshrenders);

            let id = data.map.add_intersection(data.mouseinfo.unprojected);
            let intersections = &data.intersections;
            let lol = intersections.get(x).unwrap();
            data.map.connect(lol.id, id, &data.self_state.pattern);
            let e = make_inter_entity(
                &data.map.intersections()[id],
                data.mouseinfo.unprojected,
                &data.lazy,
                &data.entities,
            );
            *data.selected = SelectedEntity(Some(e));
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
