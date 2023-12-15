use super::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egui_inspect::Inspect;
use geom::Vec3;
use simulation::map::TerraformKind;
use simulation::world_command::WorldCommand;
use simulation::Simulation;

#[derive(Inspect)]
pub struct TerraformingResource {
    pub kind: TerraformKind,
    pub radius: f32,
    pub amount: f32,
    #[inspect(skip)]
    pub level: f32,
    #[inspect(skip)]
    pub slope: Option<(Vec3, Vec3)>,
}

/// Lot brush tool
/// Allows to build houses on lots
pub fn terraforming(sim: &Simulation, uiworld: &mut UiWorld) {
    profiling::scope!("gui::terraforming");
    let res = uiworld.write::<TerraformingResource>();
    let tool = *uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let _draw = uiworld.write::<ImmediateDraw>();
    let _map = sim.map();
    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::Terraforming) {
        return;
    }

    let mpos = unwrap_ret!(inp.unprojected);

    if inp.act.contains(&InputAction::Select) {
        commands.push(WorldCommand::Terraform {
            center: mpos.xy(),
            radius: res.radius,
            amount: res.amount,
            level: res.level,
            kind: res.kind,
            slope: res.slope,
        })
    }
}

impl Default for TerraformingResource {
    fn default() -> Self {
        Self {
            kind: TerraformKind::Erode,
            radius: 1000.0,
            amount: 200.0,
            level: 50.0,
            slope: None,
        }
    }
}
