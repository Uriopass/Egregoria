use crate::saveload::Encoder;
use arc_swap::{ArcSwap, Guard};
use egui_inspect::Inspect;
use geom::Color;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Default, Clone, Serialize, Deserialize, Inspect)]
pub struct Config {
    pub tree_col: Color,

    pub grass_col: Color,
    pub sand_col: Color,
    pub sea_col: Color,
    pub border_col: Color,

    pub roof_col: Color,
    pub house_col: Color,

    pub gui_bg_col: Color,
    pub gui_title_col: Color,
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
    pub lot_commercial_col: Color,

    pub special_building_col: Color,
    pub special_building_invalid_col: Color,

    #[inspect(step = 0.01)]
    pub ssao_strength: f32,
    #[inspect(step = 0.001)]
    pub ssao_radius: f32,
    #[inspect(step = 0.00001)]
    pub ssao_falloff: f32,
    #[inspect(step = 0.01)]
    pub ssao_base: f32,
    pub ssao_samples: i32,
}

fn load_config_start() -> Config {
    let c = crate::saveload::load_raw("assets/config.json")
        .and_then(|x| serde_json::from_slice(&x).map_err(Into::into))
        .map_err(|x| {
            log::error!("couldn't read config: {}", x);
        })
        .unwrap_or_default();
    save_config(&c);
    c
}

fn save_config(config: &Config) {
    let Ok(x) = crate::saveload::JSONPretty::encode(config) else { return; };
    let _ = std::fs::write("assets/config.json", x);
}

lazy_static! {
    static ref CONFIG: ArcSwap<Config> = ArcSwap::from_pointee(load_config_start());
    static ref CONFIG_ID: AtomicUsize = AtomicUsize::new(0);
}

pub fn config() -> Guard<Arc<Config>> {
    CONFIG.load()
}

pub fn config_id() -> usize {
    CONFIG_ID.load(Ordering::Relaxed)
}

pub fn update_config(new_config: Config) {
    CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    save_config(&new_config);
    CONFIG.store(Arc::new(new_config));
}
