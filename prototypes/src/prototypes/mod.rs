pub mod company;
pub mod item;

pub use company::*;
pub use item::*;

#[derive(Debug, Clone, egui_inspect::Inspect)]
pub struct PrototypeBase {
    pub name: String,
}

impl crate::Prototype for PrototypeBase {
    type ID = ();
    const KIND: &'static str = "base";

    fn from_lua(table: &mlua::Table) -> mlua::Result<Self> {
        Ok(Self {
            name: table.get("name")?,
        })
    }

    fn id(&self) -> Self::ID {
        ()
    }
}
