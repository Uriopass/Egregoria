use crate::prototypes::PrototypeBase;
use crate::{get_lua, ItemID, NoParent, Prototype};
use mlua::Table;
use std::ops::Deref;

/// Item is the runtime representation of an item, such as meat, wood, etc.
#[derive(Clone, Debug)]
pub struct ItemPrototype {
    pub base: PrototypeBase,
    pub id: ItemID,
    pub optout_exttrade: bool,
}

impl Prototype for ItemPrototype {
    type Parent = NoParent;
    type ID = ItemID;
    const NAME: &'static str = "item";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
            optout_exttrade: get_lua(table, "optout_exttrade").unwrap_or(false),
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &NoParent
    }
}

impl Deref for ItemPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
