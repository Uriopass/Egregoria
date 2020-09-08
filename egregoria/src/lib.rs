#![allow(clippy::unreadable_literal)]
#![allow(clippy::block_in_if_condition_stmt)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use crate::frame_log::FrameLog;
use crate::gui::Gui;
use crate::interaction::{
    bulldozer_system, movable_system, roadbuild_system, roadeditor_system,
    selectable_cleanup_system, selectable_select_system, BulldozerResource, FollowEntity,
    InspectedAura, InspectedEntity, RoadEditorResource, Tool,
};
use crate::interaction::{inspected_aura_system, MovableSystem, RoadBuildResource};
use crate::lua::scenario_runner::{run_scenario_system, RunningScenario};
use crate::map_dynamic::{itinerary_update_system, ParkingManagement};
use crate::pedestrians::pedestrian_decision_system;
use crate::physics::systems::{
    coworld_maintain_system, coworld_synchronize_system, kinematics_apply_system,
};
use crate::physics::Collider;
use crate::physics::CollisionWorld;
use crate::rendering::immediate::ImmediateDraw;
use crate::vehicles::systems::{vehicle_cleanup_system, vehicle_decision_system};
use map_model::{Map, SerializedMap};
use rand_provider::RandProvider;
use std::io::Write;

#[macro_use]
extern crate log as extern_log;

#[macro_use]
pub mod utils;

#[macro_use]
mod frame_log;

#[macro_use]
pub mod gui;

pub mod engine_interaction;
pub mod interaction;
pub mod lua;
pub mod map_dynamic;
mod pedestrians;
pub mod physics;
pub mod rand_provider;
pub mod rendering;
mod saveload;
mod souls;
mod vehicles;

use crate::souls::Souls;
use crate::vehicles::VehicleComponent;
pub use imgui;
use legion::query::Query;
use legion::storage::Component;
use legion::systems::Resource;
use legion::{Entity, EntityStore, IntoQuery, Resources, Schedule, World};
use std::ops::{Deref, DerefMut};

pub use legion;
use std::sync::Mutex;

pub struct Egregoria {
    pub world: World,
    resources: Resources,
    schedule: Schedule,
}

#[derive(Default)]
pub struct ParCommandBuffer(
    Mutex<Vec<Entity>>,
    Mutex<Vec<Box<dyn for<'a> FnOnce(&'a mut Egregoria) -> () + Send>>>,
);

impl ParCommandBuffer {
    pub fn kill(&self, e: Entity) {
        self.0.lock().unwrap().push(e);
    }
    pub fn kill_all(&self, e: &[Entity]) {
        self.0.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec(&self, f: impl for<'a> FnOnce(&'a mut Egregoria) -> () + 'static + Send) {
        self.1.lock().unwrap().push(Box::new(f));
    }

    pub fn add_component<T: Component>(&self, e: Entity, c: T) {
        self.exec(move |w| {
            if let Some(mut x) = w.world.entry(e) {
                x.add_component(c)
            }
        })
    }

    pub fn remove_component<T: Component + Clone>(&self, e: Entity) {
        self.exec(move |w| {
            w.parse_del::<T>(e);
            w.world.entry(e).map(move |mut x| x.remove_component::<T>());
        })
    }
}

pub struct Deleted<T>(Vec<T>);
impl<T> Default for Deleted<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

const RNG_SEED: u64 = 123;

pub struct WriteStorage<'a, T: Component> {
    world: &'a mut World,
    query: Query<&'a mut T>,
}

impl<'a, T: Component> WriteStorage<'a, T> {
    pub fn get_mut(&mut self, e: Entity) -> Option<&mut T> {
        self.query.get_mut(self.world, e).ok()
    }
}

pub struct ReadStorage<'a, T: Component> {
    world: &'a World,
    query: Query<&'a T>,
}

impl<'a, T: Component> ReadStorage<'a, T> {
    pub fn get(&mut self, e: Entity) -> Option<&T> {
        self.query.get(self.world, e).ok()
    }
}

impl Egregoria {
    pub fn write_component<T: Component>(&mut self) -> WriteStorage<T> {
        WriteStorage {
            world: &mut self.world,
            query: <&mut T>::query(),
        }
    }

    pub fn read_component<T: Component>(&self) -> ReadStorage<T> {
        ReadStorage {
            world: &self.world,
            query: <&T>::query(),
        }
    }

    pub fn write_resource<T: Resource>(&self) -> impl DerefMut<Target = T> + '_ {
        self.resources.get_mut().unwrap()
    }

    pub fn read_resource<T: Resource>(&self) -> impl Deref<Target = T> + '_ {
        self.resources.get().unwrap()
    }

    pub fn insert<T: Resource>(&mut self, value: T) {
        self.resources.insert(value)
    }

    pub fn parse_del<T: Component + Clone>(&mut self, entity: Entity) {
        if let Some(v) = self
            .world
            .entry_ref(entity)
            .ok()
            .and_then(|x| x.get_component::<T>().ok().cloned())
        {
            self.resources
                .get_mut::<Deleted<T>>()
                .map(move |mut x| x.0.push(v));
        }
    }

    pub fn run(&mut self) {
        self.read_resource::<FrameLog>().clear();
        let t = std::time::Instant::now();
        self.schedule.execute(&mut self.world, &mut self.resources);

        let deleted: Vec<Entity> = std::mem::take(
            self.resources
                .get_mut::<ParCommandBuffer>()
                .unwrap()
                .0
                .lock()
                .unwrap()
                .as_mut(),
        );
        for entity in deleted {
            self.parse_del::<Collider>(entity);
            self.parse_del::<VehicleComponent>(entity);
            self.world.remove(entity);
        }

        let funs: Vec<Box<dyn for<'a> FnOnce(&'a mut Egregoria) -> () + Send>> = std::mem::take(
            &mut *self
                .resources
                .get_mut::<ParCommandBuffer>()
                .unwrap()
                .1
                .lock()
                .unwrap(),
        );
        for fun in funs {
            fun(self);
        }

        self.write_resource::<RenderStats>()
            .add_update_time(t.elapsed().as_secs_f32());
    }

    pub fn init() -> Egregoria {
        let mut world = World::default();
        let mut resources = Resources::default();

        info!("Seed is {}", RNG_SEED);

        // Basic resources init
        resources.insert(TimeInfo::default());
        resources.insert(CollisionWorld::new(50));
        resources.insert(KeyboardInfo::default());
        resources.insert(MouseInfo::default());
        resources.insert(InspectedEntity::default());
        resources.insert(FollowEntity::default());
        resources.insert(RenderStats::default());
        resources.insert(RandProvider::new(RNG_SEED));
        resources.insert(ParkingManagement::default());
        resources.insert(FrameLog::default());
        resources.insert(RunningScenario::default());
        resources.insert(ImmediateDraw::default());
        resources.insert(Souls::default());
        resources.insert(ParCommandBuffer::default());
        resources.insert(Tool::default());
        resources.insert(RunningScenario::default());
        resources.insert(Deleted::<Collider>::default());
        resources.insert(Deleted::<VehicleComponent>::default());
        resources.insert(ParCommandBuffer::default());

        // Systems state init
        let s = RoadBuildResource::new(&mut world);
        resources.insert(s);

        let s = RoadEditorResource::new(&mut world);
        resources.insert(s);

        let s = BulldozerResource::new(&mut world);
        resources.insert(s);

        // Dispatcher init
        let schedule = Schedule::builder()
            .add_system(vehicle_decision_system())
            .add_system(selectable_select_system())
            .add_system(selectable_cleanup_system())
            .add_system(roadbuild_system())
            .add_system(roadeditor_system())
            .add_system(bulldozer_system())
            .add_system(itinerary_update_system())
            .add_system(vehicle_cleanup_system())
            .add_system(pedestrian_decision_system())
            .add_system(run_scenario_system())
            .add_system(movable_system(MovableSystem::default()))
            .add_system(kinematics_apply_system())
            .add_system(coworld_synchronize_system())
            .add_system(coworld_maintain_system())
            .add_system(inspected_aura_system(InspectedAura::new(&mut world)))
            .build();

        Self {
            world,
            resources,
            schedule,
        }
    }
}

pub fn load_from_disk(goria: &mut Egregoria) {
    let map: Map = saveload::load_or_default::<map_model::SerializedMap>("map").into();
    goria.insert(map);
    goria.insert(crate::saveload::load_or_default::<Gui>("gui"));
}

pub fn save_to_disk(goria: &mut Egregoria) {
    let _ = std::io::stdout().flush();
    crate::saveload::save(&*goria.read_resource::<Gui>(), "gui");
    crate::saveload::save(&SerializedMap::from(&*goria.read_resource::<Map>()), "map");
}
