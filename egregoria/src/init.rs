use crate::economy::{init_market, market_update, EcoStats, Government, ItemRegistry, Market};
use crate::map::Map;
use crate::map_dynamic::{
    dispatch_system, itinerary_update, routing_changed_system, routing_update_system,
    BuildingInfos, Dispatcher, ParkingManagement,
};
use crate::pedestrians::pedestrian_decision_system;
use crate::physics::systems::coworld_synchronize;
use crate::souls::fret_station::freight_station_system;
use crate::souls::goods_company::{company_system, GoodsCompanyRegistry};
use crate::souls::human::update_decision_system;
use crate::utils::time::Tick;
use crate::vehicles::systems::{vehicle_decision_system, vehicle_state_update_system};
use crate::vehicles::trains::{locomotive_system, train_reservations_update, TrainReservations};
use crate::{
    add_souls_to_empty_buildings, utils, CollisionWorld, Egregoria, GameTime, ParCommandBuffer,
    RandProvider, Replay, RunnableSystem, RNG_SEED, SECONDS_PER_DAY, SECONDS_PER_HOUR,
};
use common::saveload::Encoder;
use hecs::World;
use resources::Resources;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn init() {
    register_system("dispatch_system", dispatch_system);
    register_system("update_decision_system", update_decision_system);
    register_system("company_system", company_system);
    register_system("pedestrian_decision_system", pedestrian_decision_system);
    register_system("coworld_synchronize", coworld_synchronize);
    register_system("locomotive_system", locomotive_system);
    register_system("vehicle_decision_system", vehicle_decision_system);
    register_system("vehicle_state_update_system", vehicle_state_update_system);
    register_system("routing_changed_system", routing_changed_system);
    register_system("routing_update_system", routing_update_system);
    register_system("itinerary_update", itinerary_update);
    register_system("market_update", market_update);
    register_system("train_reservations_update", train_reservations_update);
    register_system("freight_station", freight_station_system);

    register_system_goria("add_souls_to_empty_buildings", add_souls_to_empty_buildings);

    register_resource_noserialize::<GoodsCompanyRegistry>();
    register_resource_noserialize::<ItemRegistry>();
    register_resource_noserialize::<ParCommandBuffer>();
    register_resource_noinit::<Market>("market");
    register_resource_noinit::<EcoStats>("ecostats");

    register_init(init_market);

    register_resource("tick", Tick::default);
    register_resource("map", Map::default);
    register_resource("train_reservations", TrainReservations::default);
    register_resource("government", Government::default);
    register_resource("pmanagement", ParkingManagement::default);
    register_resource("binfos", BuildingInfos::default);
    register_resource("game_time", || {
        GameTime::new(0.0, SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64)
    });
    register_resource("coworld", || CollisionWorld::new(100));
    register_resource("randprovider", || RandProvider::new(RNG_SEED));
    register_resource("dispatcher", || Dispatcher::default());
    register_resource("replay", || Replay::default());
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

fn register_init(s: fn(&mut World, &mut Resources)) {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(move |goria| s(&mut goria.world, &mut goria.resources)),
        });
    }
}

fn register_system(name: &'static str, s: fn(&mut World, &mut Resources)) {
    unsafe {
        GSYSTEMS.push(GSystem {
            s: Box::new(move || {
                Box::new(utils::scheduler::RunnableFn {
                    f: move |goria| s(&mut goria.world, &mut goria.resources),
                    name,
                })
            }),
        });
    }
}

fn register_system_goria(name: &'static str, s: fn(&mut Egregoria)) {
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
        register_resource_noinit::<T>(name);
    }
}

fn register_resource_noinit<T: 'static + Send + Sync + Serialize + DeserializeOwned>(
    name: &'static str,
) {
    unsafe {
        SAVELOAD_FUNCS.push(SaveLoadFunc {
            name,
            save: Box::new(move |uiworld| {
                <common::saveload::Bincode as Encoder>::encode(&*uiworld.read::<T>()).unwrap()
            }),
            load: Box::new(move |uiworld, data| {
                if let Ok(res) = <common::saveload::Bincode as Encoder>::decode::<T>(&data) {
                    uiworld.insert(res);
                }
            }),
        });
    }
}
