use crate::init::{INIT_FUNCS, SAVELOAD_FUNCS};
use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use egregoria::Replay;
use hecs::{Component, DynamicBundle, QueryOne};
use hecs::{Entity, World};
use resources::{Ref, RefMut, Resource};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct UiWorld {
    pub(crate) world: World,
    resources: resources::Resources,
}

#[derive(Default)]
pub struct SaveLoadState {
    pub please_load: Option<Replay>,
    pub please_save: bool,
    pub saving_status: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl UiWorld {
    pub(crate) fn init() -> UiWorld {
        let mut w = UiWorld::default();
        unsafe {
            for s in &INIT_FUNCS {
                (s.f)(&mut w);
            }
        }
        w.load_from_disk();
        w
    }

    pub(crate) fn commands(&self) -> RefMut<WorldCommands> {
        self.write::<WorldCommands>()
    }

    pub(crate) fn received_commands(&self) -> Ref<ReceivedCommands> {
        self.read::<ReceivedCommands>()
    }

    pub(crate) fn add_comp(&mut self, e: Entity, c: impl DynamicBundle) {
        if self.world.insert(e, c).is_err() {
            log::error!("trying to add component to entity but it doesn't exist");
        }
    }

    pub(crate) fn comp<T: Component>(&self, e: Entity) -> Option<QueryOne<&T>> {
        self.world.query_one::<&T>(e).ok()
    }

    pub(crate) fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        self.world.query_one_mut::<&mut T>(e).ok()
    }

    pub(crate) fn try_write<T: Resource>(&self) -> Option<RefMut<T>> {
        self.resources.get_mut().ok()
    }

    pub(crate) fn write<T: Resource>(&self) -> RefMut<T> {
        self.resources
            .get_mut()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub(crate) fn read<T: Resource>(&self) -> Ref<T> {
        self.resources
            .get()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub(crate) fn insert<T: Resource>(&mut self, res: T) {
        self.resources.insert(res);
    }

    pub(crate) fn check_present<T: Resource>(&mut self, res: fn() -> T) {
        self.resources.entry::<T>().or_insert_with(res);
    }

    fn load_from_disk(&mut self) {
        unsafe {
            for l in &SAVELOAD_FUNCS {
                (l.load)(self);
            }
        }
    }

    pub(crate) fn save_to_disk(&self) {
        unsafe {
            for l in &SAVELOAD_FUNCS {
                (l.save)(self);
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ReceivedCommands(WorldCommands);

impl ReceivedCommands {
    #[allow(dead_code)]
    pub(crate) fn new(commands: WorldCommands) -> Self {
        Self(commands)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.0.iter()
    }
}
