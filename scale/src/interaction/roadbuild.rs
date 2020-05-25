use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::interaction::{InspectedEntity, Movable, MovedEvent, Selectable, Tool};
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

pub struct RoadBuildSystem;

impl RoadBuildState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        world.setup::<RoadBuildData>();

        Self {
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
            reader,
        }
    }
}

#[derive(SystemData)]
pub struct RoadBuildData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    tool: Read<'a, Tool>,
    self_state: Write<'a, RoadBuildState, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    inspected: Write<'a, InspectedEntity>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    transforms: WriteStorage<'a, Transform>,
    meshrender: WriteStorage<'a, MeshRender>,
}

pub struct RoadBuildState {
    selected: Option<(Entity, MapProject)>,

    pub project_entity: Entity,

    pub pattern_builder: LanePatternBuilder,
    reader: ReaderId<MovedEvent>,
}

impl<'a> System<'a> for RoadBuildSystem {
    type SystemData = RoadBuildData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state = &mut data.self_state;

        let mr = data.meshrender.get_mut(state.project_entity).unwrap();

        if !matches!(*data.tool, Tool::Roadbuild | Tool::Bulldozer) {
            data.moved.read(&mut state.reader).for_each(drop);
            state.set_selected(&data.entities, None);
            mr.hide = true;
            return;
        }

        if matches!(*data.tool, Tool::Bulldozer) {
            state.set_selected(&data.entities, None);
        }

        mr.hide = false;
        mr.orders[0].as_circle_mut().color = match *data.tool {
            Tool::Bulldozer => Color::RED,
            _ => Color::BLUE,
        };

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
            if data.inspected.dirty {
                state.on_select_dirty(&data.intersections, e, &mut data.map);
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

        if data.mouseinfo.buttons.contains(&MouseButton::Left)
            && matches!(*data.tool, Tool::Bulldozer)
        {
            match cur_proj.map(|x| x.kind) {
                Some(ProjectKind::Inter(id)) => data.map.remove_intersection(id),
                Some(ProjectKind::Road(id)) => {
                    let r = &data.map.roads()[id];
                    let src = r.src;
                    let dst = r.dst;

                    data.map.remove_road(id);

                    if data.map.intersections()[src].roads.is_empty() {
                        data.map.remove_intersection(src);
                    }
                    if data.map.intersections()[dst].roads.is_empty() {
                        data.map.remove_intersection(dst);
                    }
                }
                _ => {}
            }
            return;
        }

        if data.mouseinfo.just_pressed.contains(&MouseButton::Left) {
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

                    data.inspected.dirty = false;
                    data.inspected.e = Some(ent);

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
                        data.inspected.dirty = false;
                        data.inspected.e = Some(ent);
                    }

                    state.set_selected(&data.entities, Some((ent, hover)));
                }
                (Some((_, selected_proj)), Some(hover))
                    if compatible(map, hover.kind, selected_proj.kind) =>
                {
                    // Connection between different things
                    let selected_after =
                        make_connection(map, selected_proj, hover, state.pattern_builder.build());

                    let ent = make_selected_entity(
                        &data.entities,
                        &data.lazy,
                        state.project_entity,
                        &map,
                        hover,
                    );

                    data.inspected.dirty = false;
                    data.inspected.e = Some(ent);

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

            map.connect_straight(rx.src, mid_idx, rx.lane_pattern.clone());
            map.connect_straight(mid_idx, rx.dst, rx.lane_pattern);

            map.connect_straight(ry.src, mid_idy, ry.lane_pattern.clone());
            map.connect_straight(mid_idy, ry.dst, ry.lane_pattern);

            map.connect_straight(mid_idx, mid_idy, pattern);

            mid_idy
        }
        (Inter(src), Inter(dst)) => {
            if let [id] = map.intersections()[src].roads.as_slice() {
                let id = *id;
                let r = &map.roads()[id];
                let r_src = r.other_end(src);
                if r.lane_pattern == pattern && r_src != dst {
                    let rev = r.src == src;
                    let mut line = map.remove_road(id).interpolation_points_owned();
                    if rev {
                        line.reverse();
                    }
                    line.push(map.intersections()[dst].pos);
                    map.remove_intersection(src);
                    map.connect(r_src, dst, pattern, line);
                    return dst;
                }
            }
            map.connect_straight(src, dst, pattern);
            dst
        }
        (Inter(id_inter), Road(id_road)) | (Road(id_road), Inter(id_inter)) => {
            let r = map.remove_road(id_road);

            let r_pos = if let Road(_) = from.kind {
                from.pos
            } else {
                to.pos
            };

            let id = map.add_intersection(r_pos);
            map.connect_straight(r.src, id, r.lane_pattern.clone());
            map.connect_straight(id, r.dst, r.lane_pattern);

            let thing = if let Road(_) = to.kind {
                (id, id_inter)
            } else {
                (id_inter, id)
            };

            map.connect_straight(thing.0, thing.1, pattern);

            thing.0
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
                turn_policy: inter.turn_policy,
                light_policy: inter.light_policy,
            })
            .with(Selectable::new(15.0))
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

impl RoadBuildState {
    fn set_selected(&mut self, entities: &EntitiesRes, sel: Option<(Entity, MapProject)>) {
        if let Some((e, _)) = self.selected.take() {
            let _ = entities.delete(e);
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
    }
}
