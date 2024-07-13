use crate::gui::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use geom::Color;
use simulation::map::{IntersectionID, LightPolicy, TurnPolicy};
use simulation::map::{ProjectFilter, ProjectKind};
use simulation::Simulation;

#[derive(Clone)]
pub struct IntersectionComponent {
    pub id: IntersectionID,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

#[derive(Default)]
pub struct RoadEditorResource {
    pub inspect: Option<IntersectionComponent>,
    pub dirty: bool,
}

/// RoadEditor tool
/// Allows to edit intersections properties like turns and signals
pub fn roadeditor(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::roadeditor");
    let tool = uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let mut state = uiworld.write::<RoadEditorResource>();
    let mut imm_draw = uiworld.write::<ImmediateDraw>();
    let map = sim.map();
    let commands = &mut *uiworld.commands();

    if !matches!(*tool, Tool::RoadEditor) {
        state.inspect = None;
        return;
    }

    if let Some(id) = state.inspect.as_ref().map(|x| x.id) {
        if let Some(inter) = map.intersections().get(id) {
            let lanes = map.lanes();
            for turn in inter.turns() {
                let r = common::rand::randhash(turn.id);
                let col = Color::hsv(r * 360.0, 0.8, 0.6, 0.5);

                let or_src = unwrap_cont!(lanes.get(turn.id.src)).orientation_from(inter.id);
                let or_dst = unwrap_cont!(lanes.get(turn.id.dst)).orientation_from(inter.id);

                let p: Vec<_> = std::iter::once(turn.points.first() + or_src.z0() * 0.01)
                    .chain(turn.points.iter().copied())
                    .chain(std::iter::once(turn.points.last() + or_dst.z0() * 0.01))
                    .map(|x| x.up(0.05 + r * 0.01))
                    .collect();

                imm_draw.polyline(p, 1.0, false).color(col);
            }
        } else {
            state.inspect = None;
        }
    }

    let mut proj_pos = unwrap_ret!(inp.unprojected);
    let cur_proj = map.project(proj_pos, 10.0, ProjectFilter::INTER);

    let mut proj_col;

    if let ProjectKind::Intersection(id) = cur_proj.kind {
        if Some(id) != state.inspect.as_ref().map(|x| x.id) {
            proj_pos = cur_proj.pos;
        }
        proj_col = simulation::colors().gui_primary;
    } else {
        proj_col = simulation::colors().gui_disabled;
    }

    if inp.act.contains(&InputAction::Select) {
        if let ProjectKind::Intersection(id) = cur_proj.kind {
            proj_col = simulation::colors().gui_success;
            proj_pos = cur_proj.pos;
            let inter = &map.intersections()[id];
            state.inspect = Some(IntersectionComponent {
                id,
                turn_policy: inter.turn_policy,
                light_policy: inter.light_policy,
            });
            state.dirty = false;
        }
    }

    imm_draw.circle(proj_pos.up(0.5), 10.0).color(proj_col);

    if state.dirty {
        if let Some(interc) = &state.inspect {
            commands.map_update_intersection_policy(
                interc.id,
                interc.turn_policy,
                interc.light_policy,
            );
        }
        state.dirty = false;
    }
}
