use common::TransparentMap;
use mlua::{FromLua, Table};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

mod load;
mod prototypes;
mod tests;
mod types;
mod validation;

pub use load::*;
pub use prototypes::*;
pub use types::*;

crate::gen_prototypes!(
    companies: GoodsCompanyID => GoodsCompanyPrototype,
    items:     ItemID         => ItemPrototype,
);

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

#[macro_export]
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

#[macro_export]
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

fn get_with_err<'a, T: FromLua<'a>>(t: &Table<'a>, field: &'static str) -> mlua::Result<T> {
    t.get::<_, T>(field)
        .map_err(|e| mlua::Error::external(format!("field {}: {}", field, e)))
}
