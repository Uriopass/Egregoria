#![allow(clippy::upper_case_acronyms)]

pub use config::*;
pub use history::*;
pub use time::*;
pub use z::*;

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
    ($e: expr, $($t: expr)+) => {
        match $e {
            Some(x) => x,
            None => {
                log::error!($($t),+);
                continue;
            }
        }
    };
}

pub mod config;
pub mod history;
pub mod rand;
pub mod saveload;
pub mod time;
pub mod timestep;
mod z;

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
