use std::ops::Deref;

use mlua::{FromLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};

use egui_inspect::Inspect;

use crate::{get_lua, get_lua_opt, BuildingPrototype, GoodsCompanyID, Prototype, Recipe, Zone};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Inspect)]
pub enum CompanyKind {
    /// Buyers come to get their goods
    Store,
    /// Buyers get their goods delivered to them
    Factory,
}

#[derive(Debug, Clone)]
pub struct GoodsCompanyPrototype {
    pub base: BuildingPrototype,
    pub id: GoodsCompanyID,
    pub kind: CompanyKind,
    pub recipe: Option<Recipe>,
    pub n_trucks: u32,
    pub n_workers: u32,
    pub zone: Option<Zone>,
}

impl Prototype for GoodsCompanyPrototype {
    type Parent = BuildingPrototype;
    type ID = GoodsCompanyID;
    const NAME: &'static str = "goods-company";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = BuildingPrototype::from_lua(table)?;
        Ok(Self {
            id: Self::ID::from(&base.name),
            base,
            kind: get_lua(table, "kind")?,
            recipe: get_lua(table, "recipe")?,
            n_trucks: get_lua_opt(table, "n_trucks")?.unwrap_or(0),
            n_workers: get_lua_opt(table, "n_workers")?.unwrap_or(0),
            zone: get_lua(table, "zone").ok(),
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &self.base
    }
}

impl Deref for GoodsCompanyPrototype {
    type Target = BuildingPrototype;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> FromLua<'a> for CompanyKind {
    fn from_lua(value: Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
        let s: String = FromLua::from_lua(value, lua)?;
        match &*s {
            "store" => Ok(Self::Store),
            "factory" => Ok(Self::Factory),
            _ => Err(mlua::Error::external(format!(
                "Unknown company kind: {}",
                s
            ))),
        }
    }
}
