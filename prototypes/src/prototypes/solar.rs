use crate::{get_with_err, GoodsCompanyPrototype, Power, Prototype, SolarPanelID};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct SolarPanelPrototype {
    pub base: GoodsCompanyPrototype,
    pub id: SolarPanelID,
    /// The maximum power output when the sun is at its peak
    pub max_power: Power,
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
            max_power: get_with_err(table, "max_power")?,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> Option<&Self::Parent> {
        Some(&self.base)
    }
}

impl Deref for SolarPanelPrototype {
    type Target = GoodsCompanyPrototype;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
