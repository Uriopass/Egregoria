use common::TransparentMap;
use egui_inspect::Inspect;
use mlua::{FromLua, Lua, Table};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io;
use thiserror::Error;

mod company;
mod item;
mod tests;
mod validation;

use crate::validation::ValidationError;
pub use company::*;
pub use item::*;

pub trait Prototype: 'static + Sized {
    type ID: Copy + Clone + Eq + Ord + Hash + 'static;
    const KIND: &'static str;

    fn from_lua(table: &Table) -> mlua::Result<Self>;
    fn id(&self) -> Self::ID;
}

pub trait ConcretePrototype: Prototype {
    fn storage(prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self>;
    fn storage_mut(prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self>;
}

pub trait PrototypeID: Debug + Copy + Clone + Eq + Ord + Hash + 'static {
    type Prototype: Prototype<ID = Self>;
}

#[derive(Debug, Clone, Inspect)]
pub struct PrototypeBase {
    pub name: String,
}

impl Prototype for PrototypeBase {
    type ID = ();
    const KIND: &'static str = "base";

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        Ok(Self {
            name: table.get("name")?,
        })
    }

    fn id(&self) -> Self::ID {
        ()
    }
}

static mut PROTOTYPES: Option<&'static Prototypes> = None;

#[inline]
pub fn prototypes() -> &'static Prototypes {
    #[cfg(debug_assertions)]
    {
        assert!(unsafe { PROTOTYPES.is_some() });
    }

    // Safety: Please just don't use prototypes before they were loaded... We can allow this footgun
    unsafe { PROTOTYPES.unwrap_unchecked() }
}

#[inline]
pub fn prototype<ID: PrototypeID>(id: ID) -> &'static <ID as PrototypeID>::Prototype
where
    ID::Prototype: ConcretePrototype,
{
    match <ID as PrototypeID>::Prototype::storage(prototypes()).get(&id) {
        Some(v) => v,
        None => panic!("no prototype for id {:?}", id),
    }
}

pub fn try_prototype<ID: PrototypeID>(id: ID) -> Option<&'static <ID as PrototypeID>::Prototype>
where
    ID::Prototype: ConcretePrototype,
{
    <ID as PrototypeID>::Prototype::storage(prototypes()).get(&id)
}

pub fn prototypes_iter<T: ConcretePrototype>() -> impl Iterator<Item = &'static T> {
    T::storage(prototypes()).values()
}

pub fn prototypes_iter_ids<T: ConcretePrototype>() -> impl Iterator<Item = T::ID> {
    T::storage(prototypes()).keys().copied()
}

pub fn test_prototypes(lua: &str) {
    let l = Lua::new();

    unsafe { load_prototypes_str(l, lua).unwrap() };
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

#[derive(Debug)]
pub struct MultiError<T>(Vec<T>);

impl<T: Display> Display for MultiError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

impl<T: Error> Error for MultiError<T> {}

macro_rules! prototype_id {
    ($id:ident => $proto:ty) => {
        #[derive(
            Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
        )]
        pub struct $id(pub(crate) u64);

        egui_inspect::debug_inspect_impl!($id);

        impl $id {
            #[inline]
            pub fn new(v: &str) -> $id {
                Self(common::hash_u64(v))
            }

            #[inline]
            pub fn prototype(self) -> &'static $proto {
                crate::prototype(self)
            }

            #[inline]
            pub fn hash(&self) -> u64 {
                self.0
            }
        }

        impl<'a> From<&'a str> for $id {
            fn from(v: &'a str) -> Self {
                Self(common::hash_u64(v))
            }
        }

        impl<'a> From<&'a String> for $id {
            fn from(v: &'a String) -> Self {
                Self(common::hash_u64(&*v))
            }
        }

        impl<'a> mlua::FromLua<'a> for $id {
            fn from_lua(v: mlua::Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
                match v {
                    mlua::Value::String(s) => {
                        let Ok(v) = s.to_str() else {
                            return Err(mlua::Error::FromLuaConversionError {
                                from: "string",
                                to: stringify!($id),
                                message: Some("expected utf-8 string".into()),
                            });
                        };
                        Ok(Self(common::hash_u64(v)))
                    }
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: v.type_name(),
                        to: stringify!($id),
                        message: Some("expected string".into()),
                    }),
                }
            }
        }

        impl crate::PrototypeID for $id {
            type Prototype = $proto;
        }
    };
}

macro_rules! gen_prototypes {
    ($($name:ident : $id:ident => $t:ty,)+) => {
        $(
            prototype_id!($id => $t);
        )+

        pub struct Prototypes {
            $(
                $name: TransparentMap<$id, $t>,
            )+
        }

        impl Default for Prototypes {
            fn default() -> Self {
                Self {
                    $(
                        $name: Default::default(),
                    )+
                }
            }
        }

        $(
        impl ConcretePrototype for $t {
            fn storage(prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self> {
                &prototypes.$name
            }

            fn storage_mut(prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self> {
                &mut prototypes.$name
            }
        }

        impl $t {
            pub fn iter() -> impl Iterator<Item = &'static Self> {
                crate::prototypes_iter::<Self>()
            }
            pub fn iter_ids() -> impl Iterator<Item = $id> {
                crate::prototypes_iter_ids::<Self>()
            }
        }
        )+

        fn print_prototype_stats() {
            $(
                log::info!("loaded {} {}", <$t>::storage(prototypes()).len(), <$t>::KIND);
            )+
        }

        fn parse_prototype(table: Table, proto: &mut Prototypes) -> Result<(), PrototypeLoadError> {
            let _type = table.get::<_, String>("type")?;
            let _type_str = _type.as_str();
            match _type_str {
                $(
                    <$t>::KIND => {
                        let p: $t = Prototype::from_lua(&table).map_err(|e| {
                              PrototypeLoadError::PrototypeLuaError(_type_str.to_string(), table.get::<_, String>("name").unwrap(), e)
                        })?;
                        if let Some(v) = proto.$name.insert((&p.name).into(), p) {
                            log::warn!("duplicate {} with name: {}", <$t>::KIND, v.name);
                        }
                    }
                ),+
                _ => {
                    if let Ok(s) = table.get::<_, String>("type") {
                        log::warn!("unknown prototype: {}", s)
                    }
                }
            }

            Ok(())
        }
    };
}

gen_prototypes!(companies: GoodsCompanyID => GoodsCompanyPrototype,
                items:     ItemID         => ItemPrototype,
);

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

    let mut p = Box::new(Prototypes::default());

    let mut errors = Vec::new();

    let data_table = l.globals().get::<_, Table>("data")?;

    let _ = data_table.for_each(|_: String, t: Table| {
        let r = parse_prototype(t, &mut *p);
        if let Err(e) = r {
            errors.push(e);
        }
        Ok(())
    });

    if !errors.is_empty() {
        return Err(PrototypeLoadError::MultiError(MultiError(errors)));
    }

    validation::validate(&p)?;

    unsafe {
        PROTOTYPES = Some(Box::leak(p));
    }

    print_prototype_stats();

    Ok(())
}

fn get_with_err<'a, T: FromLua<'a>>(t: &Table<'a>, field: &'static str) -> mlua::Result<T> {
    t.get::<_, T>(field)
        .map_err(|e| mlua::Error::external(format!("field {}: {}", field, e)))
}
