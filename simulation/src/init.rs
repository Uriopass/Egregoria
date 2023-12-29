use crate::economy::{init_market, market_update, EcoStats, Government, ItemRegistry, Market};
use crate::map::Map;
use crate::map_dynamic::{
    dispatch_system, itinerary_update, routing_changed_system, routing_update_system,
    BuildingInfos, Dispatcher, ParkingManagement,
};
use crate::multiplayer::MultiplayerState;
use crate::physics::coworld_synchronize;
use crate::souls::freight_station::freight_station_system;
use crate::souls::goods_company::{company_system, GoodsCompanyRegistry};
use crate::souls::human::update_decision_system;
use crate::transportation::pedestrian_decision_system;
use crate::transportation::road::{vehicle_decision_system, vehicle_state_update_system};
use crate::transportation::testing_vehicles::{random_vehicles_update, RandomVehicles};
use crate::transportation::train::{
    locomotive_system, train_reservations_update, TrainReservations,
};
use crate::utils::resources::Resources;
use crate::utils::time::Tick;
use crate::wildlife::add_flocks_randomly;
use crate::wildlife::bird::bird_decision_system;
use crate::world::{CompanyEnt, FreightStationEnt, HumanEnt, TrainEnt, VehicleEnt, WagonEnt};
use crate::World;
use crate::{
    add_souls_to_empty_buildings, utils, CollisionWorld, GameTime, ParCommandBuffer, RandProvider,
    Replay, RunnableSystem, Simulation, SimulationOptions, RNG_SEED, SECONDS_PER_DAY,
    SECONDS_PER_HOUR,
};
use common::saveload::{Bincode, Encoder, JSON};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn init() {
    register_system("dispatch_system", dispatch_system);
    register_system("update_decision_system", update_decision_system);
    register_system("company_system", company_system);
    register_system("pedestrian_decision_system", pedestrian_decision_system);
    register_system("bird_decision_system", bird_decision_system);
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
    register_system("random_vehicles", random_vehicles_update);

    register_system_sim("add_souls_to_empty_buildings", add_souls_to_empty_buildings);
    register_system_sim("add_flocks_randomly", add_flocks_randomly);

    register_resource_noserialize::<GoodsCompanyRegistry>();
    register_resource_noserialize::<ItemRegistry>();
    register_resource_noserialize::<ParCommandBuffer<VehicleEnt>>();
    register_resource_noserialize::<ParCommandBuffer<TrainEnt>>();
    register_resource_noserialize::<ParCommandBuffer<HumanEnt>>();
    register_resource_noserialize::<ParCommandBuffer<WagonEnt>>();
    register_resource_noserialize::<ParCommandBuffer<FreightStationEnt>>();
    register_resource_noserialize::<ParCommandBuffer<CompanyEnt>>();
    register_resource_noinit::<Market, Bincode>("market");
    register_resource_noinit::<EcoStats, Bincode>("ecostats");
    register_resource_noinit::<SimulationOptions, Bincode>("simoptions");

    register_init(init_market);

    register_resource_default::<MultiplayerState, Bincode>("multiplayer_state");
    register_resource_default::<RandomVehicles, Bincode>("random_vehicles");
    register_resource_default::<Tick, Bincode>("tick");
    register_resource_default::<Map, Bincode>("map");
    register_resource_default::<TrainReservations, Bincode>("train_reservations");
    register_resource_default::<Government, Bincode>("government");
    register_resource_default::<ParkingManagement, Bincode>("pmanagement");
    register_resource_default::<BuildingInfos, Bincode>("binfos");
    register_resource::<GameTime, Bincode>("game_time", || {
        GameTime::new(0.0, SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64)
    });
    register_resource::<CollisionWorld, Bincode>("coworld", || CollisionWorld::new(100));
    register_resource::<RandProvider, Bincode>("randprovider", || RandProvider::new(RNG_SEED));
    register_resource_default::<Dispatcher, Bincode>("dispatcher");
    register_resource_default::<Replay, JSON>("replay");
}

pub struct InitFunc {
    pub f: Box<dyn Fn(&mut Simulation) + 'static>,
}

pub(crate) struct SaveLoadFunc {
    pub name: &'static str,
    pub save: Box<dyn Fn(&Simulation) -> Vec<u8> + 'static>,
    pub load: Box<dyn Fn(&mut Simulation, Vec<u8>) + 'static>,
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
            f: Box::new(move |sim| s(&mut sim.world, &mut sim.resources)),
        });
    }
}

fn register_system(name: &'static str, s: fn(&mut World, &mut Resources)) {
    unsafe {
        GSYSTEMS.push(GSystem {
            s: Box::new(move || {
                Box::new(utils::scheduler::RunnableFn {
                    f: move |sim| s(&mut sim.world, &mut sim.resources),
                    name,
                })
            }),
        });
    }
}

fn register_system_sim(name: &'static str, s: fn(&mut Simulation)) {
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

fn register_resource_default<
    T: 'static + Send + Sync + Serialize + DeserializeOwned + Default,
    E: Encoder,
>(
    name: &'static str,
) {
    register_resource::<T, E>(name, T::default);
}

fn register_resource<T: 'static + Send + Sync + Serialize + DeserializeOwned, E: Encoder>(
    name: &'static str,
    initializer: impl Fn() -> T + 'static,
) {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(move |uiw| uiw.insert(initializer())),
        });
        register_resource_noinit::<T, E>(name);
    }
}

fn register_resource_noinit<T: 'static + Send + Sync + Serialize + DeserializeOwned, E: Encoder>(
    name: &'static str,
) {
    unsafe {
        SAVELOAD_FUNCS.push(SaveLoadFunc {
            name,
            save: Box::new(move |uiworld| E::encode(&*uiworld.read::<T>()).unwrap()),
            load: Box::new(move |uiworld, data| match E::decode::<T>(&data) {
                Ok(res) => {
                    uiworld.insert(res);
                }
                Err(e) => {
                    log::error!("Error loading resource {}: {}", name, e);
                }
            }),
        });
    }
}
