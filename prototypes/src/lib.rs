#![warn(clippy::iter_over_hash_type)]

use common::TransparentMap;
use geom::Vec2;
use mlua::{FromLua, Table};
use std::fmt::Debug;
use std::hash::Hash;

mod macros;

mod load;
mod prototypes;
mod tests;
mod types;
mod validation;

pub use load::*;
pub use prototypes::*;
pub use types::*;

/// A prototype is a collection of data that is dynamically loaded with Lua and defines a type of object
pub trait Prototype: 'static + Sized {
    /// The parent prototype of this prototype (optional). Use NoParent if there is no parent
    type Parent: ConcretePrototype;

    /// The type of the ID of the prototype
    type ID: Copy + Clone + Eq + Ord + Hash + 'static;

    /// The name of the prototype used to parse the prototype from Lua's data table
    const NAME: &'static str;

    /// Parse the prototype from a Lua table
    fn from_lua(table: &Table) -> mlua::Result<Self>;

    /// The ID of the prototype
    fn id(&self) -> Self::ID;

    /// The parent of the prototype
    fn parent(&self) -> &Self::Parent;
}

/// A concrete prototype is a prototype that has a static storage and ordering (it is not virtual)
pub trait ConcretePrototype: Prototype + Clone {
    const HAS_PARENT: bool = true;

    fn ordering(prototypes: &Prototypes) -> &[Self::ID];
    fn storage(prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self>;
    fn storage_mut(prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self>;

    /// util function to recursively insert the parents of this prototype into the prototypes lists
    fn insert_parents(&self, prototypes: &mut Prototypes) {
        let p = self.parent();
        if !<Self::Parent as ConcretePrototype>::HAS_PARENT {
            return;
        }
        Self::Parent::storage_mut(prototypes).insert(p.id(), p.clone());
        p.insert_parents(prototypes);
    }
}

/// The unique ID of a prototype
pub trait PrototypeID: Debug + Copy + Clone + Eq + Ord + Hash + 'static {
    type Prototype: Prototype<ID = Self>;
}

#[derive(Clone)]
pub struct NoParent;

impl Prototype for NoParent {
    type Parent = NoParent;
    type ID = ();
    const NAME: &'static str = "no-parent";

    fn from_lua(_table: &Table) -> mlua::Result<Self> {
        unreachable!()
    }

    fn id(&self) -> Self::ID {
        unreachable!()
    }

    fn parent(&self) -> &Self::Parent {
        self
    }
}

impl ConcretePrototype for NoParent {
    const HAS_PARENT: bool = false;

    fn ordering(_prototypes: &Prototypes) -> &[Self::ID] {
        &[]
    }

    fn storage(_prototypes: &Prototypes) -> &TransparentMap<Self::ID, Self> {
        unreachable!()
    }

    fn storage_mut(_prototypes: &mut Prototypes) -> &mut TransparentMap<Self::ID, Self> {
        unreachable!()
    }

    fn insert_parents(&self, _prototypes: &mut Prototypes) {}
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

pub fn try_prototypes() -> Option<&'static Prototypes> {
    unsafe { PROTOTYPES }
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

#[inline]
pub fn try_prototype<ID: PrototypeID>(id: ID) -> Option<&'static <ID as PrototypeID>::Prototype>
where
    ID::Prototype: ConcretePrototype,
{
    <ID as PrototypeID>::Prototype::storage(prototypes()).get(&id)
}

#[inline]
pub(crate) fn try_prototype_preload<ID: PrototypeID>(
    id: ID,
) -> Option<&'static <ID as PrototypeID>::Prototype>
where
    ID::Prototype: ConcretePrototype,
{
    <ID as PrototypeID>::Prototype::storage(try_prototypes()?).get(&id)
}

#[inline]
pub fn prototypes_iter<T: ConcretePrototype>() -> impl Iterator<Item = &'static T> {
    let p = prototypes();
    let storage = T::storage(p);
    T::ordering(p).iter().map(move |id| &storage[id])
}

#[inline]
pub fn prototypes_iter_ids<T: ConcretePrototype>() -> impl Iterator<Item = T::ID> {
    T::ordering(prototypes()).iter().copied()
}

fn get_lua<'a, T: FromLua<'a>>(t: &Table<'a>, field: &'static str) -> mlua::Result<T> {
    t.get::<_, T>(field)
        .map_err(|e| mlua::Error::external(format!("field {}: {}", field, e)))
}

fn get_lua_opt<'a, T: FromLua<'a>>(t: &Table<'a>, field: &'static str) -> mlua::Result<Option<T>> {
    t.get::<_, Option<T>>(field)
        .map_err(|e| mlua::Error::external(format!("field {}: {}", field, e)))
}

fn get_v2(t: &Table, field: &'static str) -> mlua::Result<Vec2> {
    let v = get_lua::<LuaVec2>(t, field)?;
    Ok(v.0)
}

fn get_color(t: &Table, field: &'static str) -> mlua::Result<geom::Color> {
    let v = get_lua::<LuaColor>(t, field)?;
    Ok(v.0)
}
