use super::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use common::timestep::UP_DT;
use egui_inspect::Inspect;
use geom::LinearColor;
use simulation::map::TerraformKind;
use simulation::world_command::WorldCommand;
use simulation::Simulation;

#[derive(Inspect)]
pub struct TerraformingResource {
    pub kind: TerraformKind,
    pub radius: f32,
    pub amount: f32,
}

/// Lot brush tool
/// Allows to build houses on lots
pub fn terraforming(sim: &Simulation, uiworld: &mut UiWorld) {
    profiling::scope!("gui::terraforming");
    let res = uiworld.write::<TerraformingResource>();
    let tool = *uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let _map = sim.map();
    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::Terraforming) {
        return;
    }

    let mpos = unwrap_ret!(inp.unprojected);
    draw.circle(mpos.up(0.8), res.radius)
        .color(LinearColor::GREEN.a(0.1));

    if inp.act.contains(&InputAction::Select) {
        commands.push(WorldCommand::Terraform {
            center: mpos.xy(),
            radius: res.radius,
            amount: res.amount * UP_DT.as_secs_f32(),
            kind: res.kind,
        })
    }
}

impl Default for TerraformingResource {
    fn default() -> Self {
        Self {
            kind: TerraformKind::Raise,
            radius: 200.0,
            amount: 200.0,
        }
    }
}
