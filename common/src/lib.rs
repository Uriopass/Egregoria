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
macro_rules! unwrap_orr {
    ($e: expr, $t: expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => $t,
        }
    };
}

pub mod config;
pub mod history;
pub mod rand;
pub mod saveload;
pub mod time;
mod z;

pub use config::*;
pub use history::*;
pub use time::*;
pub use z::*;

#[derive(Copy, Clone)]
pub enum AudioKind {
    Music,
    Effect,
    Ui,
}

pub type FastMap<K, V> = fnv::FnvHashMap<K, V>;
pub type FastSet<V> = fnv::FnvHashSet<V>;

pub fn fastmap_with_capacity<K, V>(cap: usize) -> FastMap<K, V> {
    FastMap::with_capacity_and_hasher(cap, fnv::FnvBuildHasher::default())
}

pub fn fastset_with_capacity<V>(cap: usize) -> FastSet<V> {
    FastSet::with_capacity_and_hasher(cap, fnv::FnvBuildHasher::default())
}
