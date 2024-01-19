use crate::{get_lua, GameDuration, ItemID};
use egui_inspect::Inspect;
use mlua::{FromLua, Lua, Table, Value};

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

        let name = get_lua::<String>(&table, "id")?;
        let item_id = ItemID::from(&name);
        let amount = get_lua(&table, "amount")?;

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

    /// Time to execute the recipe when the facility is at full capacity
    pub duration: GameDuration,

    /// Quantity to store per production in terms of quantity produced. So if it takes 1ton of flour to make
    /// 1 ton of bread. A storage multiplier of 3 means 3 tons of bread will be stored before stopping to
    /// produce it.
    pub storage_multiplier: i32,
}

impl<'lua> FromLua<'lua> for Recipe {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let table: Table = FromLua::from_lua(value, lua)?;
        Ok(Self {
            consumption: get_lua(&table, "consumption")?,
            production: get_lua(&table, "production")?,
            duration: get_lua(&table, "duration")?,
            storage_multiplier: get_lua(&table, "storage_multiplier")?,
        })
    }
}
