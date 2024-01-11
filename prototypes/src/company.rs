use crate::{get_with_err, GoodsCompanyID, ItemID, Prototype, PrototypeBase};
use egui_inspect::{debug_inspect_impl, Inspect};
use geom::Vec2;
use mlua::{FromLua, Lua, Table, Value};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone)]
/// Power in watts (J/s)
pub struct Power(pub i64);
debug_inspect_impl!(Power);

#[derive(Debug, Error)]
pub enum PowerParseError {
    #[error("Invalid unit: {0} (accepted: W, kW, MW, GW)")]
    InvalidUnit(String),
    #[error("Invalid number")]
    InvalidNumber,
    #[error("Power is too big")]
    TooBig,
}

impl FromStr for Power {
    type Err = PowerParseError;

    /// Parse a power value from a string. The unit can be W, kW or MW.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (mut number, rest) =
            common::parse_f64(s).map_err(|_| PowerParseError::InvalidNumber)?;

        let unit = rest.trim();

        match unit {
            "W" => {}
            "kW" => number *= 1000.0,
            "MW" => number *= 1000.0 * 1000.0,
            _ => return Err(PowerParseError::InvalidUnit(unit.to_string())),
        }

        if number > i64::MAX as f64 {
            return Err(PowerParseError::TooBig);
        }

        Ok(Self(number as i64))
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (unit, div) = match self.0 {
            0..=999 => ("W", 1.0),
            1000..=999_999 => ("kW", 1000.0),
            1_000_000..=999_999_999 => ("MW", 1_000_000.0),
            _ => ("GW", 1_000_000_000.0),
        };

        write!(f, "{:.2}{}", self.0 as f64 / div, unit)
    }
}

impl<'lua> FromLua<'lua> for Power {
    fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::Nil => Ok(Self(0)),
            Value::Integer(i) => Ok(Self(i as i64)),
            Value::Number(n) => {
                if n > i64::MAX as f64 {
                    return Err(mlua::Error::external(PowerParseError::TooBig));
                }
                Ok(Self(n as i64))
            }
            Value::String(s) => {
                let s = s.to_str()?;
                Self::from_str(s).map_err(mlua::Error::external)
            }
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Power",
                    message: Some("expected nil, string or number".into()),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Inspect)]
pub struct RecipeItem {
    pub id: ItemID,
    pub amount: i32,
}

impl<'lua> FromLua<'lua> for RecipeItem {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let table: Table = FromLua::from_lua(value, lua)?;

        if let Ok(v) = table.get(1) {
            let item_id = ItemID::from_lua(v, lua)?;
            let amount = table.get(2)?;
            return Ok(Self {
                id: item_id,
                amount,
            });
        }

        let name = get_with_err::<String>(&table, "id")?;
        let item_id = ItemID::from(&name);
        let amount = get_with_err(&table, "amount")?;

        Ok(Self {
            id: item_id,
            amount,
        })
    }
}

#[derive(Debug, Clone, Inspect)]
pub struct Recipe {
    pub consumption: Vec<RecipeItem>,
    pub production: Vec<RecipeItem>,

    pub power_usage: Power,
    pub power_generation: Power,

    /// Time to execute the recipe when the facility is at full capacity, in seconds
    pub complexity: i32,

    /// Quantity to store per production in terms of quantity produced. So if it takes 1ton of flour to make
    /// 1 ton of bread. A storage multiplier of 3 means 3 tons of bread will be stored before stopping to
    /// produce it.
    pub storage_multiplier: i32,
}

impl<'lua> FromLua<'lua> for Recipe {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let table: Table = FromLua::from_lua(value, lua)?;
        Ok(Self {
            consumption: get_with_err(&table, "consumption")?,
            production: get_with_err(&table, "production")?,
            power_usage: get_with_err(&table, "power_usage")?,
            power_generation: get_with_err(&table, "power_generation")?,
            complexity: get_with_err(&table, "complexity")?,
            storage_multiplier: get_with_err(&table, "storage_multiplier")?,
        })
    }
}

#[derive(Debug)]
pub struct GoodsCompanyPrototype {
    pub base: PrototypeBase,
    pub id: GoodsCompanyID,
    pub label: String,
    pub bgen: BuildingGen,
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub n_trucks: i32,
    pub n_workers: i32,
    pub size: f32,
    pub asset_location: String,
    pub price: i64,
    pub zone: Option<ZoneDescription>,
}

impl Prototype for GoodsCompanyPrototype {
    type ID = GoodsCompanyID;
    const KIND: &'static str = "goods-company";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        Ok(Self {
            base: PrototypeBase::from_lua(table)?,
            id: GoodsCompanyID::from(&get_with_err::<String>(table, "name")?),
            label: get_with_err(table, "label")?,
            bgen: get_with_err(table, "bgen")?,
            kind: get_with_err(table, "kind")?,
            recipe: get_with_err(table, "recipe")?,
            n_trucks: table.get::<_, Option<i32>>("n_trucks")?.unwrap_or(0),
            n_workers: get_with_err(table, "n_workers")?,
            size: get_with_err(table, "size")?,
            asset_location: get_with_err(table, "asset_location")?,
            price: get_with_err(table, "price")?,
            zone: get_with_err(table, "zone").ok(),
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }
}

impl Deref for GoodsCompanyPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum CompanyKind {
    // Buyers come to get their goods
    Store,
    // Buyers get their goods delivered to them
    Factory,
    // Buyers get their goods instantly delivered, useful for things like electricity/water/..
    Network,
}

impl<'a> FromLua<'a> for CompanyKind {
    fn from_lua(value: Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
        let s: String = FromLua::from_lua(value, lua)?;
        match &*s {
            "store" => Ok(Self::Store),
            "factory" => Ok(Self::Factory),
            "network" => Ok(Self::Network),
            _ => Err(mlua::Error::external(format!(
                "Unknown company kind: {}",
                s
            ))),
        }
    }
}

debug_inspect_impl!(CompanyKind);

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

impl<'a> FromLua<'a> for BuildingGen {
    fn from_lua(value: Value<'a>, _: &'a Lua) -> mlua::Result<Self> {
        let table = match value {
            Value::String(s) => {
                let s = s.to_str()?;
                return match &*s {
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
        let kind = get_with_err::<String>(&table, "kind")?;
        match kind.as_str() {
            "house" => Ok(Self::House),
            "farm" => Ok(Self::Farm),
            "centered_door" => Ok(Self::CenteredDoor {
                vertical_factor: get_with_err(&table, "vertical_factor")?,
            }),
            "no_walkway" => Ok(Self::NoWalkway {
                door_pos: get_with_err(&table, "door_pos")?,
            }),
            _ => Err(mlua::Error::external(format!(
                "Unknown building gen kind: {}",
                kind
            ))),
        }
    }
}

#[derive(Debug)]
pub struct ZoneDescription {
    pub floor: String,
    pub filler: String,
    /// The price for each "production unit"
    pub price_per_area: i64,
    /// Whether the zone filler positions should be randomized
    pub randomize_filler: bool,
}

impl<'lua> FromLua<'lua> for ZoneDescription {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let table: Table = FromLua::from_lua(value, lua)?;
        Ok(Self {
            floor: get_with_err(&table, "floor")?,
            filler: get_with_err(&table, "filler")?,
            price_per_area: get_with_err(&table, "price_per_area").unwrap_or(100),
            randomize_filler: get_with_err(&table, "randomize_filler").unwrap_or(false),
        })
    }
}
