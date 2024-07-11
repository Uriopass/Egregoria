use geom::{Vec2, Vec3, OBB};
use simulation::map::TerraformKind;
use simulation::world_command::WorldCommand;
use simulation::Simulation;

use crate::gui::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;

pub struct TerraformingResource {
    pub kind: TerraformKind,
    pub radius: f32,
    pub amount: f32,
    level: Option<f32>,
    slope_start: Option<Vec3>,
    slope_end: Option<Vec3>,
}

/// Lot brush tool
/// Allows to build houses on lots
pub fn terraforming(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::terraforming");
    let mut res = uiworld.write::<TerraformingResource>();
    let tool = *uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let _map = sim.map();
    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::Terraforming) {
        res.slope_start = None;
        res.slope_end = None;
        return;
    }

    if inp.act.contains(&InputAction::SizeUp) {
        res.radius *= 1.1;
    }
    if inp.act.contains(&InputAction::SizeDown) {
        res.radius /= 1.1;
    }

    let mpos = unwrap_ret!(inp.unprojected);

    let mut amount_multiplier = 1.0;

    // handle actions
    match res.kind {
        TerraformKind::Elevation => {
            if inp.act.contains(&InputAction::SecondarySelect) {
                amount_multiplier = -1.0;
            }
        }
        TerraformKind::Smooth => {}
        TerraformKind::Level => {
            // set level on first click
            if res.level.is_none() && inp.just_act.contains(&InputAction::Select) {
                res.level = Some(mpos.z);
            }

            // when hold is released, reset level
            if !inp.act.contains(&InputAction::Select) {
                res.level = None;
            }
        }
        TerraformKind::Slope => {
            // Set the end slope (second click)
            if res.slope_start.is_some()
                && res.slope_end.is_none()
                && inp.just_act.contains(&InputAction::Select)
                && !res.slope_start.unwrap().is_close(mpos, 5.0)
            {
                res.slope_end = Some(mpos);
            }

            // Set the start slope (first click)
            if res.slope_start.is_none() && inp.just_act.contains(&InputAction::Select) {
                res.slope_start = Some(mpos);
            }

            if inp.just_act.contains(&InputAction::Close) {
                res.slope_start = None;
                res.slope_end = None;
            }
        }
        TerraformKind::Erode => {}
    }

    if inp.act.contains(&InputAction::Select) || inp.act.contains(&InputAction::SecondarySelect) {
        if res.kind == TerraformKind::Level && res.level.is_none() {
            return;
        }
        if res.kind == TerraformKind::Slope && res.slope_end.is_none() {
            return;
        }
        commands.push(WorldCommand::Terraform {
            center: mpos.xy(),
            radius: res.radius,
            amount: res.amount * amount_multiplier,
            level: res.level.unwrap_or(0.0),
            kind: res.kind,
            slope: res.slope_start.zip(res.slope_end),
        })
    }

    // Draw the state
    match res.kind {
        TerraformKind::Elevation => {}
        TerraformKind::Smooth => {}
        TerraformKind::Level => {
            if !inp.act.contains(&InputAction::Select) {
                draw.obb(
                    OBB::new(
                        mpos.xy(),
                        Vec2::X,
                        res.radius * std::f32::consts::FRAC_1_SQRT_2,
                        res.radius * std::f32::consts::FRAC_1_SQRT_2,
                    ),
                    res.level.unwrap_or(mpos.z) - 0.5,
                )
                .color(simulation::colors().gui_primary.a(0.2));
            }
        }
        TerraformKind::Slope => {
            if res.slope_start.is_none() {
                draw.circle(mpos, res.radius * 0.7)
            } else if res.slope_end.is_none() {
                draw.line(res.slope_start.unwrap(), mpos, res.radius)
            } else {
                draw.line(res.slope_start.unwrap(), res.slope_end.unwrap(), res.radius)
            }
            .color(simulation::colors().gui_primary.a(0.2));
        }
        TerraformKind::Erode => {}
    }
}

impl Default for TerraformingResource {
    fn default() -> Self {
        Self {
            kind: TerraformKind::Elevation,
            radius: 200.0,
            amount: 300.0,
            level: None,
            slope_start: None,
            slope_end: None,
        }
    }
}
