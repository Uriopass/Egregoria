use crate::gui::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use geom::Color;
use map_model::ProjectKind;
use map_model::{IntersectionID, LightPolicy, TurnPolicy};

#[derive(Clone)]
pub struct IntersectionComponent {
    pub id: IntersectionID,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

register_resource_noserialize!(RoadEditorResource);
#[derive(Default)]
pub struct RoadEditorResource {
    pub inspect: Option<IntersectionComponent>,
    pub dirty: bool,
}

#[profiling::function]
pub fn roadeditor(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool = uiworld.read::<Tool>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let mut state = uiworld.write::<RoadEditorResource>();
    let mut imm_draw = uiworld.write::<ImmediateDraw>();
    let map = goria.map();
    let commands = &mut *uiworld.commands();

    if !matches!(*tool, Tool::RoadEditor) {
        state.inspect = None;
        return;
    }

    if let Some(id) = state.inspect.as_ref().map(|x| x.id) {
        if let Some(inter) = map.intersections().get(id) {
            let lanes = map.lanes();
            for turn in inter.turns() {
                let p = unwrap_or!(turn.points.get(turn.points.n_points() / 2), continue);
                let r = common::rand::rand2(p.x, p.y);
                let col = Color::hsv(r * 360.0, 0.8, 0.6, 0.5);

                let or_src = unwrap_cont!(lanes.get(turn.id.src)).orientation_from(inter.id);
                let or_dst = unwrap_cont!(lanes.get(turn.id.dst)).orientation_from(inter.id);

                let p: Vec<_> = std::iter::once(turn.points.first() + or_src.z0() * 0.01)
                    .chain(turn.points.iter().copied())
                    .chain(std::iter::once(turn.points.last() + or_dst.z0() * 0.01))
                    .collect();

                imm_draw.polyline(p, 1.0).color(col);
            }
        } else {
            state.inspect = None;
        }
    }

    let mut proj_pos = unwrap_ret!(mouseinfo.unprojected);
    let cur_proj = unwrap_ret!(map.project(proj_pos, 0.0));

    let mut proj_col;

    if let ProjectKind::Inter(id) = cur_proj.kind {
        if Some(id) != state.inspect.as_ref().map(|x| x.id) {
            proj_pos = cur_proj.pos;
        }
        proj_col = common::config().gui_primary;
    } else {
        proj_col = common::config().gui_disabled;
    }

    if mouseinfo.pressed.contains(&MouseButton::Left) {
        if let ProjectKind::Inter(id) = cur_proj.kind {
            proj_col = common::config().gui_success;
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

    imm_draw.circle(proj_pos, 10.0).color(proj_col);

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
