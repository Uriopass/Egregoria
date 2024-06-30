use crate::{get_lua, Prototype};
use mlua::Table;
use std::ops::Deref;

use super::*;

#[derive(Clone, Debug)]
pub struct RollingStockPrototype {
    pub base: VehiclePrototype,
    pub id: RollingStockID,
    /// meter
    pub length: f32,
    /// metric ton
    pub mass: u32,
    /// m/s
    pub max_speed: f32,
    /// kN
    pub acc_force: f32,
    /// kN
    pub dec_force: f32,
}

impl Prototype for RollingStockPrototype {
    type Parent = VehiclePrototype;
    type ID = RollingStockID;
    const NAME: &'static str = "rolling-stock";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = VehiclePrototype::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            length: get_lua::<f32>(table, "length")?,
            mass: get_lua(table, "mass")?,
            max_speed: get_lua::<f32>(table, "max_speed")?,
            acc_force: get_lua::<f32>(table, "acc_force")?,
            dec_force: get_lua::<f32>(table, "dec_force")?,
        })
    }
    fn id(&self) -> Self::ID {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        &self.base
    }
}

impl Deref for RollingStockPrototype {
    type Target = VehiclePrototype;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
