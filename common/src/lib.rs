#![allow(clippy::upper_case_acronyms)]

use std::hash::Hash;

pub use config::*;
pub use history::*;
use imgui_inspect::imgui::__core::hash::Hasher;
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

pub struct PtrCmp<'a, T>(pub &'a T);

impl<'a, T> Hash for PtrCmp<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0 as *const T as usize)
    }
}

impl<'a, T> PartialEq for PtrCmp<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a, T> Eq for PtrCmp<'a, T> {}

pub mod config;
pub mod history;
pub mod logger;
pub mod rand;
pub mod saveload;
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
