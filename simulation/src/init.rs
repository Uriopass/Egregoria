use serde::de::DeserializeOwned;
use serde::Serialize;

#[allow(unused_imports)]
use common::saveload::{Bincode, Encoder, JSONPretty, JSON};
use prototypes::{GameTime, Tick};

use crate::economy::{market_update, EcoStats, Government, Market};
use crate::map::Map;
use crate::map_dynamic::{
    dispatch_system, electricity_flow_system, itinerary_update, routing_changed_system,
    routing_update_system, BuildingInfos, Dispatcher, ElectricityFlow, ParkingManagement,
};
use crate::multiplayer::MultiplayerState;
use crate::souls::freight_station::freight_station_system;
use crate::souls::goods_company::company_system;
use crate::souls::human::update_decision_system;
use crate::transportation::pedestrian_decision_system;
use crate::transportation::road::{vehicle_decision_system, vehicle_state_update_system};
use crate::transportation::testing_vehicles::{random_vehicles_update, RandomVehicles};
use crate::transportation::train::{
    locomotive_system, train_reservations_update, TrainReservations,
};
use crate::transportation::{transport_grid_synchronize, TransportGrid};
use crate::utils::resources::Resources;
use crate::world::{CompanyEnt, FreightStationEnt, HumanEnt, TrainEnt, VehicleEnt, WagonEnt};
use crate::World;
use crate::{
    add_souls_to_empty_buildings, utils, ParCommandBuffer, RandProvider, Replay, RunnableSystem,
    Simulation, SimulationOptions, RNG_SEED,
};

pub fn init() {
    //crate::rerun::init_rerun();

    // # Safety
    // This function is called only once, before any other function in this crate.
    unsafe {
        #[cfg(not(test))]
        let base = "./";
        #[cfg(test)]
        let base = "../";

        match prototypes::load_prototypes(base) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error loading prototypes: {}", e)
            }
        }
    }

    register_system("electricity_flow_system", electricity_flow_system);
    register_system("dispatch_system", dispatch_system);
    register_system("update_decision_system", update_decision_system);
    register_system("company_system", company_system);
    register_system("pedestrian_decision_system", pedestrian_decision_system);
    register_system("transport_grid_synchronize", transport_grid_synchronize);
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
    register_system("update_map", |_, res| res.write::<Map>().update());

    register_system_sim("add_souls_to_empty_buildings", add_souls_to_empty_buildings);

    register_resource_noserialize::<ParCommandBuffer<VehicleEnt>>();
    register_resource_noserialize::<ParCommandBuffer<TrainEnt>>();
    register_resource_noserialize::<ParCommandBuffer<HumanEnt>>();
    register_resource_noserialize::<ParCommandBuffer<WagonEnt>>();
    register_resource_noserialize::<ParCommandBuffer<FreightStationEnt>>();
    register_resource_noserialize::<ParCommandBuffer<CompanyEnt>>();
    register_resource_noinit::<SimulationOptions, Bincode>("simoptions");

    register_resource_default::<ElectricityFlow, Bincode>("electricity_flow");
    register_resource_default::<Market, Bincode>("market");
    register_resource_default::<EcoStats, Bincode>("ecostats");
    register_resource_default::<MultiplayerState, Bincode>("multiplayer_state");
    register_resource_default::<RandomVehicles, Bincode>("random_vehicles");
    register_resource_default::<Map, Bincode>("map");
    register_resource_default::<TrainReservations, Bincode>("train_reservations");
    register_resource_default::<Government, Bincode>("government");
    register_resource_default::<ParkingManagement, Bincode>("pmanagement");
    register_resource_default::<BuildingInfos, Bincode>("binfos");
    register_resource::<GameTime, Bincode>("game_time", || GameTime::new(Tick(1)));
    register_resource::<TransportGrid, Bincode>("transport_grid", || TransportGrid::new(100));
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

/*fn register_init(s: fn(&mut World, &mut Resources)) {
    unsafe {
        INIT_FUNCS.push(InitFunc {
            f: Box::new(move |sim| s(&mut sim.world, &mut sim.resources)),
        });
    }
}*/

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
