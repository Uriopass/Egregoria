use super::Tool;
use crate::gui::PotentialCommands;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::map::LaneKind;
use egregoria::transportation::train::{train_length, wagons_positions_for_render};
use egregoria::Egregoria;
use geom::{Color, OBB};
use std::option::Option::None;

/// Addtrain handles the "Adding a train" tool
/// It allows to add a train to any rail lane
#[profiling::function]
pub(crate) fn addtrain(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool = *uiworld.read::<Tool>();
    if !matches!(tool, Tool::Train) {
        return;
    }

    let inp = uiworld.read::<InputMap>();
    let mut potential = uiworld.write::<PotentialCommands>();

    let mut draw = uiworld.write::<ImmediateDraw>();
    let map = goria.map();
    let commands = &mut *uiworld.commands();

    let mpos = unwrap_ret!(inp.unprojected);

    let nearbylane = map.nearest_lane(mpos, LaneKind::Rail, Some(20.0));

    let nearbylane = match nearbylane.and_then(|x| map.lanes().get(x)) {
        Some(x) => x,
        None => {
            draw.circle(mpos, 10.0)
                .color(egregoria::config().gui_danger);
            return;
        }
    };

    let proj = nearbylane.points.project(mpos);
    let dist = nearbylane.points.length_at_proj(proj);

    let n_wagons = 7;
    let trainlength = train_length(n_wagons);

    let mut drawtrain = |col: Color| {
        for (p, dir) in wagons_positions_for_render(&nearbylane.points, dist, n_wagons) {
            draw.obb(OBB::new(p.xy(), dir.xy(), 16.5, 3.0), p.z + 0.5)
                .color(col);
        }
    };

    if dist <= trainlength {
        drawtrain(egregoria::config().gui_danger);
        return;
    }

    drawtrain(egregoria::config().gui_primary);

    let cmd = WorldCommand::AddTrain {
        dist,
        n_wagons,
        lane: nearbylane.id,
    };
    if inp.just_act.contains(&InputAction::Select) {
        commands.push(cmd);
    } else {
        potential.set(cmd);
    }
}
