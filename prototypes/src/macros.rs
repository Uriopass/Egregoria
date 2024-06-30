#[macro_export]
macro_rules! gen_prototypes {
    ($(mod $name:ident : $id:ident = $t:ty $(=> $parent_id:ident)?,)+) => {
        $(
            mod $name; pub use $name::*;
        )+

        $(
            $crate::prototype_id!($id => $t);
        )+

        $(
            $(
            impl From<$parent_id> for $id {
                fn from(v: $parent_id) -> Self {
                    Self(v.0)
                }
            }

            impl From<$id> for $parent_id {
                fn from(v: $id) -> Self {
                    Self(v.0)
                }
            }
            )?
        )+

        #[derive(Default)]
        pub(crate) struct Orderings {
            $(
                pub(crate) $name: Vec<$id>,
            )+
        }

        #[derive(Default)]
        pub struct Prototypes {
            $(
                pub(crate) $name: common::TransparentMap<$id, $t>,
            )+
            pub(crate) orderings: Orderings,
        }

        $(
        impl $crate::ConcretePrototype for $t {
            fn ordering(prototypes: &Prototypes) -> &[Self::ID] {
                &prototypes.orderings.$name
            }

            #[inline]
            fn storage(prototypes: &Prototypes) -> &common::TransparentMap<Self::ID, Self> {
                &prototypes.$name
            }

            fn storage_mut(prototypes: &mut Prototypes) -> &mut common::TransparentMap<Self::ID, Self> {
                &mut prototypes.$name
            }
        }

        impl $t {
            #[inline]
            pub fn iter() -> impl Iterator<Item = &'static Self> {
                $crate::prototypes_iter::<Self>()
            }
            #[inline]
            pub fn iter_ids() -> impl Iterator<Item = $id> {
                $crate::prototypes_iter_ids::<Self>()
            }
        }
        )+

        impl Prototypes {
            pub(crate) fn print_stats(&self) {
                $(
                    if <$t as $crate::ConcretePrototype>::storage(self).is_empty() {
                        log::warn!("no {} loaded", <$t as $crate::Prototype>::NAME);
                    } else {
                        log::info!("loaded {} {}", <$t as $crate::ConcretePrototype>::storage(self).len(), <$t as $crate::Prototype>::NAME);
                    }
                )+
            }

            pub(crate) fn compute_orderings(&mut self) {
                self.orderings = Orderings {
                    $(
                        $name: {
                            let mut v = <$t as $crate::ConcretePrototype>::storage(self).keys().copied().collect::<Vec<_>>();
                            v.sort_by_key(|id| {
                                let proto = &self.$name[id];
                                (&proto.order, proto.id)
                            });
                            v
                        },
                    )+
                }
            }

            pub(crate) fn parse_prototype(&mut self, table: mlua::Table) -> Result<(), $crate::PrototypeLoadError> {
                let _type = table.get::<_, String>("type")?;
                let _type_str = _type.as_str();
                match _type_str {
                    $(
                        <$t as $crate::Prototype>::NAME => {
                            let proto: $t = $crate::Prototype::from_lua(&table).map_err(|e| {
                                  $crate::PrototypeLoadError::PrototypeLuaError(_type_str.to_string(), table.get::<_, String>("name").unwrap(), e)
                            })?;

                            <$t as $crate::ConcretePrototype>::insert_parents(&proto, self);

                            if let Some(v) = self.$name.insert((&proto.name).into(), proto) {
                                log::warn!("duplicate {} with name: {}", <$t as $crate::Prototype>::NAME, v.name);
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
        }


    };
}

#[macro_export]
macro_rules! prototype_id {
    ($id:ident => $proto:ty) => {
        #[derive(
            Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
        )]
        pub struct $id(pub(crate) u64);

        egui_inspect::debug_inspect_impl!($id);

        impl core::fmt::Debug for $id {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(v) = $crate::try_prototype_preload(*self) {
                    return write!(f, "{}({:?})", stringify!($id), v.name);
                }
                write!(f, "{}({})", stringify!($id), self.0)
            }
        }

        impl $id {
            #[inline]
            pub fn new(v: &str) -> $id {
                Self(common::hash_u64(v))
            }

            #[inline]
            pub fn prototype(self) -> &'static $proto {
                $crate::prototype(self)
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

        impl $crate::PrototypeID for $id {
            type Prototype = $proto;
        }
    };
}
