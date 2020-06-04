use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::geometry::splines::Spline;
use crate::geometry::Vec2;
use crate::interaction::{Tool, Z_TOOL};
use crate::map_model::{
    IntersectionID, LanePattern, LanePatternBuilder, Map, MapProject, ProjectKind, RoadSegmentKind,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{AbsoluteLineRender, CircleRender, MeshRender};
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
                    Z_TOOL,
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

        if !matches!(*data.tool, Tool::Roadbuild) {
            mr.hide = true;
            return;
        }
        mr.hide = false;

        if data.kbinfo.just_pressed.contains(&KeyCode::Escape) {
            state.build_state = BuildState::Hover;
        }

        let map: &mut Map = &mut data.map;

        let cur_proj = map.project(data.mouseinfo.unprojected);

        state.update_drawing(
            &mut data.meshrender,
            cur_proj
                .map(|x| x.pos)
                .unwrap_or(data.mouseinfo.unprojected),
            state.pattern_builder.width(),
        );

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
                        pos: data.map.intersections()[selected_after].pos,
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
                        pos: data.map.intersections()[selected_after].pos,
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
    pub fn update_drawing(&self, mr: &mut WriteStorage<MeshRender>, proj_pos: Vec2, patwidth: f32) {
        let mr = mr.get_mut(self.project_entity).unwrap();
        mr.orders.clear();

        let transparent_blue = Color {
            r: 0.3,
            g: 0.3,
            b: 1.0,
            a: 1.0,
        };

        match self.build_state {
            BuildState::Hover => {
                mr.add(CircleRender {
                    offset: proj_pos,
                    radius: 2.0,
                    color: transparent_blue,
                });
            }
            BuildState::Start(x) => {
                mr.add(CircleRender {
                    offset: proj_pos,
                    radius: patwidth * 0.5,
                    color: transparent_blue,
                })
                .add(CircleRender {
                    offset: x.pos,
                    radius: patwidth * 0.5,
                    color: transparent_blue,
                })
                .add(AbsoluteLineRender {
                    src: proj_pos,
                    dst: x.pos,
                    thickness: patwidth,
                    color: transparent_blue,
                });
            }
            BuildState::Interpolation(p, x) => {
                let sp = Spline {
                    from: x.pos,
                    to: proj_pos,
                    from_derivative: (p - x.pos) * std::f32::consts::FRAC_1_SQRT_2,
                    to_derivative: (proj_pos - p) * std::f32::consts::FRAC_1_SQRT_2,
                };
                let mut points = sp.smart_points(1.0).peekable();
                while let Some(v) = points.next() {
                    mr.add(CircleRender {
                        offset: v,
                        radius: patwidth * 0.5,
                        color: transparent_blue,
                    });

                    if let Some(peek) = points.peek() {
                        mr.add(AbsoluteLineRender {
                            src: v,
                            dst: *peek,
                            thickness: patwidth,
                            color: transparent_blue,
                        });
                    }
                }
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
            let kind = match interpoint {
                Some(x) => RoadSegmentKind::Curved(x),
                None => RoadSegmentKind::Straight,
            };

            map.connect(src, dst, pattern, kind);

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
