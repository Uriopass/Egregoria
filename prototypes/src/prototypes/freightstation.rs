use crate::{get_lua, Money, NoParent, Prototype, PrototypeBase, Size2D};
use mlua::Table;
use std::ops::Deref;

use super::*;

/// FreightStationPrototype is a freight station
#[derive(Clone, Debug)]
pub struct FreightStationPrototype {
    pub base: PrototypeBase,
    pub id: FreightStationPrototypeID,
    pub asset_location: String,
    pub price: Money,
    pub size: Size2D,
}

impl Prototype for FreightStationPrototype {
    type Parent = NoParent;
    type ID = FreightStationPrototypeID;
    const NAME: &'static str = "freight-station";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            asset_location: get_lua(table, "asset_location")?,
            price: get_lua(table, "price")?,
            size: get_lua(table, "size")?,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> Option<&Self::Parent> {
        None
    }
}

impl Deref for FreightStationPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
