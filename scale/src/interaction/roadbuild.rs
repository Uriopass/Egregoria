use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::interaction::{Tool, Z_TOOL};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{AbsoluteLineRender, CircleRender, MeshRender};
use crate::rendering::Color;
use geom::splines::Spline;
use geom::Vec2;
use map_model::{
    IntersectionID, LanePattern, LanePatternBuilder, Map, MapProject, ProjectKind, RoadSegmentKind,
};
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

pub struct RoadBuildResource {
    build_state: BuildState,

    pub project_entity: Entity,
    pub pattern_builder: LanePatternBuilder,
}

impl<'a> System<'a> for RoadBuildSystem {
    type SystemData = RoadBuildData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state = &mut data.self_r;

        let mr = data.meshrender.get_mut(state.project_entity).unwrap(); // Unwrap ok: mr defined in new

        if !matches!(*data.tool, Tool::Roadbuild) {
            mr.hide = true;
            state.build_state = BuildState::Hover;
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
            cur_proj.pos,
            state.pattern_builder.width(),
        );

        if data.mouseinfo.just_pressed.contains(&MouseButton::Left) {
            use BuildState::*;
            use ProjectKind::*;

            match (state.build_state, cur_proj.kind) {
                (Hover, _) => {
                    // Hover selection
                    state.build_state = BuildState::Start(cur_proj);
                }
                (Start(v), Ground) => {
                    // Set interpolation point
                    state.build_state = BuildState::Interpolation(data.mouseinfo.unprojected, v);
                }
                (Start(selected_proj), _) if compatible(map, cur_proj.kind, selected_proj.kind) => {
                    // Straight connection to something
                    let selected_after = make_connection(
                        map,
                        selected_proj,
                        cur_proj,
                        None,
                        &state.pattern_builder.build(),
                    );

                    let hover = MapProject {
                        pos: data.map.intersections()[selected_after].pos,
                        kind: ProjectKind::Inter(selected_after),
                    };

                    state.build_state = BuildState::Start(hover);
                }
                (Interpolation(interpoint, selected_proj), _)
                    if compatible(map, cur_proj.kind, selected_proj.kind) =>
                {
                    // Interpolated connection to something
                    let selected_after = make_connection(
                        map,
                        selected_proj,
                        cur_proj,
                        Some(interpoint),
                        &state.pattern_builder.build(),
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

fn make_connection(
    map: &mut Map,
    from: MapProject,
    to: MapProject,
    interpoint: Option<Vec2>,
    pattern: &LanePattern,
) -> IntersectionID {
    use ProjectKind::*;

    let connection_segment = match interpoint {
        Some(x) => RoadSegmentKind::from_elbow(from.pos, to.pos, x),
        None => RoadSegmentKind::Straight,
    };

    let mut mk_inter = |proj: MapProject| match proj.kind {
        Ground => map.add_intersection(proj.pos),
        Inter(id) => id,
        Road(id) => map.split_road(id, proj.pos),
    };

    let from = mk_inter(from);
    let to = mk_inter(to);

    map.connect(from, to, pattern, connection_segment);
    to
}

fn compatible(map: &Map, x: ProjectKind, y: ProjectKind) -> bool {
    use ProjectKind::*;
    match (x, y) {
        (Ground, _) | (_, Ground) => true,
        (Road(id), Road(id2)) => id != id2,
        (Inter(id), Inter(id2)) => id != id2,
        (Inter(id_inter), Road(id_road)) | (Road(id_road), Inter(id_inter)) => {
            let r = &map.roads()[id_road];
            r.src != id_inter && r.dst != id_inter
        }
    }
}

impl RoadBuildResource {
    pub fn update_drawing(&self, mr: &mut WriteStorage<MeshRender>, proj_pos: Vec2, patwidth: f32) {
        let mr = mr.get_mut(self.project_entity).unwrap(); // Unwrap ok: Defined in new
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
                let mut points = sp.smart_points(1.0, 0.0, 1.0).peekable();
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
