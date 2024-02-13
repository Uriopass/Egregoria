use crate::{GoodsCompanyPrototype, Prototype, SolarPanelID};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct SolarPanelPrototype {
    pub base: GoodsCompanyPrototype,
    pub id: SolarPanelID,
}

impl Prototype for SolarPanelPrototype {
    type Parent = GoodsCompanyPrototype;
    type ID = SolarPanelID;
    const NAME: &'static str = "solar-panel";

    fn from_lua(table: &mlua::Table) -> mlua::Result<Self> {
        let base = GoodsCompanyPrototype::from_lua(table)?;
        Ok(Self {
            id: SolarPanelID::new(&base.name),
            base,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &self.base
    }
}

impl Deref for SolarPanelPrototype {
    type Target = GoodsCompanyPrototype;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
