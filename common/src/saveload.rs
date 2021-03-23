use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};

fn create_file(path: &str) -> Option<File> {
    File::create(path).map_err(|e| log::error!("{}", e)).ok()
}

fn open_file(path: &str) -> Option<File> {
    File::open(path).ok()
}

pub trait Encoder {
    const EXTENSION: &'static str;

    fn encode(x: &impl Serialize) -> std::io::Result<Vec<u8>>;

    fn decode<T: DeserializeOwned>(x: &[u8]) -> std::io::Result<T>;

    fn decode_seed<S: for<'a> DeserializeSeed<'a>>(
        seed: S,
        x: &[u8],
    ) -> std::io::Result<<S as DeserializeSeed<'_>>::Value>;

    fn encode_writer(x: &impl Serialize, mut w: impl Write) -> std::io::Result<()> {
        let buf = Self::encode(x)?;
        Ok(w.write_all(&*buf)?)
    }

    fn decode_reader<T: DeserializeOwned>(mut r: impl Read) -> std::io::Result<T> {
        let mut buf = vec![];
        r.read_to_end(&mut buf)?;
        Self::decode(&*buf)
    }

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

    fn load_or_default<T: DeserializeOwned + Default>(name: &'static str) -> T {
        Self::load(name).unwrap_or_default()
    }
}

pub struct Bincode;

use ::bincode::{DefaultOptions, Options};
use std::io::Result;
impl Encoder for Bincode {
    const EXTENSION: &'static str = "bc";

    fn encode(x: &impl Serialize) -> Result<Vec<u8>> {
        ::bincode::DefaultOptions::new()
            .serialize(x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> Result<T> {
        ::bincode::DefaultOptions::new()
            .deserialize(x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_seed<S: for<'a> DeserializeSeed<'a>>(
        seed: S,
        x: &[u8],
    ) -> Result<<S as DeserializeSeed<'_>>::Value> {
        seed.deserialize(&mut ::bincode::Deserializer::from_slice(
            &x,
            DefaultOptions::new(),
        ))
        .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<()> {
        ::bincode::DefaultOptions::new()
            .serialize_into(w, x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T> {
        ::bincode::DefaultOptions::new()
            .deserialize_from(r)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }
}

pub struct Cbor;

impl Encoder for Cbor {
    const EXTENSION: &'static str = "cbor";

    fn encode(x: &impl Serialize) -> std::io::Result<Vec<u8>> {
        serde_cbor::to_vec(x).map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> std::io::Result<T> {
        serde_cbor::from_slice(x).map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_seed<S: for<'a> DeserializeSeed<'a>>(
        seed: S,
        x: &[u8],
    ) -> std::io::Result<<S as DeserializeSeed<'_>>::Value> {
        seed.deserialize(&mut serde_cbor::Deserializer::from_slice(x))
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }
}

pub struct CompressedCbor;

impl Encoder for CompressedCbor {
    const EXTENSION: &'static str = "zip";

    fn encode(x: &impl Serialize) -> std::io::Result<Vec<u8>> {
        Ok(miniz_oxide::deflate::compress_to_vec(
            &*serde_cbor::to_vec(x).map_err(|x| std::io::Error::new(ErrorKind::Other, x))?,
            10,
        ))
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> std::io::Result<T> {
        let v = &miniz_oxide::inflate::decompress_to_vec(x)
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "could not decode zipped file"))?;
        serde_cbor::from_slice(v).map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_seed<S: for<'a> DeserializeSeed<'a>>(
        _: S,
        _: &[u8],
    ) -> std::io::Result<<S as DeserializeSeed<'_>>::Value> {
        unimplemented!()
    }
}

pub struct JSON;

impl Encoder for JSON {
    const EXTENSION: &'static str = "json";

    fn encode(x: &impl Serialize) -> std::io::Result<Vec<u8>> {
        serde_json::to_vec(x).map_err(Into::into)
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> std::io::Result<T> {
        serde_json::from_slice(x).map_err(Into::into)
    }

    fn decode_seed<S: for<'a> DeserializeSeed<'a>>(
        seed: S,
        x: &[u8],
    ) -> std::io::Result<<S as DeserializeSeed<'_>>::Value> {
        seed.deserialize(&mut serde_json::Deserializer::from_slice(x))
            .map_err(Into::into)
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> std::io::Result<()> {
        serde_json::to_writer_pretty(w, x).map_err(Into::into)
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> std::io::Result<T> {
        serde_json::from_reader(r).map_err(Into::into)
    }
}
