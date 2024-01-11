use egui_inspect::debug_inspect_impl;
use geom::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecipeDescription {
    pub consumption: Vec<(String, i32)>,
    pub production: Vec<(String, i32)>,
    pub complexity: i32,
    pub storage_multiplier: i32,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompanyKind {
    // Buyers come to get their goods
    Store,
    // Buyers get their goods delivered to them
    Factory { n_trucks: u32 },
    // Buyers get their goods instantly delivered, useful for things like electricity/water/..
    Network,
}

debug_inspect_impl!(CompanyKind);

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BuildingGen {
    House,
    Farm,
    CenteredDoor {
        vertical_factor: f32, // 1.0 means that the door is at the bottom, just on the street
    },
    NoWalkway {
        door_pos: Vec2, // door_pos is relative to the center of the building
    },
}

#[derive(Serialize, Deserialize)]
pub struct GoodsCompanyDescriptionJSON {
    pub name: String,
    pub bgen: BuildingGen,
    #[serde(flatten)]
    pub kind: CompanyKind,
    pub recipe: RecipeDescription,
    pub n_workers: i32,
    pub size: f32,
    pub asset_location: String,
    pub price: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<Box<ZoneDescription>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneDescription {
    pub floor: String,
    pub filler: String,
    /// The price for each "production unit"
    pub price_per_area: i64,
    /// Wether the zone filler positions should be randomized
    #[serde(default)]
    pub randomize_filler: bool,
}

impl Default for ZoneDescription {
    fn default() -> Self {
        Self {
            floor: "".to_string(),
            filler: "".to_string(),
            price_per_area: 100,
            randomize_filler: false,
        }
    }
}
