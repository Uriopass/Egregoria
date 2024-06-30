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
