use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::interaction::{Movable, MovedEvent, Selectable, SelectedEntity};
use crate::map_model::{
    IntersectionComponent, IntersectionID, LanePattern, LanePatternBuilder, Map, MapProject,
    ProjectKind,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, LineToRender, MeshRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::PanicHandler;
use specs::shrev::{EventChannel, ReaderId};
use specs::world::EntitiesRes;

pub struct MapUISystem;

impl MapUIState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        world.setup::<MapUIData>();

        Self {
            enabled: true,
            selected: None,

            project_entity: world
                .create_entity()
                .with(Transform::zero())
                .with(MeshRender::simple(
                    CircleRender {
                        radius: 2.0,
                        color: Color::BLUE,
                        ..Default::default()
                    },
                    0.9,
                ))
                .build(),

            pattern_builder: LanePatternBuilder::new(),
            map_render_dirty: true,
            reader,
        }
    }
}

#[derive(SystemData)]
pub struct MapUIData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    self_state: Write<'a, MapUIState, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    transforms: WriteStorage<'a, Transform>,
    meshrender: WriteStorage<'a, MeshRender>,
}

pub struct MapUIState {
    pub enabled: bool,

    selected: Option<(Entity, MapProject)>,

    pub project_entity: Entity,

    pub pattern_builder: LanePatternBuilder,
    pub map_render_dirty: bool,

    reader: ReaderId<MovedEvent>,
}

impl<'a> System<'a> for MapUISystem {
    type SystemData = MapUIData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state = &mut data.self_state;

        data.meshrender.get_mut(state.project_entity).unwrap().hide = !state.enabled;

        if !state.enabled {
            data.moved.read(&mut state.reader).for_each(drop);
            state.set_selected(&data.entities, None);
            return;
        }

        for event in data.moved.read(&mut state.reader) {
            if let Some((
                e,
                MapProject {
                    kind: ProjectKind::Inter(id),
                    ..
                },
            )) = state.selected
            {
                if e == event.entity {
                    data.map.update_intersection(id, |x| {
                        x.pos += event.delta_pos;
                    });
                    state.map_render_dirty = true;
                }
            }
        }

        if data.kbinfo.just_pressed.contains(&KeyCode::Escape) {
            state.set_selected(&data.entities, None);
        }

        if let Some((
            e,
            MapProject {
                kind: ProjectKind::Inter(_),
                ..
            },
        )) = state.selected
        {
            if data.selected.dirty {
                state.on_select_dirty(&data.intersections, e, &mut data.map);
            }
        }

        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some((_, p)) = state.selected {
                match p.kind {
                    ProjectKind::Inter(id) => {
                        data.map.remove_intersection(id);
                    }
                    ProjectKind::Road(id) => {
                        data.map.remove_road(id);
                    }
                }
                state.map_render_dirty = true;
                state.set_selected(&data.entities, None);
            }
        }

        let map: &mut Map = &mut data.map;

        let cur_proj = map.project(data.mouseinfo.unprojected);

        let pos_project_ent = data.transforms.get_mut(state.project_entity).unwrap();

        pos_project_ent.set_position(match cur_proj {
            Some(v)
                if state
                    .selected
                    .map(|(_, x)| compatible(map, x.kind, v.kind))
                    .unwrap_or(true) =>
            {
                v.pos
            }
            _ => data.mouseinfo.unprojected,
        });

        let left_click = data.mouseinfo.just_pressed.contains(&MouseButton::Left);

        if left_click {
            match (state.selected, cur_proj) {
                (sel, None) => {
                    // Intersection creation on empty ground
                    let id = map.add_intersection(data.mouseinfo.unprojected);

                    let hover = MapProject {
                        pos: data.mouseinfo.unprojected,
                        kind: ProjectKind::Inter(id),
                    };

                    let ent = make_selected_entity(
                        &data.entities,
                        &data.lazy,
                        state.project_entity,
                        &map,
                        hover,
                    );

                    state.map_render_dirty = true;
                    data.selected.dirty = false;
                    data.selected.e = Some(ent);

                    state.set_selected(&data.entities, Some((ent, hover)));

                    if let Some((_, selected_proj)) = sel {
                        // Connect if selected
                        make_connection(map, selected_proj, hover, state.pattern_builder.build());
                    }
                }
                (None, Some(hover)) => {
                    // Hover selection
                    let ent = make_selected_entity(
                        &data.entities,
                        &data.lazy,
                        state.project_entity,
                        &map,
                        hover,
                    );

                    if let ProjectKind::Inter(_) = hover.kind {
                        data.selected.dirty = false;
                        data.selected.e = Some(ent);
                    }

                    state.set_selected(&data.entities, Some((ent, hover)));
                }
                (Some((_, selected_proj)), Some(hover))
                    if compatible(map, hover.kind, selected_proj.kind) =>
                {
                    // Connection between different things
                    println!("Connection between {:?} and {:?}", selected_proj, hover);
                    let selected_after =
                        make_connection(map, selected_proj, hover, state.pattern_builder.build());

                    state.map_render_dirty = true;

                    let ent = make_selected_entity(
                        &data.entities,
                        &data.lazy,
                        state.project_entity,
                        &map,
                        hover,
                    );

                    data.selected.dirty = false;
                    data.selected.e = Some(ent);

                    let hover = MapProject {
                        pos: data.map.intersections()[selected_after].pos,
                        kind: ProjectKind::Inter(selected_after),
                    };

                    state.set_selected(&data.entities, Some((ent, hover)));
                }
                _ => {}
            }
        }
    }
}

fn make_connection(
    map: &mut Map,
    from: MapProject,
    to: MapProject,
    pattern: LanePattern,
) -> IntersectionID {
    use ProjectKind::*;

    match (from.kind, to.kind) {
        (Road(idx), Road(idy)) => {
            let rx = map.remove_road(idx);
            let ry = map.remove_road(idy);

            let mid_idx = map.add_intersection(from.pos);
            let mid_idy = map.add_intersection(to.pos);

            map.connect(rx.src, mid_idx, rx.lane_pattern.clone());
            map.connect(mid_idx, rx.dst, rx.lane_pattern);

            map.connect(ry.src, mid_idy, ry.lane_pattern.clone());
            map.connect(mid_idy, ry.dst, ry.lane_pattern);

            map.connect(mid_idx, mid_idy, pattern);

            mid_idy
        }
        (Inter(id), Inter(id2)) => {
            if let Some(id) = map.find_road(id, id2) {
                let road = map.roads().get(id).unwrap();
                if road.lane_pattern == pattern {
                    return id2;
                }
                map.remove_road(id);
            }
            map.connect(id, id2, pattern);
            id2
        }
        (Inter(id_inter), Road(id_road)) | (Road(id_road), Inter(id_inter)) => {
            let r = map.remove_road(id_road);

            let r_pos = if let Road(_) = from.kind {
                from.pos
            } else {
                to.pos
            };

            let id = map.add_intersection(r_pos);

            map.connect(r.src, id, r.lane_pattern.clone());
            map.connect(id, r.dst, r.lane_pattern);

            map.connect(id_inter, id, pattern);

            if let Road(_) = to.kind {
                id
            } else {
                id_inter
            }
        }
    }
}

fn make_selected_entity(
    entities: &EntitiesRes,
    lazy: &LazyUpdate,
    project_entity: Entity,
    map: &Map,
    hover: MapProject,
) -> Entity {
    let mut ent = lazy
        .create_entity(entities)
        .with(
            MeshRender::empty(0.9)
                .add(CircleRender {
                    offset: vec2!(0.0, 0.0),
                    radius: 2.0,
                    color: Color::BLUE,
                })
                .add(LineToRender {
                    to: project_entity,
                    color: Color::BLUE,
                    thickness: 4.0,
                })
                .build(),
        )
        .with(Transform::new(hover.pos));

    if let ProjectKind::Inter(inter_id) = hover.kind {
        let inter = &map.intersections()[inter_id];
        ent = ent
            .with(IntersectionComponent {
                id: inter_id,
                radius: inter.interface_radius,
                turn_policy: inter.turn_policy,
                light_policy: inter.light_policy,
            })
            .with(Selectable::new(inter.interface_radius))
            .with(Movable);
    }

    ent.build()
}

fn compatible(map: &Map, x: ProjectKind, y: ProjectKind) -> bool {
    use ProjectKind::*;
    match (x, y) {
        (Road(id), Road(id2)) => id != id2,
        (Inter(id), Inter(id2)) => id != id2,
        (Inter(id_inter), Road(id_road)) | (Road(id_road), Inter(id_inter)) => {
            let r = &map.roads()[id_road];
            r.src != id_inter && r.dst != id_inter
        }
    }
}

impl MapUIState {
    fn set_selected(&mut self, entities: &EntitiesRes, sel: Option<(Entity, MapProject)>) {
        if let Some((e, _)) = self.selected.take() {
            entities.delete(e).unwrap();
        }
        self.selected = sel;
    }

    fn on_select_dirty(
        &mut self,
        intersections: &WriteStorage<IntersectionComponent>,
        selected: Entity,
        map: &mut Map,
    ) {
        let selected_interc = intersections.get(selected).unwrap();
        map.update_intersection(selected_interc.id, |inter| {
            inter.turn_policy = selected_interc.turn_policy;
            inter.light_policy = selected_interc.light_policy;
        });

        self.map_render_dirty = true;
    }
}
