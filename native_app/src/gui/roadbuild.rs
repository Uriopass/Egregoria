use crate::gui::Tool;
use crate::input::{MouseButton, MouseInfo};
use egregoria::rendering::immediate::{ImmediateDraw, ImmediateSound};
use geom::Color;
use geom::Spline;
use geom::Vec2;
use legion::system;
use map_model::{
    IntersectionID, LanePattern, LanePatternBuilder, Map, MapProject, ProjectKind, RoadSegmentKind,
};

const MAX_TURN_ANGLE: f32 = 30.0 * std::f32::consts::PI / 180.0;

#[derive(Copy, Clone, Debug)]
enum BuildState {
    Hover,
    Start(MapProject),
    Interpolation(Vec2, MapProject),
}

impl Default for BuildState {
    fn default() -> Self {
        BuildState::Hover
    }
}

register_resource_noserialize!(RoadBuildResource);
#[derive(Default)]
pub struct RoadBuildResource {
    build_state: BuildState,
    pub pattern_builder: LanePatternBuilder,
}

use common::{AudioKind, Z_TOOL};
use BuildState::{Hover, Interpolation, Start};
use ProjectKind::{Building, Ground, Inter, Lot, Road};

#[system]
pub fn roadbuild(
    #[resource] state: &mut RoadBuildResource,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] tool: &Tool,
    #[resource] map: &mut Map,
    #[resource] immdraw: &mut ImmediateDraw,
    #[resource] immsound: &mut ImmediateSound,
) {
    if !matches!(*tool, Tool::RoadbuildStraight | Tool::RoadbuildCurved) {
        state.build_state = BuildState::Hover;
        return;
    }

    if mouseinfo.just_pressed.contains(&MouseButton::Right) {
        state.build_state = BuildState::Hover;
    }

    let mut cur_proj = map.project(mouseinfo.unprojected);
    if matches!(cur_proj.kind, ProjectKind::Lot(_)) {
        cur_proj.kind = ProjectKind::Ground;
    }

    let patwidth = state.pattern_builder.width();

    if let ProjectKind::Road(r_id) = cur_proj.kind {
        let r = &map.roads()[r_id];
        if r.src_point
            .is_close(cur_proj.pos, r.src_interface + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: ProjectKind::Inter(r.src),
                pos: r.src_point,
            };
        } else if r
            .dst_point
            .is_close(cur_proj.pos, r.dst_interface + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: ProjectKind::Inter(r.dst),
                pos: r.dst_point,
            };
        }
    }

    let is_valid = match (state.build_state, cur_proj.kind) {
        (Hover, Building(_)) => false,
        (Start(selected_proj), _) => {
            compatible(map, cur_proj.kind, selected_proj.kind)
                && check_angle(map, selected_proj, cur_proj.pos)
                && check_angle(map, cur_proj, selected_proj.pos)
        }
        (Interpolation(interpoint, selected_proj), _) => {
            let sp = Spline {
                from: selected_proj.pos,
                to: cur_proj.pos,
                from_derivative: (interpoint - selected_proj.pos) * std::f32::consts::FRAC_1_SQRT_2,
                to_derivative: (cur_proj.pos - interpoint) * std::f32::consts::FRAC_1_SQRT_2,
            };

            compatible(map, cur_proj.kind, selected_proj.kind)
                && check_angle(map, selected_proj, interpoint)
                && check_angle(map, cur_proj, interpoint)
                && !sp.is_steep(state.pattern_builder.width())
        }
        _ => true,
    };

    state.update_drawing(immdraw, cur_proj.pos, patwidth, is_valid);

    if is_valid && mouseinfo.just_pressed.contains(&MouseButton::Left) {
        log::info!(
            "left clicked with state {:?} and {:?}",
            state.build_state,
            cur_proj.kind
        );

        // FIXME: Use or patterns when stable
        match (state.build_state, cur_proj.kind, *tool) {
            (Hover, Ground, _) | (Hover, Road(_), _) | (Hover, Inter(_), _) => {
                // Hover selection
                state.build_state = Start(cur_proj);
            }
            (Start(v), Ground, Tool::RoadbuildCurved) => {
                // Set interpolation point
                state.build_state = Interpolation(mouseinfo.unprojected, v);
            }
            (Start(selected_proj), _, _) => {
                // Straight connection to something
                immsound.play("road_lay", AudioKind::Ui);
                let selected_after = make_connection(
                    map,
                    selected_proj,
                    cur_proj,
                    None,
                    &state.pattern_builder.build(),
                );

                let hover = MapProject {
                    pos: map.intersections()[selected_after].pos,
                    kind: Inter(selected_after),
                };

                state.build_state = Start(hover);
            }
            (Interpolation(interpoint, selected_proj), _, _) => {
                // Interpolated connection to something
                immsound.play("road_lay", AudioKind::Ui);
                let selected_after = make_connection(
                    map,
                    selected_proj,
                    cur_proj,
                    Some(interpoint),
                    &state.pattern_builder.build(),
                );

                let hover = MapProject {
                    pos: map.intersections()[selected_after].pos,
                    kind: Inter(selected_after),
                };

                state.build_state = Start(hover);
            }
            _ => {}
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
    let connection_segment = match interpoint {
        Some(x) => RoadSegmentKind::from_elbow(from.pos, to.pos, x),
        None => RoadSegmentKind::Straight,
    };

    let mut mk_inter = |proj: MapProject| match proj.kind {
        Ground => map.add_intersection(proj.pos),
        Inter(id) => id,
        Road(id) => map.split_road(id, proj.pos),
        Building(_) | Lot(_) => unreachable!(),
    };

    let from = mk_inter(from);
    let to = mk_inter(to);

    map.connect(from, to, pattern, connection_segment);
    to
}

fn check_angle(map: &Map, from: MapProject, to: Vec2) -> bool {
    match from.kind {
        Inter(i) => {
            let inter = &map.intersections()[i];
            let dir = (to - inter.pos).normalize();
            for &road in &inter.roads {
                let road = &map.roads()[road];
                let v = road.orientation_from(i);
                if v.angle(dir).abs() < MAX_TURN_ANGLE {
                    return false;
                }
            }
            true
        }
        Road(r) => {
            let r = &map.roads()[r];
            let (proj, _, rdir1) = r.generated_points().project_segment_dir(from.pos);
            let rdir2 = -rdir1;
            let dir = (to - proj).normalize();
            if rdir1.angle(dir).abs() < MAX_TURN_ANGLE || rdir2.angle(dir).abs() < MAX_TURN_ANGLE {
                return false;
            }
            true
        }
        _ => true,
    }
}

fn compatible(map: &Map, x: ProjectKind, y: ProjectKind) -> bool {
    match (x, y) {
        (Ground, Ground)
        | (Ground, Road(_))
        | (Ground, Inter(_))
        | (Road(_), Ground)
        | (Inter(_), Ground) => true,
        (Road(id), Road(id2)) => id != id2,
        (Inter(id), Inter(id2)) => id != id2,
        (Inter(id_inter), Road(id_road)) | (Road(id_road), Inter(id_inter)) => {
            let r = &map.roads()[id_road];
            r.src != id_inter && r.dst != id_inter
        }
        _ => false,
    }
}

impl RoadBuildResource {
    pub fn update_drawing(
        &self,
        immdraw: &mut ImmediateDraw,
        proj_pos: Vec2,
        patwidth: f32,
        is_valid: bool,
    ) {
        let col = if is_valid {
            Color {
                r: 0.3,
                g: 0.4,
                b: 1.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.85,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            }
        };

        match self.build_state {
            BuildState::Hover => {
                immdraw
                    .circle(proj_pos, patwidth * 0.5)
                    .color(col)
                    .z(Z_TOOL);
            }
            BuildState::Start(x) => {
                immdraw
                    .circle(proj_pos, patwidth * 0.5)
                    .color(col)
                    .z(Z_TOOL);
                immdraw.circle(x.pos, patwidth * 0.5).color(col).z(Z_TOOL);
                immdraw.line(proj_pos, x.pos, patwidth).color(col).z(Z_TOOL);
            }
            BuildState::Interpolation(p, x) => {
                let sp = Spline {
                    from: x.pos,
                    to: proj_pos,
                    from_derivative: (p - x.pos) * std::f32::consts::FRAC_1_SQRT_2,
                    to_derivative: (proj_pos - p) * std::f32::consts::FRAC_1_SQRT_2,
                };
                let points: Vec<_> = sp.smart_points(1.0, 0.0, 1.0).collect();

                immdraw.polyline(points, patwidth).color(col).z(Z_TOOL);

                immdraw
                    .circle(sp.get(0.0), patwidth * 0.5)
                    .color(col)
                    .z(Z_TOOL);
                immdraw
                    .circle(sp.get(1.0), patwidth * 0.5)
                    .color(col)
                    .z(Z_TOOL);
            }
        }
    }
}
