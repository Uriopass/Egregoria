use crate::utils::scheduler::SeqSchedule;
use crate::world_command::WorldCommand;
use crate::Simulation;
use prototypes::Tick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Replay {
    pub enabled: bool,
    commands: Vec<(Tick, WorldCommand)>,
    pub last_tick_recorded: Tick,
}

impl Replay {
    pub fn push(&mut self, tick: Tick, command: WorldCommand) {
        self.commands.push((tick, command));
    }
}

pub struct SimulationReplayLoader {
    pub replay: Replay,
    pub pastt: Tick,
    pub idx: usize,
    pub speed: usize,
    pub advance_n_ticks: usize,
}

impl SimulationReplayLoader {
    /// Returns true if the replay is finished
    pub fn advance_tick(&mut self, sim: &mut Simulation, schedule: &mut SeqSchedule) -> bool {
        // iterate through tick grouped commands
        let mut ticks_left = if self.speed == 0 {
            let v = self.advance_n_ticks;
            self.advance_n_ticks = 0;
            v
        } else {
            self.speed
        };
        while self.idx < self.replay.commands.len() && ticks_left > 0 {
            let curt = self.replay.commands[self.idx].0;
            while self.pastt < curt {
                sim.tick(schedule, &[]);
                self.pastt.0 += 1;
                ticks_left -= 1;
                if ticks_left == 0 {
                    return false;
                }
            }

            let idx_start = self.idx;
            while self.idx < self.replay.commands.len() && self.replay.commands[self.idx].0 == curt
            {
                self.idx += 1;
            }
            let command_slice = &self.replay.commands[idx_start..self.idx];

            log::info!(
                "[replay] acttick {:?} ({})",
                self.pastt,
                command_slice.len()
            );
            sim.tick(schedule, command_slice.iter().map(|(_, c)| c));
            self.pastt.0 += 1;
            ticks_left -= 1;
            if ticks_left == 0 {
                return false;
            }
        }
        if self.idx < self.replay.commands.len() {
            return false;
        }
        while ticks_left > 0 && self.pastt < self.replay.last_tick_recorded {
            sim.tick(schedule, &[]);
            self.pastt.0 += 1;
            ticks_left -= 1;
            if ticks_left == 0 {
                return false;
            }
        }
        true
    }
}
