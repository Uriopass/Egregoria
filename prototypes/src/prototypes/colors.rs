use crate::{get_color, NoParent, Prototype, PrototypeBase};
use geom::Color;
use mlua::Table;
use std::ops::Deref;

use super::*;

/// ColorsPrototype is the prototype to hold data about colors
#[derive(Clone, Debug)]
pub struct ColorsPrototype {
    pub base: PrototypeBase,
    pub id: ColorsPrototypeID,

    pub sand_col: Color,
    pub sea_col: Color,

    pub roof_col: Color,
    pub house_col: Color,

    pub gui_success: Color,
    pub gui_danger: Color,
    pub gui_primary: Color,
    pub gui_disabled: Color,

    pub road_low_col: Color,
    pub road_mid_col: Color,
    pub road_hig_col: Color,
    pub road_line_col: Color,
    pub road_pylon_col: Color,

    pub lot_unassigned_col: Color,
    pub lot_residential_col: Color,
}

impl Prototype for ColorsPrototype {
    type Parent = NoParent;
    type ID = ColorsPrototypeID;
    const NAME: &'static str = "colors";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = PrototypeBase::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,

            sand_col: get_color(table, "sand_col")?,
            sea_col: get_color(table, "sea_col")?,

            roof_col: get_color(table, "roof_col")?,
            house_col: get_color(table, "house_col")?,

            gui_success: get_color(table, "gui_success")?,
            gui_danger: get_color(table, "gui_danger")?,
            gui_primary: get_color(table, "gui_primary")?,
            gui_disabled: get_color(table, "gui_disabled")?,

            road_low_col: get_color(table, "road_low_col")?,
            road_mid_col: get_color(table, "road_mid_col")?,
            road_hig_col: get_color(table, "road_hig_col")?,
            road_line_col: get_color(table, "road_line_col")?,
            road_pylon_col: get_color(table, "road_pylon_col")?,

            lot_unassigned_col: get_color(table, "lot_unassigned_col")?,
            lot_residential_col: get_color(table, "lot_residential_col")?,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> &Self::Parent {
        &NoParent
    }
}

impl Deref for ColorsPrototype {
    type Target = PrototypeBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
