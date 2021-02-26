use std::collections::HashMap;

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

pub use config::*;
pub use history::*;
pub use time::*;

#[derive(Copy, Clone)]
pub enum AudioKind {
    Music,
    Effect,
    Ui,
}

pub fn get_mut_pair<'a, K, V>(
    conns: &'a mut HashMap<K, V>,
    a: &K,
    b: &K,
) -> Option<(&'a mut V, &'a mut V)>
where
    K: std::fmt::Debug + Eq + std::hash::Hash,
{
    unsafe {
        let a = conns.get_mut(a)? as *mut _;
        let b = conns.get_mut(b)? as *mut _;
        assert_ne!(a, b, "The two keys must not resolve to the same value");
        Some((&mut *a, &mut *b))
    }
}
