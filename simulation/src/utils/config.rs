use arc_swap::{ArcSwap, Guard};
use common::saveload::Encoder;
use egui_inspect::Inspect;
use geom::Color;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Default, Clone, Serialize, Deserialize, Inspect)]
#[serde(default)]
pub struct Config {
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

fn load_config_start() -> Config {
    let c = common::saveload::load_raw("assets/config.json")
        .and_then(|x| common::saveload::JSON::decode(&x).map_err(Into::into))
        .map_err(|x| {
            log::error!("couldn't read config: {}", x);
        })
        .unwrap_or_default();
    save_config(&c);
    c
}

fn save_config(config: &Config) {
    let Ok(x) = common::saveload::JSONPretty::encode(config) else {
        return;
    };
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
