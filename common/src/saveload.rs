use bincode::{DefaultOptions, Options};
use imgui_inspect::imgui::__core::fmt::Display;
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

fn create_file(path: &str) -> Option<File> {
    File::create(path).map_err(|e| log::error!("{}", e)).ok()
}

fn open_file(path: &str) -> Option<File> {
    File::open(path).ok()
}

pub trait Encoder {
    const EXTENSION: &'static str;
    type Err: Display;

    fn encode(x: &impl Serialize) -> Result<Vec<u8>, Self::Err> {
        let mut cursor = std::io::Cursor::new(vec![]);
        Self::encode_writer(x, &mut cursor)?;
        Ok(cursor.into_inner())
    }

    fn decode<'a, T: Deserialize<'a>>(x: &'a [u8]) -> Result<T, Self::Err>;

    fn decode_seed<'a, S: DeserializeSeed<'a>>(
        seed: S,
        x: &'a [u8],
    ) -> Result<S::Value, Self::Err> {
        let cursor = std::io::Cursor::new(x);
        Self::decode_seed_reader(seed, cursor)
    }

    fn decode_seed_reader<'a, S: DeserializeSeed<'a>>(
        seed: S,
        read: impl Read,
    ) -> Result<S::Value, Self::Err>;

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<(), Self::Err>;
    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T, Self::Err>;

    fn filename(name: &'static str) -> String {
        format!("world/{}.{}", name, Self::EXTENSION)
    }

    fn load_reader(name: &'static str) -> Option<BufReader<File>> {
        let file = open_file(&Self::filename(name))?;
        Some(BufReader::new(file))
    }

    fn save(x: &impl Serialize, name: &'static str) -> Option<()> {
        Self::save_silent(x, name)?;
        log::info!("successfully saved {}", name);
        Some(())
    }

    fn save_silent(x: &impl Serialize, name: &'static str) -> Option<()> {
        let _ = std::fs::create_dir("world");

        let file = create_file(&Self::filename(name))?;

        let w = BufWriter::new(file);

        Self::encode_writer(x, w)
            .map_err(|e| log::error!("failed serializing: {}", e))
            .ok()?;
        Some(())
    }

    fn load<T: DeserializeOwned>(name: &'static str) -> Option<T> {
        Self::decode_reader(Self::load_reader(name)?)
            .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
            .map(|x| {
                log::info!("successfully loaded {}", name);
                x
            })
            .ok()
    }

    fn load_seed<S: DeserializeSeed<'static>>(name: &'static str, seed: S) -> Option<S::Value> {
        Self::decode_seed_reader(seed, Self::load_reader(name)?)
            .map_err(|err| log::error!("failed deserializing {}: {}", name, err))
            .map(|x| {
                log::info!("successfully loaded {}", name);
                x
            })
            .ok()
    }

    fn load_or_default<T: DeserializeOwned + Default>(name: &'static str) -> T {
        Self::load(name).unwrap_or_default()
    }
}

pub struct Binary;

impl Encoder for Binary {
    const EXTENSION: &'static str = "bc";
    type Err = bincode::Error;

    fn encode(x: &impl Serialize) -> Result<Vec<u8>, Self::Err> {
        bincode::DefaultOptions::new().serialize(x)
    }

    fn decode<'a, T: Deserialize<'a>>(x: &'a [u8]) -> Result<T, Self::Err> {
        bincode::DefaultOptions::new().deserialize(x)
    }

    fn decode_seed<'a, S: DeserializeSeed<'a>>(
        seed: S,
        x: &'a [u8],
    ) -> Result<S::Value, Self::Err> {
        seed.deserialize(&mut bincode::Deserializer::from_slice(
            &x,
            DefaultOptions::new(),
        ))
    }

    fn decode_seed_reader<'a, S: DeserializeSeed<'a>>(
        seed: S,
        read: impl Read,
    ) -> Result<<S as DeserializeSeed<'a>>::Value, Self::Err> {
        seed.deserialize(&mut bincode::Deserializer::with_reader(
            read,
            DefaultOptions::new(),
        ))
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<(), Self::Err> {
        bincode::DefaultOptions::new().serialize_into(w, x)
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T, Self::Err> {
        bincode::DefaultOptions::new().deserialize_from(r)
    }
}

pub struct JSON;

impl Encoder for JSON {
    const EXTENSION: &'static str = "json";
    type Err = serde_json::Error;

    fn decode<'a, T: Deserialize<'a>>(x: &'a [u8]) -> Result<T, Self::Err> {
        serde_json::from_slice(x)
    }

    fn decode_seed_reader<'a, S: DeserializeSeed<'a>>(
        seed: S,
        read: impl Read,
    ) -> Result<<S as DeserializeSeed<'a>>::Value, Self::Err> {
        seed.deserialize(&mut serde_json::Deserializer::from_reader(read))
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<(), Self::Err> {
        serde_json::to_writer_pretty(w, x)
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T, Self::Err> {
        serde_json::from_reader(r)
    }
}
