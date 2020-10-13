use std::collections::HashMap;

pub mod config;
pub mod inspect;
pub mod rand;
pub mod saveload;
pub mod time;

pub use config::*;
pub use time::*;

pub fn get_mut_pair<'a, K, V>(
    conns: &'a mut HashMap<K, V>,
    a: &K,
    b: &K,
) -> Option<(&'a mut V, &'a mut V)>
where
    K: std::fmt::Debug + Eq + std::hash::Hash,
{
    unsafe {
        assert_ne!(a, b, "`a` ({:?}) must not equal `b` ({:?})", a, b);
        let a = conns.get_mut(a)? as *mut _;
        let b = conns.get_mut(b)? as *mut _;
        Some((&mut *a, &mut *b))
    }
}
