use geom::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RecipeDescription {
    pub consumption: Vec<(String, i32)>,
    pub production: Vec<(String, i32)>,
    pub complexity: i32,
    pub storage_multiplier: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BuildingGenDescription {
    pub kind: String,
    pub vertical_factor: Option<f32>,
    pub door_pos: Option<Vec2>,
}

#[derive(Serialize, Deserialize)]
pub struct GoodsCompanyDescriptionJSON {
    pub name: String,
    pub bgen: BuildingGenDescription,
    pub kind: String,
    pub recipe: RecipeDescription,
    pub n_workers: i32,
    pub n_trucks: Option<u32>,
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
