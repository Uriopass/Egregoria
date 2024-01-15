mod company;
mod item;
mod solar;

use crate::NoParent;
pub use company::*;
pub use item::*;
pub use solar::*;

#[derive(Debug, Clone, egui_inspect::Inspect)]
pub struct PrototypeBase {
    pub name: String,
    pub order: String,
    pub label: String,
}

impl crate::Prototype for PrototypeBase {
    type Parent = NoParent;
    type ID = ();
    const NAME: &'static str = "base";

    fn from_lua(table: &mlua::Table) -> mlua::Result<Self> {
        use crate::get_with_err;
        Ok(Self {
            name: get_with_err(table, "name")?,
            order: get_with_err(table, "order").unwrap_or(String::new()),
            label: get_with_err(table, "label")?,
        })
    }

    fn id(&self) -> Self::ID {
        ()
    }
}
