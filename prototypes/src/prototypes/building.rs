use crate::{
    get_lua, get_v2, Money, NoParent, Power, Prototype, PrototypeBase, RenderAsset, Size2D,
};
use egui_inspect::debug_inspect_impl;
use geom::Vec2;
use mlua::{FromLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use super::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BuildingGen {
    House,
    Farm,
    CenteredDoor {
        vertical_factor: f32, // 1.0 means that the door is at the bottom, just on the street
    },
    NoWalkway {
        door_pos: Vec2, // door_pos is relative to the center of the building
    },
}
debug_inspect_impl!(BuildingGen);

/// BuildingPrototype is a building
#[derive(Clone, Debug)]
pub struct BuildingPrototype {
    pub base: PrototypeBase,
    pub id: BuildingPrototypeID,
    pub size: Size2D,
    pub bgen: BuildingGen,
    pub asset: RenderAsset,
    pub price: Money,
    pub power_consumption: Option<Power>,
    pub power_production: Option<Power>,
}

impl Prototype for BuildingPrototype {
    type Parent = NoParent;
    type ID = BuildingPrototypeID;
    const NAME: &'static str = "building";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            bgen: get_lua(table, "bgen")?,
            size: get_lua(table, "size")?,
            asset: get_lua(table, "asset")?,
            price: get_lua(table, "price")?,
            power_consumption: get_lua(table, "power_consumption")?,
            power_production: get_lua(table, "power_production")?,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &NoParent
    }
}

impl Deref for BuildingPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> FromLua<'a> for BuildingGen {
    fn from_lua(value: Value<'a>, _: &'a Lua) -> mlua::Result<Self> {
        let table = match value {
            Value::String(s) => {
                let s = s.to_str()?;
                return match s {
                    "house" => Ok(Self::House),
                    "farm" => Ok(Self::Farm),
                    _ => Err(mlua::Error::external(format!(
                        "Unknown building gen kind: {}",
                        s
                    ))),
                };
            }
            Value::Table(t) => t,
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "BuildingGen",
                message: Some("expected string or table".into()),
            })?,
        };
        let kind = get_lua::<String>(&table, "kind")?;
        match kind.as_str() {
            "house" => Ok(Self::House),
            "farm" => Ok(Self::Farm),
            "centered_door" => Ok(Self::CenteredDoor {
                vertical_factor: get_lua(&table, "vertical_factor")?,
            }),
            "no_walkway" => Ok(Self::NoWalkway {
                door_pos: get_v2(&table, "door_pos")?,
            }),
            _ => Err(mlua::Error::external(format!(
                "Unknown building gen kind: {}",
                kind
            ))),
        }
    }
}
