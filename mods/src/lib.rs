use mlua::{FromLuaMulti, Lua, TableExt};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;

pub use mlua;

mod stdlib;
use std::path::Path;
pub use stdlib::*;

trait ResultExt<T> {
    fn ok_print(self) -> Option<T>;
}
impl<T, E: Display> ResultExt<T> for Result<T, E> {
    fn ok_print(self) -> Option<T> {
        self.map_err(|err| log::error!("{}", err)).ok()
    }
}

pub fn call_f<'a, R: FromLuaMulti<'a>>(l: &'a Lua, f: &str) -> Option<R> {
    l.globals().call_function(f, ()).ok_print()
}

pub fn eval_f(l: &Lua, f: &str) -> Option<()> {
    call_f(l, f)
}

pub fn load<P: AsRef<Path>>(name: P) -> Option<Lua> {
    let name = name.as_ref();
    let mut data_file = File::open(name)
        .map_err(|err| log::error!("Could not open `{:?}`, {}", name, err))
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
