use bincode::DefaultOptions;
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter};

static USE_JSON: bool = true;

fn filename(name: &'static str) -> String {
    if USE_JSON {
        format!("world/{}.json", name)
    } else {
        format!("world/{}.bc", name)
    }
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

    let w = BufWriter::new(file);

    if USE_JSON {
        let _ = serde_json::to_writer(w, x)
            .map_err(|err| error!("failed serializing {}: {}", name, err));
    } else {
        let _ = bincode::serialize_into(w, x)
            .map_err(|err| error!("failed serializing {}: {}", name, err));
    }
    info!("successfully saved {}", name);
    Some(())
}

pub fn load_or_default<T: DeserializeOwned + Default>(name: &'static str) -> T {
    load(name).unwrap_or_default()
}

pub fn load<T: DeserializeOwned>(name: &'static str) -> Option<T> {
    let des = if USE_JSON {
        serde_json::from_reader(load_reader(name)?)
            .map_err(|err| error!("failed deserializing {}: {}", name, err))
    } else {
        bincode::deserialize_from(load_reader(name)?)
            .map_err(|err| error!("failed deserializing {}: {}", name, err))
    };

    des.map(|x| {
        info!("successfully loaded {}", name);
        x
    })
    .ok()
}

pub fn load_seed<S: DeserializeSeed<'static>>(name: &'static str, seed: S) -> Option<S::Value> {
    let r = load_reader(name)?;

    let des = if USE_JSON {
        seed.deserialize(&mut serde_json::Deserializer::from_reader(r))
            .map_err(|err| error!("failed deserializing {}: {}", name, err))
            .ok()
    } else {
        seed.deserialize(&mut bincode::Deserializer::with_reader(
            r,
            DefaultOptions::new(),
        ))
        .map_err(|err| error!("failed deserializing {}: {}", name, err))
        .ok()
    };

    des.map(|x| {
        info!("successfully loaded {}", name);
        x
    })
}

pub fn load_reader(name: &'static str) -> Option<BufReader<File>> {
    let file = open_file(&filename(name))?;
    Some(BufReader::new(file))
}
