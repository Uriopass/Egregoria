use crate::init::{INIT_FUNCS, SAVELOAD_FUNCS};
use simulation::engine_interaction::{WorldCommand, WorldCommands};
use simulation::utils::resources::{Ref, RefMut, Resources};
use simulation::{Simulation, SimulationReplayLoader};
use std::any::Any;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Default)]
pub struct UiWorld {
    resources: Resources,
}

#[derive(Default)]
pub struct SaveLoadState {
    pub please_load: Option<SimulationReplayLoader>,
    pub please_load_sim: Option<Simulation>,
    pub render_reset: bool,
    pub please_save: bool,
    pub saving_status: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl UiWorld {
    pub fn init() -> UiWorld {
        let mut w = UiWorld::default();
        unsafe {
            for s in &INIT_FUNCS {
                (s.f)(&mut w);
            }
        }
        w.load_from_disk();
        w
    }

    pub fn commands(&self) -> RefMut<WorldCommands> {
        self.write::<WorldCommands>()
    }

    pub fn received_commands(&self) -> Ref<ReceivedCommands> {
        self.read::<ReceivedCommands>()
    }

    pub fn try_write<T: Any + Send + Sync>(&self) -> Option<RefMut<T>> {
        self.resources.try_write().ok()
    }

    pub fn write<T: Any + Send + Sync>(&self) -> RefMut<T> {
        self.resources.write()
    }

    pub fn read<T: Any + Send + Sync>(&self) -> Ref<T> {
        self.resources.read()
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, res: T) {
        self.resources.insert(res);
    }

    pub fn camera_mut(&self) -> RefMut<crate::rendering::OrbitCamera> {
        self.write::<crate::rendering::OrbitCamera>()
    }

    pub fn check_present<T: Any + Send + Sync>(&mut self, res: fn() -> T) {
        self.resources.write_or_insert_with(res);
    }

    fn load_from_disk(&mut self) {
        unsafe {
            for l in &SAVELOAD_FUNCS {
                (l.load)(self);
            }
        }
    }

    pub fn save_to_disk(&self) {
        unsafe {
            for l in &SAVELOAD_FUNCS {
                (l.save)(self);
            }
        }
    }
}

#[derive(Default)]
pub struct ReceivedCommands(WorldCommands);

impl ReceivedCommands {
    #[allow(dead_code)]
    pub fn new(commands: WorldCommands) -> Self {
        Self(commands)
    }
    pub fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.0.iter()
    }
}
