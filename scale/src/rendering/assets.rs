use crate::gui::InspectDragf;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Copy, Serialize, Deserialize, Inspect)]
pub struct AssetID {
    pub id: u16,
}

impl AssetID {
    pub const CAR: AssetID = AssetID { id: 0 };
    pub const PEDESTRIAN: AssetID = AssetID { id: 1 };
}

#[derive(Clone, Copy, Component, Inspect)]
pub struct AssetRender {
    pub id: AssetID,
    pub hide: bool,
    #[inspect(proxy_type = "InspectDragf")]
    pub scale: f32,
}
