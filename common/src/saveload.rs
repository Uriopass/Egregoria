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

    fn encode(x: &impl Serialize) -> Result<Vec<u8>>;

    fn decode<T: DeserializeOwned>(x: &[u8]) -> Result<T>;

    fn decode_seed<V, S: for<'a> DeserializeSeed<'a, Value = V>>(seed: S, x: &[u8]) -> Result<V>;

    fn encode_writer(x: &impl Serialize, mut w: impl Write) -> Result<()> {
        let buf = Self::encode(x)?;
        w.write_all(&*buf)
    }

    fn decode_reader<T: DeserializeOwned>(mut r: impl Read) -> Result<T> {
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
use std::time::Instant;

impl Encoder for Bincode {
    const EXTENSION: &'static str = "bc";

    fn encode(x: &impl Serialize) -> Result<Vec<u8>> {
        let mut v = Vec::with_capacity(4096);
        DefaultOptions::new()
            .serialize_into(std::io::Cursor::new(&mut v), x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))?;
        Ok(v)
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> Result<T> {
        DefaultOptions::new()
            .deserialize(x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_seed<V, S: for<'a> DeserializeSeed<'a, Value = V>>(seed: S, x: &[u8]) -> Result<V> {
        seed.deserialize(&mut ::bincode::Deserializer::from_slice(
            x,
            DefaultOptions::new(),
        ))
        .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<()> {
        DefaultOptions::new()
            .serialize_into(w, x)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T> {
        DefaultOptions::new()
            .deserialize_from(r)
            .map_err(|x| std::io::Error::new(ErrorKind::Other, x))
    }
}

pub struct CompressedBincode;

impl Encoder for CompressedBincode {
    const EXTENSION: &'static str = "zip";

    fn encode(x: &impl Serialize) -> Result<Vec<u8>> {
        let encoded = &*Bincode::encode(x)?;
        let t = Instant::now();
        let compressed = miniz_oxide::deflate::compress_to_vec(encoded, 1); // bigger level values take far too long and only compress a bit better (about 5%)
        log::info!("took {}s to compress", t.elapsed().as_secs_f32());
        Ok(compressed)
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> Result<T> {
        let v = &miniz_oxide::inflate::decompress_to_vec(x)
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "could not decode zipped file"))?;
        Bincode::decode(v)
    }

    fn decode_seed<V, S: for<'a> DeserializeSeed<'a, Value = V>>(
        seed: S,
        data: &[u8],
    ) -> Result<V> {
        let v = &miniz_oxide::inflate::decompress_to_vec(data)
            .map_err(|_| std::io::Error::new(ErrorKind::Other, "could not decode zipped file"))?;
        Bincode::decode_seed(seed, v)
    }
}

pub struct JSON;

impl Encoder for JSON {
    const EXTENSION: &'static str = "json";

    fn encode(x: &impl Serialize) -> Result<Vec<u8>> {
        serde_json::to_vec(x).map_err(Into::into)
    }

    fn decode<T: DeserializeOwned>(x: &[u8]) -> Result<T> {
        serde_json::from_slice(x).map_err(Into::into)
    }

    fn decode_seed<V, S: for<'a> DeserializeSeed<'a, Value = V>>(seed: S, x: &[u8]) -> Result<V> {
        seed.deserialize(&mut serde_json::Deserializer::from_slice(x))
            .map_err(Into::into)
    }

    fn encode_writer(x: &impl Serialize, w: impl Write) -> Result<()> {
        serde_json::to_writer_pretty(w, x).map_err(Into::into)
    }

    fn decode_reader<T: DeserializeOwned>(r: impl Read) -> Result<T> {
        serde_json::from_reader(r).map_err(Into::into)
    }
}
