use crate::economy::{market_update, Government, Market};
use crate::map_dynamic::{
    itinerary_update, routing_changed_system, routing_update_system, BuildingInfos,
    ParkingManagement,
};
use crate::pedestrians::pedestrian_decision_system;
use crate::physics::systems::{coworld_synchronize, kinematics_apply};
use crate::souls::goods_company::{company_system, GoodsCompanyRegistry};
use crate::souls::human::update_decision_system;
use crate::vehicles::systems::{vehicle_decision_system, vehicle_state_update_system};
use crate::{
    utils, CollisionWorld, Egregoria, GameTime, ParCommandBuffer, RandProvider, RunnableSystem,
    RNG_SEED, SECONDS_PER_DAY, SECONDS_PER_HOUR,
};
use common::saveload::Encoder;
use hecs::World;
use map_model::Map;
use resources::Resources;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn init() {
    register_system("update_decision_system", update_decision_system);
    register_system("company_system", company_system);
    register_system("pedestrian_decision_system", pedestrian_decision_system);
    register_system("kinematics_apply", kinematics_apply);
    register_system("coworld_synchronize", coworld_synchronize);
    register_system("vehicle_decision_system", vehicle_decision_system);
    register_system("vehicle_state_update_system", vehicle_state_update_system);
    register_system("routing_changed_system", routing_changed_system);
    register_system("routing_update_system", routing_update_system);
    register_system("itinerary_update", itinerary_update);
    register_system("market_update", market_update);

    register_resource_noserialize::<GoodsCompanyRegistry>();
    register_resource_noserialize::<ParCommandBuffer>();

    register_resource("map", Map::default);
    register_resource("government", Government::default);
    register_resource("market", Market::default);
    register_resource("pmanagement", ParkingManagement::default);
    register_resource("binfos", BuildingInfos::default);
    register_resource("game_time", || {
        GameTime::new(0.0, SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64)
    });
    register_resource("coworld", || CollisionWorld::new(100));
    register_resource("randprovider", || RandProvider::new(RNG_SEED));
}

pub struct InitFunc {
    pub f: Box<dyn Fn(&mut Egregoria) + 'static>,
}

pub(crate) struct SaveLoadFunc {
    pub name: &'static str,
    pub save: Box<dyn Fn(&Egregoria) -> Vec<u8> + 'static>,
    pub load: Box<dyn Fn(&mut Egregoria, Vec<u8>) + 'static>,
}

pub(crate) struct GSystem {
    pub(crate) s: Box<dyn Fn() -> Box<dyn RunnableSystem>>,
}

pub(crate) static mut INIT_FUNCS: Vec<InitFunc> = Vec::new();
pub(crate) static mut SAVELOAD_FUNCS: Vec<SaveLoadFunc> = Vec::new();
pub(crate) static mut GSYSTEMS: Vec<GSystem> = Vec::new();

fn register_system(name: &'static str, s: fn(&mut World, &mut Resources)) {
    unsafe {
        GSYSTEMS.push(GSystem {
            s: Box::new(move || Box::new(utils::scheduler::RunnableFn { f: s, name })),
        });
    }
}

fn register_resource_noserialize<T: 'static + Default + Send + Sync>() {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(|uiw| uiw.insert(T::default())),
        });
    }
}

fn register_resource<T: 'static + Send + Sync + Serialize + DeserializeOwned>(
    name: &'static str,
    initializer: impl Fn() -> T + 'static,
) {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(move |uiw| uiw.insert(initializer())),
        });
        SAVELOAD_FUNCS.push(SaveLoadFunc {
            name,
            save: Box::new(move |uiworld| {
                <common::saveload::Bincode as Encoder>::encode(&*uiworld.read::<T>()).unwrap()
            }),
            load: Box::new(move |uiworld, data| {
                if let Ok(res) = <common::saveload::JSON as Encoder>::decode::<T>(&data) {
                    uiworld.insert(res);
                }
            }),
        });
    }
}
