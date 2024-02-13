use crate::{get_lua, Money, Prototype, RecTimeInterval};
use mlua::Table;
use std::ops::Deref;

use super::*;

/// LeisurePrototype is a building where people can go to relax
#[derive(Clone, Debug)]
pub struct LeisurePrototype {
    pub base: BuildingPrototype,
    pub id: LeisurePrototypeID,
    pub opening_hours: RecTimeInterval,
    pub capacity: u32,
    pub entry_fee: Money,
}

impl Prototype for LeisurePrototype {
    type Parent = BuildingPrototype;
    type ID = LeisurePrototypeID;
    const NAME: &'static str = "leisure";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = BuildingPrototype::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            opening_hours: get_lua(table, "opening_hours")?,
            capacity: get_lua(table, "capacity")?,
            entry_fee: get_lua(table, "entry_fee")?,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &self.base
    }
}

impl Deref for LeisurePrototype {
    type Target = BuildingPrototype;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
