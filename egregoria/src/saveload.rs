use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn filename(name: &'static str) -> String {
    format!("world/{}.bc", name)
}

fn create_file(path: &str) -> Option<File> {
    File::create(path).map_err(|e| error!("{}", e)).ok()
}

fn open_file(path: &str) -> Option<File> {
    File::open(path).ok()
}

pub fn save<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    let _ = std::fs::create_dir("world");

    let file = create_file(&filename(name))?;

    let _ = bincode::serialize_into(BufWriter::new(file), x)
        .map_err(|err| error!("failed serializing {}: {}", name, err));
    info!("successfully saved {}", name);
    Some(())
}

pub fn load_or_default<T: DeserializeOwned + Default>(name: &'static str) -> T {
    load(name).unwrap_or_default()
}

pub fn load<T: DeserializeOwned>(name: &'static str) -> Option<T> {
    let file = open_file(&filename(name))?;

    let des = bincode::deserialize_from(BufReader::new(file));
    des.map_err(|err| error!("failed deserializing {}: {}", name, err))
        .map(|x| {
            info!("successfully loaded {}", name);
            x
        })
        .ok()
}
