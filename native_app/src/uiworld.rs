use atomic_refcell::{AtomicRef, AtomicRefMut};
use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use legion::storage::Component;
use legion::systems::Resource;
use legion::{Entity, IntoQuery, Resources, World};

#[derive(Default)]
pub struct UiWorld {
    pub world: World,
    resources: Resources,
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

    pub fn commands(&self) -> AtomicRefMut<WorldCommands> {
        self.write::<WorldCommands>()
    }
    pub fn received_commands(&self) -> AtomicRef<ReceivedCommands> {
        self.read::<ReceivedCommands>()
    }

    pub fn add_comp(&mut self, e: Entity, c: impl Component) {
        if self
            .world
            .entry(e)
            .map(move |mut e| e.add_component(c))
            .is_none()
        {
            log::error!("trying to add component to entity but it doesn't exist");
        }
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<&T> {
        <&T>::query().get(&self.world, e).ok()
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        <&mut T>::query().get_mut(&mut self.world, e).ok()
    }

    pub fn write_or_default<T: Resource + Default>(&mut self) -> AtomicRefMut<T> {
        self.resources.get_mut_or_insert_with(T::default)
    }

    pub fn try_write<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        self.resources.get_mut()
    }

    pub fn write<T: Resource>(&self) -> AtomicRefMut<T> {
        self.resources
            .get_mut()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn read<T: Resource>(&self) -> AtomicRef<T> {
        self.resources
            .get()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn insert<T: Resource>(&mut self, res: T) {
        self.resources.insert(res)
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
