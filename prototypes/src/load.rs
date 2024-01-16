use crate::validation::ValidationError;
use crate::{validation, Prototypes, PROTOTYPES};
use common::error::MultiError;
use mlua::{Lua, Table};
use std::io;
use thiserror::Error;

pub fn test_prototypes(lua: &str) {
    let l = Lua::new();

    unsafe { load_prototypes_str(l, lua).unwrap() };
}

/// Loads the prototypes from the data.lua file
/// # Safety
/// This function is not thread safe, and should only be called once at the start of the program.
pub unsafe fn load_prototypes(base: &str) -> Result<(), PrototypeLoadError> {
    log::info!("loading prototypes from {}", base);
    let l = Lua::new();

    let base = base.to_string();

    l.globals()
        .get::<_, Table>("package")?
        .set("path", base.clone() + "base_mod/?.lua")?;

    load_prototypes_str(
        l,
        &common::saveload::load_string(base + "base_mod/data.lua")?,
    )
}

unsafe fn load_prototypes_str(l: Lua, main: &str) -> Result<(), PrototypeLoadError> {
    l.load(include_str!("prototype_init.lua")).exec()?;

    l.load(main).exec()?;

    let mut p = Box::<Prototypes>::default();

    let mut errors = Vec::new();

    let data_table = l.globals().get::<_, Table>("data")?;

    let _ = data_table.for_each(|_: String, t: Table| {
        let r = p.parse_prototype(t);
        if let Err(e) = r {
            errors.push(e);
        }
        Ok(())
    });

    if !errors.is_empty() {
        return Err(PrototypeLoadError::MultiError(MultiError(errors)));
    }

    validation::validate(&p)?;

    p.compute_orderings();
    p.print_stats();

    unsafe {
        PROTOTYPES = Some(Box::leak(p));
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum PrototypeLoadError {
    #[error("loading data.lua: {0}")]
    LoadingDataLua(#[from] io::Error),
    #[error("lua error: {0}")]
    LuaError(#[from] mlua::Error),
    #[error("lua error for {0} {1}: {2}")]
    PrototypeLuaError(String, String, mlua::Error),
    #[error("multiple errors: {0}")]
    MultiError(MultiError<PrototypeLoadError>),
    #[error("validation errors: {0}")]
    ValidationErrors(#[from] MultiError<ValidationError>),
}
