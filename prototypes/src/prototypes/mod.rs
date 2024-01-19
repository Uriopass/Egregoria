mod company;
mod freightstation;
mod item;
mod solar;

pub use company::*;
pub use freightstation::*;
pub use item::*;
pub use solar::*;

crate::gen_prototypes!(
    companies: GoodsCompanyID = GoodsCompanyPrototype,
    items:     ItemID         = ItemPrototype,
    solar:     SolarPanelID   = SolarPanelPrototype => GoodsCompanyID,
    stations:  FreightStationPrototypeID = FreightStationPrototype,
);

/** Prototype template. remplace $proto with the root name e.g Item
```rs
use crate::{NoParent, Prototype, PrototypeBase};
use mlua::Table;
use std::ops::Deref;

use super::*;

/// $proto is
#[derive(Clone, Debug)]
pub struct $protoPrototype {
    pub base: PrototypeBase,
    pub id: $protoID,
}

impl Prototype for $protoPrototype {
    type Parent = NoParent;
    type ID = $protoID;
    const NAME: &'static str = ;

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }
}

impl Deref for $protoPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
```
*/

#[derive(Debug, Clone, egui_inspect::Inspect)]
pub struct PrototypeBase {
    pub name: String,
    pub order: String,
    pub label: String,
}

impl crate::Prototype for PrototypeBase {
    type Parent = crate::NoParent;
    type ID = ();
    const NAME: &'static str = "base";

    fn from_lua(table: &mlua::Table) -> mlua::Result<Self> {
        use crate::get_lua;
        Ok(Self {
            name: get_lua(table, "name")?,
            order: get_lua(table, "order").unwrap_or(String::new()),
            label: get_lua(table, "label")?,
        })
    }

    fn id(&self) -> Self::ID {}
}
