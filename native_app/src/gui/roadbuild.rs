use crate::gui::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use geom::Spline;
use geom::Vec2;
use map_model::{LanePatternBuilder, Map, MapProject, ProjectKind};

const MAX_TURN_ANGLE: f32 = 30.0 * std::f32::consts::PI / 180.0;

#[derive(Copy, Clone, Debug)]
pub enum BuildState {
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
    pub build_state: BuildState,
    pub pattern_builder: LanePatternBuilder,
    pub snap_to_grid: bool,
}

use crate::uiworld::UiWorld;
use common::{AudioKind, Z_TOOL};
use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use egregoria::Egregoria;
use BuildState::{Hover, Interpolation, Start};
use ProjectKind::{Building, Ground, Inter, Road};

pub fn roadbuild(goria: &Egregoria, uiworld: &mut UiWorld) {
    let state = &mut *uiworld.write::<RoadBuildResource>();
    let immdraw = &mut *uiworld.write::<ImmediateDraw>();
    let immsound = &mut *uiworld.write::<ImmediateSound>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let tool = uiworld.read::<Tool>();
    let map = &*goria.read::<Map>();
    let commands: &mut WorldCommands = &mut *uiworld.commands();

    if !matches!(*tool, Tool::RoadbuildStraight | Tool::RoadbuildCurved) {
        state.build_state = BuildState::Hover;
        return;
    }

    let mousepos = if state.snap_to_grid {
        mouseinfo.unprojected.snap(30.0, 30.0)
    } else {
        mouseinfo.unprojected
    };

    for command in uiworld.received_commands().iter() {
        if matches!(
            *uiworld.read::<Tool>(),
            Tool::RoadbuildCurved | Tool::RoadbuildStraight
        ) {
            if let WorldCommand::MapMakeConnection(_, to, _, _) = command {
                let proj = map.project(to.pos, 0.0);
                if matches!(proj.kind, ProjectKind::Inter(_)) {
                    state.build_state = BuildState::Start(proj);
                }
            }
        }
    }

    if mouseinfo.just_pressed.contains(&MouseButton::Right) {
        state.build_state = BuildState::Hover;
    }

    let mut cur_proj = map.project(mousepos, 0.0);
    if matches!(cur_proj.kind, ProjectKind::Lot(_)) {
        cur_proj.kind = ProjectKind::Ground;
    }

    let patwidth = state.pattern_builder.width();

    if let ProjectKind::Road(r_id) = cur_proj.kind {
        let r = &map.roads()[r_id];
        if r.points
            .first()
            .is_close(cur_proj.pos, r.interface_from(r.src) + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: ProjectKind::Inter(r.src),
                pos: r.points.first(),
            };
        } else if r
            .points
            .last()
            .is_close(cur_proj.pos, r.interface_from(r.dst) + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: ProjectKind::Inter(r.dst),
                pos: r.points.last(),
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
                commands.map_make_connection(
                    selected_proj,
                    cur_proj,
                    None,
                    state.pattern_builder.build(),
                );

                state.build_state = Hover;
            }
            (Interpolation(interpoint, selected_proj), _, _) => {
                // Interpolated connection to something
                immsound.play("road_lay", AudioKind::Ui);
                commands.map_make_connection(
                    selected_proj,
                    cur_proj,
                    Some(interpoint),
                    state.pattern_builder.build(),
                );

                state.build_state = Hover;
            }
            _ => {}
        }
    }
}

fn check_angle(map: &Map, from: MapProject, to: Vec2) -> bool {
    match from.kind {
        Inter(i) => {
            let inter = &map.intersections()[i];
            let dir = (to - inter.pos).normalize();
            for &road in &inter.roads {
                let road = &map.roads()[road];
                let v = road.dir_from(i);
                if v.angle(dir).abs() < MAX_TURN_ANGLE {
                    return false;
                }
            }
            true
        }
        Road(r) => {
            let r = &map.roads()[r]; // fixme dont crash
            let (proj, _, rdir1) = r.points().project_segment_dir(from.pos);
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
            common::config().gui_primary
        } else {
            common::config().gui_danger
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
