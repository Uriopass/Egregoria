use geom::polygon::Polygon;
use lazy_static::*;
use mlua::{FromLuaMulti, Lua, TableExt};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::sync::{Mutex, RwLock};
use std::time::SystemTime;

pub use mlua;

mod stdlib;
pub use stdlib::*;

trait ResultExt<T> {
    fn ok_print(self) -> Option<T>;
}

impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn ok_print(self) -> Option<T> {
        self.map_err(|err| log::error!("{}", err)).ok()
    }
}

struct LuaFile {
    source: String,
    time: SystemTime,
    lua: Lua,
}

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

pub fn call_f<'a, R: FromLuaMulti<'a>>(l: &'a Lua, f: &str) -> Option<R> {
    l.globals().call_function(f, ()).ok_print()
}

pub fn eval_f(l: &Lua, f: &str) -> Option<()> {
    call_f(l, f)
}

pub fn load(name: &str) -> Option<Lua> {
    let mut data_file = File::open(name)
        .map_err(|err| log::error!("Could not open `{}`, {}", name, err))
        .ok()?;

    let mut data = String::new();
    data_file.read_to_string(&mut data).ok()?;
    let lua = unsafe { Lua::unsafe_new() };
    lua.load(r#"package.path = "lua/?.lua;?.lua""#)
        .eval()
        .ok_print()?;
    add_std(&lua);
    lua.load(&data).eval().ok_print()?;
    Some(lua)
}

pub fn with_init<F: FnOnce(&Lua), T: for<'a> FromLuaMulti<'a>>(
    f: F,
    name: &'static str,
) -> Option<T> {
    let mut data_file = File::open(name)
        .map_err(|err| log::error!("Could not open `{}`, {}", name, err))
        .ok()?;

    let time = data_file.metadata().ok_print()?.modified().ok_print()?;

    {
        let guard = MODS.files.read().unwrap();
        let cur = guard.get(name);

        match cur {
            Some(sf) => {
                let luaf = sf.lock().unwrap();
                if luaf.time == time {
                    return luaf.lua.load(&luaf.source).eval().ok_print();
                }
                log::info!("re-loading {}", name);
            }
            None => {
                log::info!("loading {}", name);
            }
        }
    }

    let mut data = String::new();
    data_file.read_to_string(&mut data).ok()?;
    let lua = unsafe { Lua::unsafe_new() };
    add_std(&lua);
    f(&lua);
    let f = Mutex::new(LuaFile {
        source: data,
        time,
        lua,
    });

    MODS.files.write().unwrap().insert(name, f);

    let guard = MODS.files.read().unwrap();
    let luaf = guard.get(name).unwrap().lock().unwrap();
    luaf.lua.load(&luaf.source).eval().ok_print()
}

pub fn eval_script<T: for<'a> FromLuaMulti<'a>>(name: &'static str) -> Option<T> {
    with_init(|_| {}, name)
}

pub fn gen_house() -> Option<Polygon> {
    eval_script::<LuaPolygon>("lua/test.lua").map(|x| x.0)
}
