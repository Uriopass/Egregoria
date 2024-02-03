use prototypes::{GameTime, Tick};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type Resource = dyn Any + Send + Sync + 'static;
type ResourceSingleThread = dyn Any + 'static;

#[derive(Default)]
pub struct Resources {
    resources: common::FastMap<TypeId, RwLock<Box<Resource>>>,
}

#[derive(Default)]
pub struct ResourcesSingleThread {
    resources: common::FastMap<TypeId, RefCell<Box<ResourceSingleThread>>>,
}

fn downcast_resource<T: Any + Send + Sync>(resource: Box<Resource>) -> T {
    *resource
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("downcasting resources should always succeed"))
}

fn downcast_resource_single_thread<T: Any>(resource: Box<ResourceSingleThread>) -> T {
    *resource
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("downcasting resources should always succeed"))
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) -> Tick {
        self.read::<GameTime>().tick
    }

    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), RwLock::new(Box::new(resource)))
            .map(|resource| downcast_resource(resource.into_inner().unwrap()))
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .map(|resource| downcast_resource(resource.into_inner().unwrap()))
    }

    pub fn write_or_default<T: Any + Send + Sync + Default>(&mut self) -> RefMut<T> {
        self.write_or_insert_with(Default::default)
    }

    pub fn write_or_insert_with<T: Any + Send + Sync>(
        &mut self,
        f: impl FnOnce() -> T,
    ) -> RefMut<T> {
        unsafe {
            // Safety: we just created the rwlock with a &mut self, it cannot be poisoned yet
            RefMut::from_lock(
                self.resources
                    .entry(TypeId::of::<T>())
                    .or_insert_with(move || RwLock::new(Box::new(f()))),
            )
            .unwrap_unchecked()
        }
    }

    pub fn read<T: Any + Send + Sync>(&self) -> Ref<T> {
        Ref::from_lock(self.resources.get(&TypeId::of::<T>()).unwrap()).unwrap()
    }

    pub fn try_read<T: Any + Send + Sync>(&self) -> Result<Ref<T>, CantGetResource> {
        Ok(Ref::from_lock(
            self.resources
                .get(&TypeId::of::<T>())
                .ok_or(NoSuchResource)?,
        )?)
    }

    pub fn write<T: Any + Send + Sync>(&self) -> RefMut<T> {
        RefMut::from_lock(self.resources.get(&TypeId::of::<T>()).unwrap()).unwrap()
    }

    pub fn try_write<T: Any + Send + Sync>(&self) -> Result<RefMut<T>, CantGetResource> {
        Ok(RefMut::from_lock(
            self.resources
                .get(&TypeId::of::<T>())
                .ok_or(NoSuchResource)?,
        )?)
    }

    pub fn iter(&self) -> impl Iterator<Item = &TypeId> {
        self.resources.keys()
    }
}

pub struct Ref<'a, T: Any + Send + Sync> {
    read_guard: RwLockReadGuard<'a, Box<Resource>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any + Send + Sync> Ref<'a, T> {
    pub(crate) fn from_lock(lock: &'a RwLock<Box<Resource>>) -> Result<Self, InvalidBorrow> {
        lock.try_read()
            .map(|guard| Self {
                read_guard: guard,
                phantom: PhantomData,
            })
            .map_err(|_| InvalidBorrow::Immutable)
    }
}

impl<'a, T: Any + Send + Sync> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: we are mapped by type ids
        unsafe { self.read_guard.downcast_ref().unwrap_unchecked() }
    }
}

/// Mutable borrow of a [`Resource`] stored in a [`Resources`] container.
///
/// [`Resource`]: trait.Resource.html
/// [`Resources`]: struct.Resources.html
pub struct RefMut<'a, T: Any + Send + Sync> {
    write_guard: RwLockWriteGuard<'a, Box<Resource>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any + Send + Sync> RefMut<'a, T> {
    pub(crate) fn from_lock(lock: &'a RwLock<Box<Resource>>) -> Result<Self, InvalidBorrow> {
        lock.try_write()
            .map(|guard| Self {
                write_guard: guard,
                phantom: PhantomData,
            })
            .map_err(|_| InvalidBorrow::Mutable)
    }
}

impl<'a, T: Any + Send + Sync> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: we are mapped by type ids
        unsafe { self.write_guard.downcast_ref().unwrap_unchecked() }
    }
}

impl<'a, T: Any + Send + Sync> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: we are mapped by type ids
        unsafe { self.write_guard.downcast_mut().unwrap_unchecked() }
    }
}

impl ResourcesSingleThread {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<T: Any>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Any>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)))
            .map(|resource| downcast_resource_single_thread(resource.into_inner()))
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .map(|resource| downcast_resource_single_thread(resource.into_inner()))
    }

    pub fn write_or_default<T: Any + Default>(&mut self) -> RefMutSingle<T> {
        self.write_or_insert_with(Default::default)
    }

    pub fn write_or_insert_with<T: Any>(&mut self, f: impl FnOnce() -> T) -> RefMutSingle<T> {
        unsafe {
            // Safety: we just created the rwlock with a &mut self, it cannot be poisoned yet
            RefMutSingle::from_lock(
                self.resources
                    .entry(TypeId::of::<T>())
                    .or_insert_with(move || RefCell::new(Box::new(f()))),
            )
            .unwrap_unchecked()
        }
    }

    pub fn read<T: Any>(&self) -> RefSingle<T> {
        RefSingle::from_lock(self.resources.get(&TypeId::of::<T>()).unwrap()).unwrap()
    }

    pub fn try_read<T: Any>(&self) -> Result<RefSingle<T>, CantGetResource> {
        Ok(RefSingle::from_lock(
            self.resources
                .get(&TypeId::of::<T>())
                .ok_or(NoSuchResource)?,
        )?)
    }

    pub fn write<T: Any>(&self) -> RefMutSingle<T> {
        RefMutSingle::from_lock(self.resources.get(&TypeId::of::<T>()).unwrap()).unwrap()
    }

    pub fn try_write<T: Any>(&self) -> Result<RefMutSingle<T>, CantGetResource> {
        Ok(RefMutSingle::from_lock(
            self.resources
                .get(&TypeId::of::<T>())
                .ok_or(NoSuchResource)?,
        )?)
    }

    pub fn iter(&self) -> impl Iterator<Item = &TypeId> {
        self.resources.keys()
    }
}

pub struct RefSingle<'a, T: Any> {
    read_guard: std::cell::Ref<'a, Box<ResourceSingleThread>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> RefSingle<'a, T> {
    pub(crate) fn from_lock(
        lock: &'a RefCell<Box<ResourceSingleThread>>,
    ) -> Result<Self, InvalidBorrow> {
        lock.try_borrow()
            .map(|guard| Self {
                read_guard: guard,
                phantom: PhantomData,
            })
            .map_err(|_| InvalidBorrow::Immutable)
    }
}

impl<'a, T: Any> Deref for RefSingle<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: we are mapped by type ids
        unsafe { self.read_guard.downcast_ref().unwrap_unchecked() }
    }
}

/// Mutable borrow of a [`Resource`] stored in a [`Resources`] container.
///
/// [`Resource`]: trait.Resource.html
/// [`Resources`]: struct.Resources.html
pub struct RefMutSingle<'a, T: Any> {
    write_guard: std::cell::RefMut<'a, Box<ResourceSingleThread>>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> RefMutSingle<'a, T> {
    pub(crate) fn from_lock(
        lock: &'a RefCell<Box<ResourceSingleThread>>,
    ) -> Result<Self, InvalidBorrow> {
        lock.try_borrow_mut()
            .map(|guard| Self {
                write_guard: guard,
                phantom: PhantomData,
            })
            .map_err(|_| InvalidBorrow::Mutable)
    }
}

impl<'a, T: Any> Deref for RefMutSingle<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: we are mapped by type ids
        unsafe { self.write_guard.downcast_ref().unwrap_unchecked() }
    }
}

impl<'a, T: Any> DerefMut for RefMutSingle<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: we are mapped by type ids
        unsafe { self.write_guard.downcast_mut().unwrap_unchecked() }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchResource;

impl Display for NoSuchResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad("no such resource")
    }
}

impl Error for NoSuchResource {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InvalidBorrow {
    /// Can't access mutably because the resource is accessed either immutably or mutably elsewhere.
    Mutable,
    /// Can't access immutably because the resource is accessed mutably elsewhere.
    Immutable,
}

impl Display for InvalidBorrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            InvalidBorrow::Mutable => "cannot borrow mutably",
            InvalidBorrow::Immutable => "cannot borrow immutably",
        })
    }
}

impl Error for InvalidBorrow {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CantGetResource {
    /// Accessing the resource would violate borrow rules.
    InvalidBorrow(InvalidBorrow),
    /// No resource of this type is present in the container.
    NoSuchResource(NoSuchResource),
}

impl Display for CantGetResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use CantGetResource::*;
        match self {
            InvalidBorrow(error) => error.fmt(f),
            NoSuchResource(error) => error.fmt(f),
        }
    }
}

impl Error for CantGetResource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CantGetResource::*;
        match self {
            InvalidBorrow(error) => Some(error),
            NoSuchResource(error) => Some(error),
        }
    }
}

impl From<NoSuchResource> for CantGetResource {
    fn from(error: NoSuchResource) -> Self {
        CantGetResource::NoSuchResource(error)
    }
}

impl From<InvalidBorrow> for CantGetResource {
    fn from(error: InvalidBorrow) -> Self {
        CantGetResource::InvalidBorrow(error)
    }
}
