// Prototype template. remplace $proto with the root name e.g Item

use crate::{NoParent, Prototype, PrototypeBase, get_lua};
use mlua::Table;
use std::ops::Deref;

use super::*;

/// $protoPrototype is
#[derive(Clone, Debug)]
pub struct $protoPrototype {
    pub base: $parent,
    pub id: $protoPrototypeID,
}

impl Prototype for $protoPrototype {
    type Parent = $parent;
    type ID = $protoPrototypeID;
    const NAME: &'static str = ;

    fn from_lua(table: &Table) -> mlua::Result<Self> {
        let base = $parent::from_lua(table)?;
        Ok(Self {
            id: Self::ID::new(&base.name),
            base,
        })
    }

    fn id(&self) -> Self::ID {
        self.id
    }

    fn parent(&self) -> Option<&Self::Parent> {
        Some(&self.base)
    }
}

impl Deref for $protoPrototype {
    type Target = $parent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}