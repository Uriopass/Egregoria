use rustc_hash::FxHasher;
use std::any::TypeId;
use std::hash::{BuildHasher, Hash, Hasher};

pub fn hash_iter<I>(iter: I) -> u64
where
    I: IntoIterator,
    I::Item: Hash,
{
    let mut hasher = FxHasher::default();
    for item in iter {
        item.hash(&mut hasher);
    }
    hasher.finish()
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

pub type FastMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type FastSet<V> = rustc_hash::FxHashSet<V>;

pub fn fastmap_with_capacity<K, V>(cap: usize) -> FastMap<K, V> {
    FastMap::with_capacity_and_hasher(cap, Default::default())
}

pub fn fastset_with_capacity<V>(cap: usize) -> FastSet<V> {
    FastSet::with_capacity_and_hasher(cap, Default::default())
}

pub type TransparentMap<K, V> = std::collections::HashMap<K, V, TransparentHasherU64>;

pub fn transparentmap_with_capacity<K, V>(cap: usize) -> TransparentMap<K, V> {
    TransparentMap::with_capacity_and_hasher(cap, Default::default())
}

pub fn transparentmap<K, V>() -> TransparentMap<K, V> {
    TransparentMap::with_hasher(Default::default())
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
