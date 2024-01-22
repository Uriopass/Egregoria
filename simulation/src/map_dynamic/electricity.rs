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
    pub fn productivity(&self, network: ElectricityNetworkID) -> f32 {
        self.flowmap
            .get(&network)
            .map(|f| f.productivity)
            .unwrap_or(1.0)
    }

    pub fn network_stats(&self, network: ElectricityNetworkID) -> NetworkFlow {
        self.flowmap.get(&network).cloned().unwrap_or(NetworkFlow {
            consumed_power: Power::ZERO,
            produced_power: Power::ZERO,
            productivity: 1.0,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkFlow {
    pub consumed_power: Power,
    pub produced_power: Power,
    /// The productivity of the network, between 0 and 1
    /// Ratio of the power produced by the network compared to the power consumed capped to 1
    pub productivity: f32,
}

/// Compute the electricity flow of the map and store it in the [`ElectricityFlow`] resource
/// All producing buildings will produce power, and all consuming buildings will consume power
/// The productivity of the network is the ratio of the two
/// Buildings can then use this productivity to scale how much they work
pub fn electricity_flow_system(world: &mut World, resources: &mut Resources) {
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
                productivity: if consumed_power == Power::ZERO {
                    1.0
                } else {
                    f64::min(produced_power.0 as f64 / consumed_power.0 as f64, 1.0) as f32
                },
            },
        );
    }
}
