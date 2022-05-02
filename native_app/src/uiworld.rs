use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use hecs::{Component, DynamicBundle, QueryOne};
use hecs::{Entity, World};
use resources::{Ref, RefMut, Resource};

#[derive(Default)]
pub struct UiWorld {
    pub world: World,
    resources: resources::Resources,
}

#[allow(dead_code)]
impl UiWorld {
    pub fn init() -> UiWorld {
        let mut w = UiWorld::default();
        for s in inventory::iter::<InitFunc> {
            (s.f)(&mut w);
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

    pub fn add_comp(&mut self, e: Entity, c: impl DynamicBundle) {
        if self.world.insert(e, c).is_err() {
            log::error!("trying to add component to entity but it doesn't exist");
        }
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<QueryOne<&T>> {
        self.world.query_one::<&T>(e).ok()
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        self.world.query_one_mut::<&mut T>(e).ok()
    }

    pub fn write_or_default<T: Resource + Default>(&mut self) -> RefMut<T> {
        self.resources.entry::<T>().or_default()
    }

    pub fn try_write<T: Resource>(&self) -> Option<RefMut<T>> {
        self.resources.get_mut().ok()
    }

    pub fn write<T: Resource>(&self) -> RefMut<T> {
        self.resources
            .get_mut()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn read<T: Resource>(&self) -> Ref<T> {
        self.resources
            .get()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn insert<T: Resource>(&mut self, res: T) {
        self.resources.insert(res);
    }

    fn load_from_disk(&mut self) {
        for l in inventory::iter::<SaveLoadFunc> {
            (l.load)(self);
        }
    }

    pub(crate) fn save_to_disk(&self) {
        for l in inventory::iter::<SaveLoadFunc> {
            (l.save)(self);
        }
    }
}

pub(crate) struct SaveLoadFunc {
    pub save: Box<dyn Fn(&UiWorld) + 'static>,
    pub load: Box<dyn Fn(&mut UiWorld) + 'static>,
}
inventory::collect!(SaveLoadFunc);

pub(crate) struct InitFunc {
    pub f: Box<dyn Fn(&mut UiWorld) + 'static>,
}
inventory::collect!(InitFunc);

macro_rules! init_func {
    ($f: expr) => {
        inventory::submit! {
            $crate::uiworld::InitFunc {
                f: Box::new($f),
            }
        }
    };
}

macro_rules! register_resource {
    ($t: ty, $name: expr) => {
        init_func!(|uiworld| {
            uiworld.insert(<$t>::default());
        });
        inventory::submit! {
            $crate::uiworld::SaveLoadFunc {
                save: Box::new(|uiworld| {
                     <common::saveload::JSON as common::saveload::Encoder>::save(&*uiworld.read::<$t>(), $name);
                }),
                load: Box::new(|uiworld| {
                    if let Some(res) = <common::saveload::JSON as common::saveload::Encoder>::load::<$t>($name) {
                        uiworld.insert(res);
                    }
                })
            }
        }
    };
}

macro_rules! register_resource_noserialize {
    ($t: ty) => {
        init_func!(|uiworld| {
            uiworld.insert(<$t>::default());
        });
    };
}

#[derive(Default)]
pub struct ReceivedCommands(WorldCommands);
register_resource_noserialize!(ReceivedCommands);

impl ReceivedCommands {
    pub fn new(commands: WorldCommands) -> Self {
        Self(commands)
    }
    pub fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.0.iter()
    }
}
