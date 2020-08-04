use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;

fn filename(name: &'static str) -> String {
    format!("world/{}.bc", name)
}

fn open_file(path: String) -> Option<File> {
    match File::create(path) {
        Ok(f) => Some(f),
        Err(e) => {
            println!("{}", e);
            None
        }
    }
}

pub fn save<T: Serialize>(x: &T, name: &'static str) -> Option<()> {
    let _ = std::fs::create_dir("world");

    let file = open_file(filename(name))?;

    let _ = bincode::serialize_into(file, x)
        .map_err(|err| println!("Error while serializing {}: {}", name, err));
    Some(())
}

pub fn load<T: DeserializeOwned>(name: &'static str) -> Option<T> {
    let file = open_file(filename(name))?;

    let des = bincode::deserialize_from(file);
    des.map_err(|err| println!("Error while deserializing {}: {}", name, err))
        .ok()
}
