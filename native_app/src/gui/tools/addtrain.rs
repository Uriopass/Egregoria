use crate::gui::{PotentialCommands, Tool};
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use geom::{Color, OBB};
use prototypes::RollingStockID;
use simulation::map::LaneKind;
use simulation::transportation::train::{calculate_locomotive, wagons_positions_for_render};
use simulation::world_command::WorldCommand;
use simulation::Simulation;
use std::option::Option::None;

#[derive(Clone, Debug, Default)]
pub struct TrainSpawnResource {
    pub wagons: Vec<RollingStockID>,
    /// m/s
    pub max_speed: f32,
    /// m/s^2
    pub acceleration: f32,
    /// m/s^2
    pub deceleration: f32,
    /// meter
    pub total_lenght: f32,
}

/// Addtrain handles the "Adding a train" tool
/// It allows to add a train to any rail lane
pub fn addtrain(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::addtrain");
    let state = &mut *uiworld.write::<TrainSpawnResource>();
    let tool = *uiworld.read::<Tool>();

    if !matches!(tool, Tool::Train) {
        state.wagons.clear();
        state.set_zero();
        return;
    }

    let inp = uiworld.read::<InputMap>();
    let mut potential = uiworld.write::<PotentialCommands>();

    let mut draw = uiworld.write::<ImmediateDraw>();
    let map = sim.map();
    let commands = &mut *uiworld.commands();

    let mpos = unwrap_ret!(inp.unprojected);

    let nearbylane = map.nearest_lane(mpos, LaneKind::Rail, Some(20.0));

    let nearbylane = match nearbylane.and_then(|x| map.lanes().get(x)) {
        Some(x) => x,
        None => {
            draw.circle(mpos, 10.0)
                .color(simulation::colors().gui_danger);
            return;
        }
    };

    if state.wagons.is_empty() {
        return;
    }

    let proj = nearbylane.points.project(mpos);
    let dist = nearbylane.points.length_at_proj(proj);

    let trainlength = state.total_lenght + 1.0;

    let mut drawtrain = |col: Color| {
        wagons_positions_for_render(&state.wagons, &nearbylane.points, dist).for_each(
            |(pos, dir, length)| {
                draw.obb(OBB::new(pos.xy(), dir.xy(), length, 4.0), pos.z + 0.5)
                    .color(col);
            },
        );
    };

    if dist <= trainlength {
        drawtrain(simulation::colors().gui_danger);
        return;
    }

    drawtrain(simulation::colors().gui_primary);

    let cmd = WorldCommand::SpawnTrain {
        wagons: state.wagons.clone(),
        lane: nearbylane.id,
        dist,
    };

    if inp.just_act.contains(&InputAction::Select) {
        commands.push(cmd);
    } else {
        potential.set(cmd);
    }
}

impl TrainSpawnResource {
    pub fn calculate(&mut self) {
        let locomotive = calculate_locomotive(&self.wagons);
        if locomotive.acc_force.is_nan() || locomotive.dec_force.is_nan() {
            self.set_zero();
            return;
        }

        self.max_speed = locomotive.max_speed;
        self.acceleration = locomotive.acc_force;
        self.deceleration = locomotive.dec_force;
        self.total_lenght = locomotive.length;
    }

    pub fn set_zero(&mut self) {
        self.max_speed = 0.0;
        self.acceleration = 0.0;
        self.deceleration = 0.0;
        self.total_lenght = 0.0;
    }
}
