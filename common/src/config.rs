use arc_swap::{ArcSwap, Guard};
use geom::Color;
use imgui_inspect_derive::*;
use lazy_static::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Inspect, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tree_col: Color,
    pub grass_col: Color,
    pub sand_col: Color,
    pub sea_col: Color,
    pub roof_col: Color,
    pub gui_bg_col: Color,
    pub gui_title_col: Color,
    pub road_low_col: Color,
    pub road_mid_col: Color,
    pub road_hig_col: Color,
    pub road_line_col: Color,
    pub lot_col: Color,
}

fn load_config_start() -> Config {
    let c = serde_json::from_reader(BufReader::new(
        File::open("assets/config.json").expect("Could not open config file."),
    ))
    .unwrap();
    save_config(&c);
    c
}

fn save_config(config: &Config) {
    serde_json::to_writer_pretty(
        BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("assets/config.json")
                .expect("Could not open config file"),
        ),
        config,
    )
    .expect("could not serialize config");
}

lazy_static! {
    static ref CONFIG: ArcSwap<Config> = ArcSwap::from_pointee(load_config_start());
    static ref CONFIG_ID: AtomicUsize = AtomicUsize::new(0);
}

pub fn config() -> Guard<'static, Arc<Config>> {
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
