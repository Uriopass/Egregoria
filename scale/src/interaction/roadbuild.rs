use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::interaction::Tool;
use crate::map_model::{
    IntersectionID, LanePattern, LanePatternBuilder, Map, MapProject, ProjectKind,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::PanicHandler;

pub struct RoadBuildSystem;

impl RoadBuildResource {
    pub fn new(world: &mut World) -> Self {
        world.setup::<RoadBuildData>();

        Self {
            build_state: BuildState::Hover,

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
        }
    }
}

#[derive(SystemData)]
pub struct RoadBuildData<'a> {
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    tool: Read<'a, Tool>,
    self_r: Write<'a, RoadBuildResource, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    meshrender: WriteStorage<'a, MeshRender>,
}

#[derive(Clone, Copy)]
enum BuildState {
    Hover,
    Start(MapProject),
    Interpolation(Vec2, MapProject),
}

impl BuildState {
    #[allow(dead_code)]
    pub fn proj(&self) -> Option<&MapProject> {
        use BuildState::*;
        match self {
            Hover => None,
            Start(x) | Interpolation(_, x) => Some(x),
        }
    }
}

pub struct RoadBuildResource {
    build_state: BuildState,

    pub project_entity: Entity,
    pub pattern_builder: LanePatternBuilder,
}

impl<'a> System<'a> for RoadBuildSystem {
    type SystemData = RoadBuildData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state = &mut data.self_r;

        let mr = data.meshrender.get_mut(state.project_entity).unwrap();

        if !matches!(*data.tool, Tool::Roadbuild | Tool::Bulldozer) {
            mr.hide = true;
            return;
        }

        mr.hide = false;
        mr.orders[0].as_circle_mut().color = match *data.tool {
            Tool::Bulldozer => Color::RED,
            _ => Color::BLUE,
        };

        if data.kbinfo.just_pressed.contains(&KeyCode::Escape) {
            state.build_state = BuildState::Hover;
        }

        let map: &mut Map = &mut data.map;

        let cur_proj = map.project(data.mouseinfo.unprojected);

        // todo: move this to bulldozer system
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

        state.update_drawing(&mut data.meshrender, data.mouseinfo.unprojected);

        if data.mouseinfo.just_pressed.contains(&MouseButton::Left) {
            match (state.build_state, cur_proj) {
                (BuildState::Hover, None) => {
                    // Intersection creation on empty ground
                    let id = map.add_intersection(data.mouseinfo.unprojected);

                    let hover = MapProject {
                        pos: data.mouseinfo.unprojected,
                        kind: ProjectKind::Inter(id),
                    };

                    state.build_state = BuildState::Start(hover);
                }
                (BuildState::Start(v), None) => {
                    // Set interpolation point
                    state.build_state = BuildState::Interpolation(data.mouseinfo.unprojected, v);
                }
                (BuildState::Interpolation(interpoint, selected_proj), None) => {
                    // Interpolated connection to empty
                    let id = map.add_intersection(data.mouseinfo.unprojected);

                    let selected_after = make_connection(
                        map,
                        selected_proj,
                        MapProject {
                            pos: data.mouseinfo.unprojected,
                            kind: ProjectKind::Inter(id),
                        },
                        Some(interpoint),
                        state.pattern_builder.build(),
                    );

                    let hover = MapProject {
                        pos: data.map.intersections()[selected_after].pos,
                        kind: ProjectKind::Inter(selected_after),
                    };

                    state.build_state = BuildState::Start(hover);
                }
                (BuildState::Hover, Some(hover)) => {
                    // Hover selection
                    state.build_state = BuildState::Start(hover);
                }
                (BuildState::Start(selected_proj), Some(hover))
                    if compatible(map, hover.kind, selected_proj.kind) =>
                {
                    // Straight connection to something
                    let selected_after = make_connection(
                        map,
                        selected_proj,
                        hover,
                        None,
                        state.pattern_builder.build(),
                    );

                    let hover = MapProject {
                        pos: data.map.intersections()[selected_after].barycenter,
                        kind: ProjectKind::Inter(selected_after),
                    };

                    state.build_state = BuildState::Start(hover);
                }
                (BuildState::Interpolation(interpoint, selected_proj), Some(hover))
                    if compatible(map, hover.kind, selected_proj.kind) =>
                {
                    // Interpolated connection to something
                    let selected_after = make_connection(
                        map,
                        selected_proj,
                        hover,
                        Some(interpoint),
                        state.pattern_builder.build(),
                    );

                    let hover = MapProject {
                        pos: data.map.intersections()[selected_after].barycenter,
                        kind: ProjectKind::Inter(selected_after),
                    };

                    state.build_state = BuildState::Start(hover);
                }
                _ => {}
            }
        }
    }
}

impl RoadBuildResource {
    pub fn update_drawing(&self, mr: &mut WriteStorage<MeshRender>, mouse: Vec2) {
        let mr = mr.get_mut(self.project_entity).unwrap();
        mr.orders.clear();

        match self.build_state {
            BuildState::Hover => {
                mr.add(CircleRender {
                    offset: mouse,
                    radius: 1.0,
                    color: Color::BLUE,
                });
            }
            BuildState::Start(x) => {
                mr.add(CircleRender {
                    offset: mouse,
                    radius: 1.0,
                    color: Color::BLUE,
                })
                .add(CircleRender {
                    offset: x.pos,
                    radius: 1.0,
                    color: Color::BLUE,
                });
            }
            BuildState::Interpolation(p, x) => {
                mr.add(CircleRender {
                    offset: mouse,
                    radius: 1.0,
                    color: Color::BLUE,
                })
                .add(CircleRender {
                    offset: x.pos,
                    radius: 1.0,
                    color: Color::BLUE,
                })
                .add(CircleRender {
                    offset: p,
                    radius: 1.0,
                    color: Color::GREEN,
                });
            }
        }
    }
}

fn make_connection(
    map: &mut Map,
    from: MapProject,
    to: MapProject,
    interpoint: Option<Vec2>,
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
            // todo: simplify this
            if let Some(interpoint) = interpoint {
                map.connect(
                    src,
                    dst,
                    pattern,
                    PolyLine::new(vec![
                        map.intersections()[src].pos,
                        interpoint,
                        map.intersections()[dst].pos,
                    ]),
                );
            } else {
                map.connect_straight(src, dst, pattern);
            }
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
