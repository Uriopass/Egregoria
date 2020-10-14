use arc_swap::{ArcSwap, Guard};
use geom::Color;
use imgui_inspect_derive::*;
use lazy_static::*;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::sync::Arc;

#[derive(Inspect, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tree_color: Color,
    pub gui_bg_color: Color,
    pub gui_title_color: Color,
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
}

pub fn config() -> Guard<'static, Arc<Config>> {
    CONFIG.load()
}

pub fn update_config(new_config: Config) {
    save_config(&new_config);
    CONFIG.store(Arc::new(new_config));
}
