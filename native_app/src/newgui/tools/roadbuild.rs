use engine::AudioKind;
use geom::{BoldLine, BoldSpline, Camera, PolyLine, ShapeEnum, Spline};
use geom::{PolyLine3, Vec2, Vec3};
use simulation::map::{
    LanePatternBuilder, Map, MapProject, ProjectFilter, ProjectKind, PylonPosition, RoadSegmentKind,
};
use simulation::world_command::{WorldCommand, WorldCommands};
use simulation::Simulation;
use BuildState::{Hover, Interpolation, Start, StartInterp};
use ProjectKind::{Building, Ground, Inter, Road};

use crate::inputmap::{InputAction, InputMap};
use crate::newgui::{PotentialCommands, Tool};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::UiWorld;

#[derive(Copy, Clone, Debug, Default)]
pub enum BuildState {
    #[default]
    Hover,
    Start(MapProject),
    StartInterp(MapProject),
    Interpolation(Vec2, MapProject),
}

/// Road building tool
/// Allows to build roads and intersections
pub fn roadbuild(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::roadbuild");
    let state = &mut *uiworld.write::<RoadBuildResource>();
    let immdraw = &mut *uiworld.write::<ImmediateDraw>();
    let immsound = &mut *uiworld.write::<ImmediateSound>();
    let potential_command = &mut *uiworld.write::<PotentialCommands>();
    let mut inp = uiworld.write::<InputMap>();
    let tool = *uiworld.read::<Tool>();
    let map = &*sim.map();
    let commands: &mut WorldCommands = &mut uiworld.commands();
    let cam = &*uiworld.read::<Camera>();

    if !tool.is_roadbuild() {
        state.build_state = Hover;
        state.height_offset = 0.0;
        return;
    }

    let nosnapping = inp.act.contains(&InputAction::NoSnapping);

    // Prepare mousepos depending on snap to grid
    let unproj = unwrap_ret!(inp.unprojected);
    let grid_size = 20.0;
    let mousepos = if state.snap_to_grid {
        let v = unproj.xy().snap(grid_size, grid_size);
        v.z(unwrap_ret!(map.environment.height(v)) + state.height_offset)
    } else {
        unproj.up(state.height_offset)
    };

    let log_camheight = cam.eye().z.log10();
    /*
    let cutoff = 3.3;

    if state.snap_to_grid && log_camheight < cutoff {
        let alpha = 1.0 - log_camheight / cutoff;
        let col = simulation::colors().gui_primary.a(alpha);
        let screen = AABB::new(unproj.xy(), unproj.xy()).expand(300.0);
        let startx = (screen.ll.x / grid_size).ceil() * grid_size;
        let starty = (screen.ll.y / grid_size).ceil() * grid_size;

        let height = |p| map.terrain.height(p);
        for x in 0..(screen.w() / grid_size) as i32 {
            let x = startx + x as f32 * grid_size;
            for y in 0..(screen.h() / grid_size) as i32 {
                let y = starty + y as f32 * grid_size;
                let p = vec2(x, y);
                let p3 = p.z(unwrap_cont!(height(p)) + 0.1);
                let px = p + Vec2::x(grid_size);
                let py = p + Vec2::y(grid_size);

                immdraw
                    .line(p3, px.z(unwrap_cont!(height(px)) + 0.1), 0.3)
                    .color(col);
                immdraw
                    .line(p3, py.z(unwrap_cont!(height(py)) + 0.1), 0.3)
                    .color(col);
            }
        }
    }*/

    // If a road was placed recently (as it is async with networking) prepare the next road
    for command in uiworld.received_commands().iter() {
        if let WorldCommand::MapMakeConnection { to, .. } = command {
            if let proj @ MapProject { kind: Inter(_), .. } =
                map.project(to.pos, 0.0, ProjectFilter::ALL)
            {
                if matches!(tool, Tool::RoadbuildCurved) {
                    state.build_state = StartInterp(proj);
                } else {
                    state.build_state = Start(proj);
                }
            }
        }
    }

    if inp.just_act.contains(&InputAction::Close) && !matches!(state.build_state, Hover) {
        inp.just_act.remove(&InputAction::Close);
        state.build_state = Hover;
    }

    if inp.just_act.contains(&InputAction::UpElevation) {
        state.height_offset += 5.0;
        state.height_offset = state.height_offset.min(100.0);
    }

    if inp.just_act.contains(&InputAction::DownElevation) {
        state.height_offset -= 5.0;
        state.height_offset = state.height_offset.max(0.0);
    }

    let mut cur_proj = map.project(
        mousepos,
        (log_camheight * 5.0).clamp(1.0, 10.0),
        ProjectFilter::INTER | ProjectFilter::ROAD,
    );

    let patwidth = state.pattern_builder.width();

    if let Road(r_id) = cur_proj.kind {
        let r = &map.roads()[r_id];
        if r.points
            .first()
            .is_close(cur_proj.pos, r.interface_from(r.src) + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: Inter(r.src),
                pos: r.points.first(),
            };
        } else if r
            .points
            .last()
            .is_close(cur_proj.pos, r.interface_from(r.dst) + patwidth * 0.5)
        {
            cur_proj = MapProject {
                kind: Inter(r.dst),
                pos: r.points.last(),
            };
        }
    }

    if nosnapping {
        cur_proj = MapProject {
            pos: mousepos,
            kind: Ground,
        }
    }

    let is_rail = state.pattern_builder.rail;

    let mut is_valid = match (state.build_state, cur_proj.kind) {
        (Hover, Building(_)) => false,
        (Start(selected_proj) | StartInterp(selected_proj), _) => {
            let sp = BoldLine::new(
                PolyLine::new(vec![selected_proj.pos.xy(), cur_proj.pos.xy()]),
                patwidth * 0.5,
            );

            compatible(map, cur_proj, selected_proj)
                && check_angle(map, selected_proj, cur_proj.pos.xy(), is_rail)
                && check_angle(map, cur_proj, selected_proj.pos.xy(), is_rail)
                && !check_intersect(
                    map,
                    &ShapeEnum::BoldLine(sp),
                    (selected_proj.pos.z + cur_proj.pos.z) / 2.0,
                    cur_proj.kind,
                    selected_proj.kind,
                )
        }
        (Interpolation(interpoint, selected_proj), _) => {
            let sp = Spline {
                from: selected_proj.pos.xy(),
                to: cur_proj.pos.xy(),
                from_derivative: (interpoint - selected_proj.pos.xy())
                    * std::f32::consts::FRAC_1_SQRT_2,
                to_derivative: (cur_proj.pos.xy() - interpoint) * std::f32::consts::FRAC_1_SQRT_2,
            };

            compatible(map, cur_proj, selected_proj)
                && check_angle(map, selected_proj, interpoint, is_rail)
                && check_angle(map, cur_proj, interpoint, is_rail)
                && !sp.is_steep(state.pattern_builder.width())
                && !check_intersect(
                    map,
                    &ShapeEnum::BoldSpline(BoldSpline::new(sp, patwidth * 0.5)),
                    (selected_proj.pos.z + cur_proj.pos.z) / 2.0,
                    selected_proj.kind,
                    cur_proj.kind,
                )
        }
        _ => true,
    };

    let build_args = match state.build_state {
        StartInterp(selected_proj) if !cur_proj.is_ground() => {
            Some((selected_proj, None, state.pattern_builder.build()))
        }
        Start(selected_proj) => Some((selected_proj, None, state.pattern_builder.build())),
        Interpolation(interpoint, selected_proj) => {
            let inter = Some(interpoint);
            Some((selected_proj, inter, state.pattern_builder.build()))
        }
        _ => None,
    };
    potential_command.0.clear();

    let mut points = None;

    if let Some((selected_proj, inter, pat)) = build_args {
        potential_command.set(WorldCommand::MapMakeConnection {
            from: selected_proj,
            to: cur_proj,
            inter,
            pat,
        });

        let connection_segment = match inter {
            Some(x) => RoadSegmentKind::from_elbow(selected_proj.pos.xy(), cur_proj.pos.xy(), x),
            None => RoadSegmentKind::Straight,
        };

        let (p, err) = simulation::map::Road::generate_points(
            selected_proj.pos,
            cur_proj.pos,
            connection_segment,
            is_rail,
            &map.environment,
        );
        points = Some(p);
        if err.is_some() {
            is_valid = false;
        }
    }

    state.update_drawing(map, immdraw, cur_proj, patwidth, is_valid, points);

    if is_valid && inp.just_act.contains(&InputAction::Select) {
        log::info!(
            "left clicked with state {:?} and {:?}",
            state.build_state,
            cur_proj.kind
        );

        match (state.build_state, cur_proj.kind) {
            (Hover, Ground) | (Hover, Road(_)) | (Hover, Inter(_)) => {
                // Hover selection
                if tool == Tool::RoadbuildCurved {
                    state.build_state = StartInterp(cur_proj);
                } else {
                    state.build_state = Start(cur_proj);
                }
            }
            (StartInterp(v), Ground) => {
                // Set interpolation point
                state.build_state = Interpolation(mousepos.xy(), v);
            }
            (Start(_) | StartInterp(_), _) => {
                // Straight connection to something
                immsound.play("road_lay", AudioKind::Ui);
                if let Some(wc) = potential_command.0.drain(..).next() {
                    commands.push(wc);
                }

                state.build_state = Hover;
            }
            (Interpolation(_, _), _) => {
                // Interpolated connection to something
                immsound.play("road_lay", AudioKind::Ui);
                if let Some(wc) = potential_command.0.drain(..).next() {
                    commands.push(wc);
                }

                state.build_state = Hover;
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct RoadBuildResource {
    pub build_state: BuildState,
    pub pattern_builder: LanePatternBuilder,
    pub snap_to_grid: bool,
    pub height_offset: f32,
}

fn check_angle(map: &Map, from: MapProject, to: Vec2, is_rail: bool) -> bool {
    let max_turn_angle = if is_rail {
        1.0 * std::f32::consts::PI / 180.0
    } else {
        30.0 * std::f32::consts::PI / 180.0
    };

    match from.kind {
        Inter(i) => {
            let inter = &map.intersections()[i];
            let dir = (to - inter.pos.xy()).normalize();
            for &road in &inter.roads {
                let road = &map.roads()[road];
                let v = road.dir_from(i);
                if v.angle(dir).abs() < max_turn_angle {
                    return false;
                }
            }
            true
        }
        Road(r) => {
            let Some(r) = map.roads().get(r) else {
                return false;
            };
            let (proj, _, rdir1) = r.points().project_segment_dir(from.pos);
            let rdir2 = -rdir1;
            let dir = (to - proj.xy()).normalize();
            if rdir1.xy().angle(dir).abs() < max_turn_angle
                || rdir2.xy().angle(dir).abs() < max_turn_angle
            {
                return false;
            }
            true
        }
        _ => true,
    }
}

fn compatible(map: &Map, x: MapProject, y: MapProject) -> bool {
    if x.pos.distance(y.pos) < 10.0 {
        return false;
    }
    match (x.kind, y.kind) {
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

/// Check if the given shape intersects with any existing road or intersection
fn check_intersect(
    map: &Map,
    obj: &ShapeEnum,
    z: f32,
    start: ProjectKind,
    end: ProjectKind,
) -> bool {
    map.spatial_map()
        .query(obj, ProjectFilter::ROAD | ProjectFilter::INTER)
        .any(move |x| {
            if let Road(rid) = x {
                let r = &map.roads()[rid];
                if (r.points.first().z - z).abs() > 0.1 || (r.points.last().z - z).abs() > 0.1 {
                    return false;
                }
                if let Inter(id) = start {
                    if r.src == id || r.dst == id {
                        return false;
                    }
                }
                if let Inter(id) = end {
                    if r.src == id || r.dst == id {
                        return false;
                    }
                }
            }
            x != start && x != end
        })
}

impl RoadBuildResource {
    pub fn update_drawing(
        &self,
        map: &Map,
        immdraw: &mut ImmediateDraw,
        proj: MapProject,
        patwidth: f32,
        is_valid: bool,
        points: Option<PolyLine3>,
    ) {
        let mut proj_pos = proj.pos;
        proj_pos.z += 0.4;
        let col = if is_valid {
            simulation::colors().gui_primary
        } else {
            simulation::colors().gui_danger
        };

        let p = match self.build_state {
            Hover => {
                immdraw.circle(proj_pos, patwidth * 0.5).color(col);
                return;
            }
            StartInterp(x) if proj.kind.is_ground() => {
                let dir = unwrap_or!((proj_pos - x.pos).try_normalize(), {
                    immdraw.circle(proj_pos, patwidth * 0.5).color(col);
                    return;
                });
                let mut poly = Vec::with_capacity(33);
                for i in 0..=32 {
                    let ang = std::f32::consts::PI * i as f32 * (2.0 / 32.0);
                    let mut v = Vec3::from_angle(ang, dir.z);
                    let center = if v.dot(dir) < 0.0 {
                        x.pos.up(0.4)
                    } else {
                        proj_pos
                    };

                    v = v * patwidth * 0.5;
                    v.z = 0.0;
                    v += center;

                    poly.push(v);
                }
                immdraw.polyline(poly, 3.0, true).color(col);

                return;
            }
            _ => unwrap_ret!(points),
        };

        for PylonPosition {
            terrain_height,
            pos,
            ..
        } in simulation::map::Road::pylons_positions(&p, &map.environment)
        {
            immdraw
                .circle(pos.xy().z(terrain_height + 0.1), patwidth * 0.5)
                .color(col);
        }

        immdraw.circle(p.first().up(0.4), patwidth * 0.5).color(col);
        immdraw.circle(p.last().up(0.4), patwidth * 0.5).color(col);
        immdraw
            .polyline(
                p.into_vec()
                    .into_iter()
                    .map(|v| v.up(0.4))
                    .collect::<Vec<_>>(),
                patwidth,
                false,
            )
            .color(col);
    }
}
