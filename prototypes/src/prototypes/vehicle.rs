use crate::{get_lua, Money, NoParent, Prototype, PrototypeBase, RenderAsset};
use mlua::Table;
use std::ops::Deref;

use super::*;

#[derive(Clone, Debug)]
pub struct VehiclePrototype {
    pub base: PrototypeBase,
    pub id: VehiclePrototypeID,
    pub asset: RenderAsset,
    pub price: Money,
}

impl Prototype for VehiclePrototype {
    type Parent = NoParent;
    type ID = VehiclePrototypeID;
    const NAME: &'static str = "vehicle";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            asset: get_lua(table, "asset")?,
            price: get_lua(table, "price")?,
        })
    }
    fn id(&self) -> Self::ID {
        self.id
    }
    fn parent(&self) -> &Self::Parent {
        &NoParent
    }
}

impl Deref for VehiclePrototype {
    type Target = PrototypeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
