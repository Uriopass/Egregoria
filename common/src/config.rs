use arc_swap::{ArcSwap, Guard};
use geom::Color;
use imgui_inspect_derive::*;
use lazy_static::*;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CONFIG_AT_COMPILE: &[u8] = include_bytes!("../../assets/config.json");

#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct Config {
    pub tree_col: Color,

    pub grass_col: Color,
    pub sand_col: Color,
    pub sea_col: Color,

    pub roof_col: Color,

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

    pub lot_unassigned_col: Color,
    pub lot_residential_col: Color,
    pub lot_commercial_col: Color,

    pub special_building_col: Color,
    pub special_building_invalid_col: Color,
}

fn load_config_start() -> Config {
    let c = serde_json::from_slice(
        std::fs::read("assets/config.json")
            .as_deref()
            .unwrap_or(CONFIG_AT_COMPILE),
    )
    .unwrap();
    save_config(&c);
    c
}

fn save_config(config: &Config) {
    if let Err(e) = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("assets/config.json")
        .and_then(|x| serde_json::to_writer_pretty(BufWriter::new(x), config).map_err(Into::into))
    {
        log::error!("could not save config: {}", e)
    }
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
