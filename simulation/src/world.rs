use crate::economy::{Bought, Market, Sold, Workers};
use crate::map_dynamic::{
    DispatchID, Dispatcher, Itinerary, ItineraryFollower, ItineraryLeader, ParkingManagement,
    Router,
};
use crate::souls::desire::{BuyFood, Home, Work};
use crate::souls::freight_station::FreightStation;
use crate::souls::goods_company::GoodsCompanyState;
use crate::souls::human::{HumanDecision, PersonalInfo};
use crate::transportation::train::{Locomotive, LocomotiveReservation, RailWagon};
use crate::transportation::{
    Location, Pedestrian, Speed, TransportGrid, Transporter, Vehicle, VehicleKind, VehicleState,
};
use crate::utils::par_command_buffer::SimDrop;
use crate::utils::resources::Resources;
use crate::{impl_entity, impl_trans, SoulID};
use common::iter::chain;
use derive_more::{From, TryInto};
use geom::{Transform, Vec2, Vec3};
use serde::Deserialize;
use slotmapd::__impl::Serialize;
use slotmapd::{new_key_type, HopSlotMap};
use std::fmt::{Display, Formatter};

new_key_type! {
    pub struct VehicleID;
    pub struct TrainID;
    pub struct HumanID;
    pub struct WagonID;
    pub struct FreightStationID;
    pub struct CompanyID;
}

impl_entity!(VehicleID, VehicleEnt, vehicles);
impl_entity!(HumanID, HumanEnt, humans);
impl_entity!(TrainID, TrainEnt, trains);
impl_entity!(WagonID, WagonEnt, wagons);
impl_entity!(FreightStationID, FreightStationEnt, freight_stations);
impl_entity!(CompanyID, CompanyEnt, companies);

impl_trans!(HumanID);
impl_trans!(VehicleID);
impl_trans!(TrainID);
impl_trans!(WagonID);
impl_trans!(FreightStationID);
impl_trans!(CompanyID);

#[derive(PartialEq, Eq, Copy, Clone, Debug, From, TryInto)]
pub enum AnyEntity {
    VehicleID(VehicleID),
    TrainID(TrainID),
    WagonID(WagonID),
    FreightStationID(FreightStationID),
    CompanyID(CompanyID),
    HumanID(HumanID),
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct VehicleEnt {
    pub trans: Transform,
    pub speed: Speed,
    pub vehicle: Vehicle,
    pub it: Itinerary,
    pub collider: Option<Transporter>,
}

impl SimDrop for VehicleEnt {
    fn sim_drop(mut self, id: VehicleID, res: &mut Resources) {
        if let Some(collider) = self.collider {
            res.write::<TransportGrid>().remove_maintain(collider.0);
        }

        if let VehicleState::Parked(resa) | VehicleState::RoadToPark(_, _, resa) =
            std::mem::replace(&mut self.vehicle.state, VehicleState::Driving)
        {
            res.write::<ParkingManagement>().free(resa);
        }

        if matches!(self.vehicle.kind, VehicleKind::Truck) {
            res.write::<Dispatcher>()
                .unregister(DispatchID::SmallTruck(id))
        }
    }
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct HumanEnt {
    pub trans: Transform,
    pub speed: Speed,
    pub location: Location,
    pub pedestrian: Pedestrian,
    pub collider: Option<Transporter>,

    pub router: Router,
    pub it: Itinerary,

    pub decision: HumanDecision,
    pub home: Home,
    pub food: BuyFood,
    pub bought: Bought,
    pub work: Option<Work>,

    pub personal_info: Box<PersonalInfo>,
}

impl SimDrop for HumanEnt {
    fn sim_drop(mut self, id: HumanID, res: &mut Resources) {
        if let Some(collider) = self.collider {
            res.write::<TransportGrid>().remove_maintain(collider.0);
        }

        res.write::<Market>().remove(SoulID::Human(id));

        self.router
            .clear_steps(&mut res.write::<ParkingManagement>())
    }
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct TrainEnt {
    pub trans: Transform,
    pub speed: Speed,
    pub it: Itinerary,
    pub locomotive: Locomotive,
    pub res: LocomotiveReservation,
    #[inspect(skip)]
    pub leader: ItineraryLeader,
}

impl SimDrop for TrainEnt {
    fn sim_drop(self, id: TrainID, res: &mut Resources) {
        res.write::<Dispatcher>()
            .unregister(DispatchID::FreightTrain(id));
    }
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct WagonEnt {
    pub trans: Transform,
    pub speed: Speed,
    pub wagon: RailWagon,
    pub itfollower: ItineraryFollower,
}

impl SimDrop for WagonEnt {
    fn sim_drop(self, _: WagonID, _: &mut Resources) {}
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct FreightStationEnt {
    pub trans: Transform,
    pub f: FreightStation,
}

impl SimDrop for FreightStationEnt {
    fn sim_drop(self, id: FreightStationID, res: &mut Resources) {
        res.write::<Market>().remove(SoulID::FreightStation(id));

        let mut d = res.write::<Dispatcher>();
        for (id, _) in self.f.trains {
            d.free(id);
        }
        drop(d);
    }
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct CompanyEnt {
    pub trans: Transform,
    pub comp: GoodsCompanyState,
    pub workers: Workers,
    pub sold: Sold,
    pub bought: Bought,
}

impl SimDrop for CompanyEnt {
    fn sim_drop(self, id: CompanyID, res: &mut Resources) {
        res.write::<Market>().remove(SoulID::GoodsCompany(id));
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct World {
    pub vehicles: HopSlotMap<VehicleID, VehicleEnt>,
    pub humans: HopSlotMap<HumanID, HumanEnt>,
    pub trains: HopSlotMap<TrainID, TrainEnt>,
    pub wagons: HopSlotMap<WagonID, WagonEnt>,
    pub freight_stations: HopSlotMap<FreightStationID, FreightStationEnt>,
    pub companies: HopSlotMap<CompanyID, CompanyEnt>,
}

impl World {
    pub fn get<E: EntityID>(&self, id: E) -> Option<&E::Entity> {
        <<E as EntityID>::Entity as Entity>::storage(self).get(id)
    }

    pub fn storage<E: Entity>(&self) -> &HopSlotMap<E::ID, E> {
        E::storage(self)
    }

    pub fn storage_id<E: EntityID>(&self, _: E) -> &HopSlotMap<E, E::Entity> {
        E::Entity::storage(self)
    }

    pub fn insert<E: Entity>(&mut self, e: E) -> E::ID {
        E::storage_mut(self).insert(e)
    }

    pub fn contains(&self, id: AnyEntity) -> bool {
        match id {
            AnyEntity::VehicleID(id) => self.storage_id(id).contains_key(id),
            AnyEntity::TrainID(id) => self.storage_id(id).contains_key(id),
            AnyEntity::WagonID(id) => self.storage_id(id).contains_key(id),
            AnyEntity::FreightStationID(id) => self.storage_id(id).contains_key(id),
            AnyEntity::CompanyID(id) => self.storage_id(id).contains_key(id),
            AnyEntity::HumanID(id) => self.storage_id(id).contains_key(id),
        }
    }

    pub fn pos_any(&self, id: AnyEntity) -> Option<Vec3> {
        match id {
            AnyEntity::VehicleID(x) => self.pos(x),
            AnyEntity::TrainID(x) => self.pos(x),
            AnyEntity::WagonID(x) => self.pos(x),
            AnyEntity::HumanID(x) => self.pos(x),
            _ => None,
        }
    }

    pub fn it_any(&self, id: AnyEntity) -> Option<&Itinerary> {
        match id {
            AnyEntity::VehicleID(x) => Some(&self.get(x)?.it),
            AnyEntity::TrainID(x) => Some(&self.get(x)?.it),
            AnyEntity::HumanID(x) => Some(&self.get(x)?.it),
            _ => None,
        }
    }

    pub fn pos<E: WorldTransform>(&self, id: E) -> Option<Vec3> {
        self.get(id).map(|x| E::trans(x).pos)
    }

    pub fn trans<E: WorldTransform>(&self, id: E) -> Option<Transform> {
        self.get(id).map(|x| E::trans(x))
    }

    #[rustfmt::skip]
    pub fn query_trans_itin(&self) -> impl Iterator<Item = (AnyEntity, (&Transform, &Itinerary))> + '_ {
        chain((
            self.humans  .iter().map(|(id, x)| (AnyEntity::HumanID(id), (&x.trans, &x.it))),
            self.vehicles.iter().map(|(id, x)| (AnyEntity::VehicleID(id), (&x.trans, &x.it))),
            self.trains  .iter().map(|(id, x)| (AnyEntity::TrainID(id), (&x.trans, &x.it))),
        ))
    }

    #[rustfmt::skip]
    pub fn query_selectable_pos(&self) -> impl Iterator<Item = (AnyEntity, Vec2)> + '_ {
        chain((
            self.humans  .iter().map(|(id, x)| (AnyEntity::HumanID(id), x.trans.pos.xy())),
            self.vehicles.iter().map(|(id, x)| (AnyEntity::VehicleID(id), x.trans.pos.xy())),
            self.trains  .iter().map(|(id, x)| (AnyEntity::TrainID(id), x.trans.pos.xy())),
            self.wagons  .iter().map(|(id, x)| (AnyEntity::WagonID(id), x.trans.pos.xy())),
        ))
    }

    #[rustfmt::skip]
    pub fn query_it_trans_speed(
        &mut self,
    ) -> impl Iterator<Item = (&mut Itinerary, &mut Transform, f32)> + '_ {
        chain((
            self.humans  .values_mut().map(|h| (&mut h.it, &mut h.trans, h.speed.0)),
            self.trains  .values_mut().map(|h| (&mut h.it, &mut h.trans, h.speed.0)),
            self.vehicles.values_mut().map(|h| (&mut h.it, &mut h.trans, h.speed.0)),
        ))
    }


    #[rustfmt::skip]
    pub fn query_trans_speed_coll_vehicle(
        &self,
    ) -> impl Iterator<Item = (&Transform, &Speed, Transporter, Option<&Vehicle>)> {
        chain((
              self.vehicles.values().filter_map(|x| { x.collider.map(|coll| (&x.trans, &x.speed, coll, Some(&x.vehicle))) }),
              self.humans  .values().filter_map(|x| { x.collider.map(|coll| (&x.trans, &x.speed, coll, None)) }),
        ))
    }

    pub fn entities(&self) -> impl Iterator<Item = AnyEntity> + '_ {
        chain((
            chain((
                self.humans.keys().map(AnyEntity::HumanID),
                self.vehicles.keys().map(AnyEntity::VehicleID),
                self.trains.keys().map(AnyEntity::TrainID),
                self.wagons.keys().map(AnyEntity::WagonID),
            )),
            chain((
                self.freight_stations
                    .keys()
                    .map(AnyEntity::FreightStationID),
                self.companies.keys().map(AnyEntity::CompanyID),
            )),
        ))
    }
}

/// A trait that describes an entity, therefore having storage within the world
pub trait Entity: 'static + Sized + Send {
    type ID: EntityID<Entity = Self>;

    fn storage(w: &World) -> &HopSlotMap<Self::ID, Self>;
    fn storage_mut(w: &mut World) -> &mut HopSlotMap<Self::ID, Self>;
}

/// A trait that describes an entity id to be able to find an Entity from an ID
pub trait EntityID: 'static + slotmapd::Key + Send {
    type Entity: Entity<ID = Self>;
}

/// A trait that describes an entity having a position within the world
pub trait WorldTransform: EntityID {
    fn trans(obj: &Self::Entity) -> Transform;
}

mod macros {
    #[macro_export]
    macro_rules! impl_trans {
        ($t:ty) => {
            impl WorldTransform for $t {
                fn trans(obj: &Self::Entity) -> Transform {
                    obj.trans
                }
            }
        };
    }

    #[macro_export]
    macro_rules! impl_entity {
        ($id:ty, $obj:ty, $s:ident) => {
            debug_inspect_impl!($id);

            impl Entity for $obj {
                type ID = $id;

                fn storage(w: &World) -> &HopSlotMap<Self::ID, Self> {
                    &w.$s
                }

                fn storage_mut(w: &mut World) -> &mut HopSlotMap<Self::ID, Self> {
                    &mut w.$s
                }
            }

            impl EntityID for $id {
                type Entity = $obj;
            }
        };
    }
}

impl Display for AnyEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyEntity::HumanID(id) => write!(f, "{:?}", id),
            AnyEntity::VehicleID(id) => write!(f, "{:?}", id),
            AnyEntity::TrainID(id) => write!(f, "{:?}", id),
            AnyEntity::WagonID(id) => write!(f, "{:?}", id),
            AnyEntity::FreightStationID(id) => write!(f, "{:?}", id),
            AnyEntity::CompanyID(id) => write!(f, "{:?}", id),
        }
    }
}
