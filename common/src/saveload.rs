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

pub fn encode<T: Serialize>(x: &T) -> Option<Vec<u8>> {
    bincode::serialize(x)
        .map_err(|e| log::error!("failed serializing: {}", e))
        .ok()
}

pub fn decode<T: DeserializeOwned>(x: &[u8]) -> Option<T> {
    bincode::deserialize(x)
        .map_err(|e| log::error!("failed deserializing: {}", e))
        .ok()
}

pub fn decode_seed<'a, S: DeserializeSeed<'a>>(seed: S, x: &'a [u8]) -> Option<S::Value> {
    seed.deserialize(&mut bincode::Deserializer::from_slice(
        &x,
        DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding(),
    ))
    .map_err(|err| log::error!("failed deserializing: {}", err))
    .ok()
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

    bincode::serialize_into(w, x)
        .map_err(|e| log::error!("failed serializing: {}", e))
        .ok()?;
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

fn filename_json(name: &'static str) -> String {
    format!("world/{}.json", name)
}

pub fn load_reader_json(name: &'static str) -> Option<BufReader<File>> {
    let file = open_file(&filename_json(name))?;
    Some(BufReader::new(file))
}

pub fn load_seed_json<S: DeserializeSeed<'static>>(
    name: &'static str,
    seed: S,
) -> Option<S::Value> {
    seed.deserialize(&mut serde_json::Deserializer::from_reader(
        load_reader_json(name)?,
    ))
    .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
    .map(|x| {
        log::info!("successfully loaded {}", name);
        x
    })
    .ok()
}

pub fn save_json<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    save_silent_json(x, name);
    log::info!("successfully saved {}", name);
    Some(())
}

pub fn save_silent_json<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    let _ = std::fs::create_dir("world");

    let file = create_file(&filename_json(name))?;

    let w = BufWriter::new(file);

    serde_json::to_writer_pretty(w, x)
        .map_err(|e| log::error!("failed serializing: {}", e))
        .ok()?;
    Some(())
}

pub fn load_json<T: DeserializeOwned>(name: &'static str) -> Option<T> {
    serde_json::from_reader(load_reader_json(name)?)
        .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
        .map(|x| {
            log::info!("successfully loaded {}", name);
            x
        })
        .ok()
}
