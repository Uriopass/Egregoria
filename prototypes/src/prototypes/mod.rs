mod building;       pub use building::*;
mod colors;         pub use colors::*;
mod freightstation; pub use freightstation::*;
mod goods_company;  pub use goods_company::*;
mod item;           pub use item::*;
mod leisure;        pub use leisure::*;
mod road_vehicles;  pub use road_vehicles::*;
mod rolling_stock;  pub use rolling_stock::*;
mod solar;          pub use solar::*;
mod vehicles;       pub use vehicles::*;

crate::gen_prototypes!(
    items:     ItemID              = ItemPrototype,
    buildings: BuildingPrototypeID = BuildingPrototype,
    companies: GoodsCompanyID      = GoodsCompanyPrototype => BuildingPrototypeID,
    leisure:   LeisurePrototypeID  = LeisurePrototype => BuildingPrototypeID,
    solar:     SolarPanelID        = SolarPanelPrototype => GoodsCompanyID,

    vehicles:      VehiclePrototypeID = VehiclePrototype,
    road_vehicles: RoadVehicleID      = RoadVehiclePrototype => VehiclePrototypeID,
    rolling_stock: RollingStockID     = RollingStockPrototype => VehiclePrototypeID,

    colors:    ColorsPrototypeID   = ColorsPrototype,
    stations:  FreightStationPrototypeID = FreightStationPrototype,
);

/** Prototype template. remplace $proto with the root name e.g Item
```rs
use crate::{NoParent, Prototype, PrototypeBase, get_lua};
use mlua::Table;
use std::ops::Deref;

use super::*;

/// $protoPrototype is
#[derive(Clone, Debug)]
pub struct $protoPrototype {
    pub base: $parent,
    pub id: $protoPrototypeID,
}

impl Prototype for $protoPrototype {
    type Parent = $parent;
    type ID = $protoPrototypeID;
    const NAME: &'static str = ;

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = $parent::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> Option<&Self::Parent> {
        Some(&self.base)
    }
}

impl Deref for $protoPrototype {
    type Target = $parent;

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

    fn parent(&self) -> &Self::Parent {
        &crate::NoParent
    }
}
