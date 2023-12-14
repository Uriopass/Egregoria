use std::any::TypeId;
use std::cmp::Ordering;
use std::hash::{BuildHasher, Hash, Hasher};

pub use history::*;

#[macro_export]
macro_rules! unwrap_or {
    ($e: expr, $t: expr) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

#[macro_export]
macro_rules! assert_ret {
    ($e: expr) => {
        if !$e {
            return false;
        }
    };
}

#[macro_export]
macro_rules! unwrap_ret {
    ($e: expr) => {
        unwrap_ret!($e, ())
    };
    ($e: expr, $ret: expr) => {
        match $e {
            Some(x) => x,
            None => return $ret,
        }
    };
}

#[macro_export]
macro_rules! unwrap_cont {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => continue,
        }
    };
}

#[macro_export]
macro_rules! unwrap_orr {
    ($e: expr, $t: expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => $t,
        }
    };
}

#[macro_export]
macro_rules! unwrap_retlog {
    ($e: expr, $($t: expr),+) => {
        match $e {
            Some(x) => x,
            None => {
                log::error!($($t),+);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_contlog {
    ($e: expr, $($t: expr),+) => {
        match $e {
            Some(x) => x,
            None => {
                log::error!($($t),+);
                continue;
            }
        }
    };
}
pub struct DeferFn<V, T: FnOnce() -> V>(pub Option<T>);

impl<V, T: FnOnce() -> V> Drop for DeferFn<V, T> {
    fn drop(&mut self) {
        (self.0.take().unwrap())();
    }
}

#[macro_export]
macro_rules! defer {
    ($e: expr) => {
        let _guard = $crate::DeferFn(Some(move || $e));
    };
}

#[macro_export]
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        assert!(
            $x - $y < $d || $y - $x < $d,
            "assert_delta failed: |{} - {}| < {}",
            $x,
            $y,
            $d
        );
    };
}

#[macro_export]
macro_rules! defer_serialize {
    ($me:ty, $defered:ty) => {
        impl serde::Serialize for $me {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
            where
                S: serde::Serializer,
            {
                let v: $defered = self.into();
                serde::Serialize::serialize(&v, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $me {
            fn deserialize<D>(
                deserializer: D,
            ) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let v: $defered = serde::Deserialize::deserialize(deserializer)?;
                Ok(v.into())
            }
        }
    };
}

#[inline]
pub fn hash_u64<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = FxHasher::default();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[inline]
/// Hashes the object's type plus content to make sure that the hash is unique even across zero sized types
pub fn hash_type_u64<T>(obj: &T) -> u64
where
    T: Hash + 'static,
{
    let mut hasher = FxHasher::default();
    TypeId::of::<T>().hash(&mut hasher);
    obj.hash(&mut hasher);
    hasher.finish()
}

pub struct AccessCmp<'a, T, F>(pub &'a T, pub F);

impl<'a, T, F, U> PartialOrd<Self> for AccessCmp<'a, T, F>
where
    F: Fn(&'a T) -> U,
    U: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1(self.0).partial_cmp(&other.1(other.0))
    }
}

impl<'a, T, F, U> Ord for AccessCmp<'a, T, F>
where
    F: Fn(&'a T) -> U,
    U: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.1(self.0).cmp(&other.1(other.0))
    }
}

impl<'a, T, F, U> PartialEq for AccessCmp<'a, T, F>
where
    F: Fn(&'a T) -> U,
    U: PartialEq<U>,
{
    fn eq(&self, other: &Self) -> bool {
        self.1(self.0).eq(&other.1(other.0))
    }
}

impl<'a, T, F, U> Eq for AccessCmp<'a, T, F>
where
    F: Fn(&'a T) -> U,
    U: Eq,
{
}

mod chunkid;
pub mod descriptions;
pub mod history;
pub mod logger;
pub mod rand;
pub mod saveload;
pub mod scroll;
pub mod timestep;

pub use chunkid::*;

pub use inline_tweak as tw;
use rustc_hash::FxHasher;

#[derive(Copy, Clone)]
pub enum AudioKind {
    Music,
    Effect,
    Ui,
}

pub type FastMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastSet<V> = rustc_hash::FxHashSet<V>;

pub fn fastmap_with_capacity<K, V>(cap: usize) -> FastMap<K, V> {
    FastMap::with_capacity_and_hasher(cap, Default::default())
}

pub fn fastset_with_capacity<V>(cap: usize) -> FastSet<V> {
    FastSet::with_capacity_and_hasher(cap, Default::default())
}

#[derive(Default)]
pub struct TransparentHasherU64(u64);

impl Hasher for TransparentHasherU64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("can only use u64 for transparenthasher")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

impl BuildHasher for TransparentHasherU64 {
    type Hasher = TransparentHasherU64;

    fn build_hasher(&self) -> Self::Hasher {
        TransparentHasherU64(self.0)
    }
}
