use bincode::{DefaultOptions, Options};
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn filename(name: &'static str) -> String {
    format!("world/{}.bc", name)
}

fn create_file(path: &str) -> Option<File> {
    File::create(path).map_err(|e| log::error!("{}", e)).ok()
}

fn open_file(path: &str) -> Option<File> {
    File::open(path).ok()
}

pub fn save<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    save_silent(x, name);
    log::info!("successfully saved {}", name);
    Some(())
}

pub fn save_silent<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    let _ = std::fs::create_dir("world");

    let file = create_file(&filename(name))?;

    let w = BufWriter::new(file);

    let _ = bincode::serialize_into(w, x);
    Some(())
}

pub fn load_or_default<T: DeserializeOwned + Default>(name: &'static str) -> T {
    load(name).unwrap_or_default()
}

pub fn load<T: DeserializeOwned>(name: &'static str) -> Option<T> {
    bincode::deserialize_from(load_reader(name)?)
        .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
        .map(|x| {
            log::info!("successfully loaded {}", name);
            x
        })
        .ok()
}

pub fn load_seed<S: DeserializeSeed<'static>>(name: &'static str, seed: S) -> Option<S::Value> {
    seed.deserialize(&mut bincode::Deserializer::with_reader(
        load_reader(name)?,
        DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding(),
    ))
    .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
    .map(|x| {
        log::info!("successfully loaded {}", name);
        x
    })
    .ok()
}

pub fn load_reader(name: &'static str) -> Option<BufReader<File>> {
    let file = open_file(&filename(name))?;
    Some(BufReader::new(file))
}
