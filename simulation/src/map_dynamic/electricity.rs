use crate::map::{BuildingKind, ElectricityNetworkID, Map};
use crate::map_dynamic::BuildingInfos;
use crate::utils::resources::Resources;
use crate::{SoulID, World};
use prototypes::Power;
use serde::Deserialize;
use slotmapd::__impl::Serialize;
use std::collections::BTreeMap;

#[derive(Default, Serialize, Deserialize)]
pub struct ElectricityFlow {
    flowmap: BTreeMap<ElectricityNetworkID, NetworkFlow>,
}

impl ElectricityFlow {
    pub fn blackout(&self, network: ElectricityNetworkID) -> bool {
        self.flowmap
            .get(&network)
            .map(|f| f.blackout)
            .unwrap_or(false)
    }

    pub fn network_stats(&self, network: ElectricityNetworkID) -> NetworkFlow {
        self.flowmap.get(&network).cloned().unwrap_or(NetworkFlow {
            consumed_power: Power::ZERO,
            produced_power: Power::ZERO,
            blackout: false,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkFlow {
    pub consumed_power: Power,
    pub produced_power: Power,
    /// Whether the network is in a blackout
    pub blackout: bool,
}

/// Compute the electricity flow of the map and store it in the [`ElectricityFlow`] resource
/// All producing buildings will produce power, and all consuming buildings will consume power
/// If a network produces less power than it consumes, a blackout will occur
pub fn electricity_flow_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("map_dynamic::electricity_flow");

    let map = resources.read::<Map>();
    let binfos = resources.read::<BuildingInfos>();
    let mut flow = resources.write::<ElectricityFlow>();

    flow.flowmap.clear();

    for network in map.electricity.networks.values() {
        let mut consumed_power: Power = Power::ZERO;
        let mut produced_power: Power = Power::ZERO;

        for building in network.buildings.iter() {
            let building = map.buildings.get(*building).unwrap();

            match building.kind {
                BuildingKind::House => {
                    consumed_power += Power::new(100);
                }
                BuildingKind::GoodsCompany(comp) => {
                    let proto = comp.prototype();

                    let Some(SoulID::GoodsCompany(owner)) = binfos.owner(building.id) else {
                        continue;
                    };

                    let Some(ent) = world.companies.get(owner) else {
                        continue;
                    };
                    let productivity = ent.raw_productivity(proto, building.zone.as_ref()) as f64;

                    consumed_power += proto.power_consumption.unwrap_or(Power::ZERO) * productivity;
                    produced_power += proto.power_production.unwrap_or(Power::ZERO) * productivity;
                }
                BuildingKind::RailFreightStation(_) => {}
                BuildingKind::TrainStation => {}
                BuildingKind::ExternalTrading => {}
            }
        }

        flow.flowmap.insert(
            network.id,
            NetworkFlow {
                consumed_power,
                produced_power,
                blackout: consumed_power > produced_power,
            },
        );
    }
}
