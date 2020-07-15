use geom::polygon::Polygon;
use lazy_static::*;
use mlua::{FromLuaMulti, Lua};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, RwLock};
use std::time::SystemTime;

mod stdlib;
use stdlib::*;

trait ResultExt<T> {
    fn ok_print(self) -> Option<T>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn ok_print(self) -> Option<T> {
        self.map_err(|err| println!("{}", err)).ok()
    }
}

struct LuaFile {
    source: String,
    time: SystemTime,
    lua: Lua,
}

unsafe impl Send for LuaFile {}

pub struct Mods {
    files: RwLock<HashMap<&'static str, Mutex<LuaFile>>>, // Mutex is very important to guarentee sync
}

impl Mods {
    fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }
}

lazy_static! {
    static ref MODS: Mods = Mods::new();
}

pub fn eval_script<T: for<'a> FromLuaMulti<'a>>(name: &'static str) -> Option<T> {
    let mut data_file = File::open(name)
        .map_err(|err| format!("Could not open `{}`, {}", name, err))
        .unwrap();

    let time = data_file.metadata().ok_print()?.modified().ok_print()?;

    let mut mkfile = || {
        let mut data = String::new();
        data_file.read_to_string(&mut data).unwrap();
        let lua = Lua::new();
        add_std(&lua);
        Mutex::new(LuaFile {
            source: data,
            time,
            lua,
        })
    };

    {
        let guard = MODS.files.read().unwrap();
        let cur = guard.get(name);

        match cur {
            Some(sf) => {
                let luaf = sf.lock().unwrap();
                if luaf.time == time {
                    return luaf.lua.load(&luaf.source).eval().ok_print();
                }
                println!("Re-loading {}", name);
            }
            None => {
                println!("Loading {}", name);
            }
        }
    }

    MODS.files.write().unwrap().insert(name, mkfile());

    let guard = MODS.files.read().unwrap();
    let luaf = guard.get(name).unwrap().lock().unwrap();
    luaf.lua.load(&luaf.source).eval().ok_print()
}

pub fn gen_house() -> Option<Polygon> {
    eval_script::<LuaPolygon>("scripts/test.lua").map(|x| x.0)
}
