use std::fmt::{Debug, Formatter};

use flat_spatial::grid::GridHandle;
use serde::{Deserialize, Serialize};

use egui_inspect::InspectVec2Rotation;
use geom::{Transform, Vec2};
pub use pedestrian::*;
pub use vehicle::*;

use crate::map::BuildingID;
use crate::utils::resources::Resources;
use crate::world::VehicleID;
use crate::{Simulation, World};

pub mod pedestrian;
pub mod road;
pub mod testing_vehicles;
pub mod train;
mod vehicle;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Outside,
    Vehicle(VehicleID),
    Building(BuildingID),
}
debug_inspect_impl!(Location);

#[derive(Clone, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Speed(pub f32);
debug_inspect_impl!(Speed);

impl Debug for Speed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}m/s", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Inspect)]
pub enum TransportationGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

#[derive(Copy, Clone, Serialize, Deserialize, Inspect)]
pub struct TransportState {
    #[inspect(proxy_type = "InspectVec2Rotation")]
    pub dir: Vec2,
    pub speed: f32,
    pub radius: f32,
    pub height: f32,
    pub group: TransportationGroup,
    pub flag: u64,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            dir: Vec2::X,
            speed: 0.0,
            radius: 1.0,
            height: 0.0,
            group: TransportationGroup::Unknown,
            flag: 0,
        }
    }
}

pub type TransportGrid = flat_spatial::Grid<TransportState, Vec2>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Transporter(pub GridHandle);
debug_inspect_impl!(Transporter);

impl Transporter {
    pub fn destroy(self) -> impl FnOnce(&mut Simulation) {
        move |sim| {
            let cw = &mut sim.write::<TransportGrid>();
            cw.remove_maintain(self.0);
        }
    }
}

pub fn transport_grid_synchronize(world: &mut World, resources: &mut Resources) {
    profiling::scope!("physics::transport_grid_synchronize");
    let mut transport_grid = resources.write::<TransportGrid>();

    world.query_trans_speed_coll_vehicle().for_each(
        |(trans, kin, coll, v): (&Transform, &Speed, Transporter, Option<&Vehicle>)| {
            transport_grid.set_position(coll.0, trans.pos.xy());
            let (_, po) = transport_grid.get_mut(coll.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
            po.dir = trans.dir.xy();
            po.speed = kin.0;
            po.height = trans.pos.z;
            if let Some(v) = v {
                po.flag = v.flag;
            }
        },
    );

    transport_grid.maintain_deterministic();
}
