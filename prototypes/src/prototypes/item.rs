use crate::prototypes::PrototypeBase;
use crate::{get_with_err, ItemID, Prototype};
use mlua::Table;
use std::ops::Deref;

/// Item is the runtime representation of an item, such as meat, wood, etc.
#[derive(Clone, Debug)]
pub struct ItemPrototype {
    pub id: ItemID,
    pub base: PrototypeBase,
    pub label: String,
    pub optout_exttrade: bool,
}

impl Deref for ItemPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Prototype for ItemPrototype {
    type ID = ItemID;
    const KIND: &'static str = "item";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: ItemID::new(&base.name),
            base,
            label: get_with_err(table, "label")?,
            optout_exttrade: get_with_err(table, "optout_exttrade").unwrap_or(false),
        })
    }

    fn id(&self) -> Self::ID {
        ItemID::from(&self.name)
    }
}
