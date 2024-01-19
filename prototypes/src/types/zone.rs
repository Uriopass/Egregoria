use crate::{get_lua, Money};
use mlua::{FromLua, Lua, Table, Value};

#[derive(Debug, Clone)]
pub struct Zone {
    pub floor: String,
    pub filler: String,
    /// The price for each "production unit"
    pub price_per_area: Money,
    /// Whether the zone filler positions should be randomized
    pub randomize_filler: bool,
}

impl<'lua> FromLua<'lua> for Zone {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let table: Table = FromLua::from_lua(value, lua)?;
        Ok(Self {
            floor: get_lua(&table, "floor")?,
            filler: get_lua(&table, "filler")?,
            price_per_area: get_lua(&table, "price_per_area").unwrap_or(Money::new_bucks(100)),
            randomize_filler: get_lua(&table, "randomize_filler").unwrap_or(false),
        })
    }
}
