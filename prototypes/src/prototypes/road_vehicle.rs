use crate::{get_lua, Prototype};

use mlua::Table;
use std::ops::Deref;

use super::*;

#[derive(Clone, Debug)]
pub struct RoadVehiclePrototype {
    pub base: VehiclePrototype,
    pub id: RoadVehicleID,
    /// m/s
    pub max_speed: f32,
    /// m.s^2
    pub acceleration: f32,
    /// m.s^2
    pub deceleration: f32,
}

impl Prototype for RoadVehiclePrototype {
    type Parent = VehiclePrototype;
    type ID = RoadVehicleID;
    const NAME: &'static str = "road-vehicle";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = VehiclePrototype::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            max_speed: get_lua::<f32>(table, "max_speed")?,
            acceleration: get_lua::<f32>(table, "acceleration")?,
            deceleration: get_lua::<f32>(table, "deceleration")?,
        })
    }
    fn id(&self) -> Self::ID {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        &self.base
    }
}

impl Deref for RoadVehiclePrototype {
    type Target = VehiclePrototype;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
