use std::cmp::Ordering;

mod chunkid;
pub mod error;
mod hash;
pub mod history;
pub mod iter;
pub mod logger;
pub mod macros;
pub mod rand;
pub mod saveload;
pub mod scroll;
pub mod timestep;

pub use chunkid::*;
pub use hash::*;

pub use inline_tweak as tw;

pub fn parse_f32(x: &str) -> fast_float::Result<(f32, &str)> {
    fast_float::parse_partial::<f32, _>(x)
        .map(|(v, l)| (v, if l == x.len() { "" } else { &x[l..] }))
}

pub fn parse_f64(x: &str) -> fast_float::Result<(f64, &str)> {
    fast_float::parse_partial::<f64, _>(x)
        .map(|(v, l)| (v, if l == x.len() { "" } else { &x[l..] }))
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
