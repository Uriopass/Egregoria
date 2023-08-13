use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

/// BTreeMapScroller helps traversing a btree in multiple steps without keeping a reference to the btree, using a pivot point each time
/// The idea is for example to do 0..10 then 10..20 then 20..30 etc
/// The BTree allows to do this efficiently
/// All keys won't necessarily be traversed if the tree changes between next() calls
#[derive(Default, Serialize, Deserialize)]
pub struct BTreeMapScroller<K: Ord + Clone> {
    pub pivot: Option<K>,
}

/// BTreeSetScroller helps traversing a btree set in multiple steps without keeping a reference to the btree, using a pivot point each time
/// The idea is for example to do 0..10 then 10..20 then 20..30 etc
/// The BTree allows to do this efficiently
/// All keys won't necessarily be traversed if the tree changes between next() calls
#[derive(Default, Serialize, Deserialize)]
pub struct BTreeSetScroller<K: Ord + Clone> {
    pub pivot: Option<K>,
}

impl<K: Ord + Clone> BTreeMapScroller<K> {
    /// See [`BTreeMapScroller`](struct.BTreeMapScroller.html) for more details
    pub fn new() -> Self {
        Self { pivot: None }
    }

    pub fn reset(&mut self) {
        self.pivot = None;
    }

    /// Checks if the iterator is exhausted, meaning that the next call to iter() will return an empty iterator
    /// This is useful to know if we need to start to scroll again
    pub fn exhausted<V>(&self, btree: &BTreeMap<K, V>) -> bool {
        self.pivot.is_some()
            && btree
                .range((
                    Bound::Excluded(self.pivot.as_ref().unwrap()),
                    Bound::Unbounded,
                ))
                .next()
                .is_none()
    }

    /// See [`BTreeMapScroller`](struct.BTreeMapScroller.html) for more details
    pub fn iter<'a, V>(&'a mut self, btree: &'a BTreeMap<K, V>) -> impl Iterator<Item = (&K, &V)> {
        let left_bound;
        if let Some(x) = self.pivot.take() {
            left_bound = Bound::Excluded(x);
        } else {
            left_bound = Bound::Unbounded;
        };

        btree.range((left_bound, Bound::Unbounded)).map(|x| {
            self.pivot = Some(x.0.clone());
            x
        })
    }

    /// See [`BTreeMapScroller`](struct.BTreeMapScroller.html) for more details
    pub fn iter_mut<'a, V>(
        &'a mut self,
        btree: &'a mut BTreeMap<K, V>,
    ) -> impl Iterator<Item = (&K, &mut V)> {
        let left_bound;
        if let Some(x) = self.pivot.take() {
            left_bound = Bound::Excluded(x);
        } else {
            left_bound = Bound::Unbounded;
        };
        btree.range_mut((left_bound, Bound::Unbounded)).map(|x| {
            self.pivot = Some(x.0.clone());
            x
        })
    }
}

impl<K: Ord + Clone> BTreeSetScroller<K> {
    /// See [`BTreeSetScroller`](struct.BTreeSetScroller.html) for more details
    pub fn new() -> Self {
        Self { pivot: None }
    }

    /// Checks if the iterator is exhausted, meaning that the next call to iter() will return an empty iterator
    /// This is useful to know if we need to start to scroll again
    pub fn exhausted(&self, btree: &BTreeSet<K>) -> bool {
        self.pivot.is_some()
            && btree
                .range((
                    Bound::Excluded(self.pivot.as_ref().unwrap()),
                    Bound::Unbounded,
                ))
                .next()
                .is_none()
    }

    pub fn reset(&mut self) {
        self.pivot = None;
    }

    /// See [`BTreeSetScroller`](struct.BTreeSetScroller.html) for more details
    /// Returns an iterator over the keys of the btree, but the iterator can stop without a ref to the btree and can start again
    pub fn iter<'a>(&'a mut self, btree: &'a BTreeSet<K>) -> impl Iterator<Item = &K> {
        let left_bound;
        if let Some(x) = self.pivot.take() {
            left_bound = Bound::Excluded(x);
        } else {
            left_bound = Bound::Unbounded;
        };

        btree.range((left_bound, Bound::Unbounded)).map(|x| {
            self.pivot = Some(x.clone());
            x
        })
    }

    /// See [`BTreeSetScroller`](struct.BTreeSetScroller.html) for more details
    /// Returns an iterator over the keys of the btree, but the iterator can stop without a ref to the btree and can start again
    /// The iterator will start again from the beginning on the next iter_looped call when it reaches the end
    /// This is not an infinite iterator, any key may not be processed twice in a single iter_looped call
    pub fn iter_looped<'a>(&'a mut self, btree: &'a BTreeSet<K>) -> impl Iterator<Item = &K> {
        if self.exhausted(btree) {
            self.reset();
        }
        let left_bound;
        if let Some(x) = self.pivot.take() {
            left_bound = Bound::Excluded(x);
        } else {
            left_bound = Bound::Unbounded;
        };

        btree.range((left_bound, Bound::Unbounded)).map(|x| {
            self.pivot = Some(x.clone());
            x
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_btree_map_iter() {
        let mut scroller = BTreeMapScroller::new();

        let mut btree = BTreeMap::new();
        btree.insert(1, 1);
        btree.insert(2, 2);
        btree.insert(3, 3);

        let mut iter = scroller.iter(&btree);
        assert_eq!(iter.next(), Some((&1, &1)));
        assert_eq!(iter.next(), Some((&2, &2)));
        drop(iter);
        let mut iter = scroller.iter(&btree);
        assert_eq!(iter.next(), Some((&3, &3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_btree_map_iter_mut() {
        let mut scroller = BTreeMapScroller::new();

        let mut btree = BTreeMap::new();
        btree.insert(1, 1);
        btree.insert(2, 2);
        btree.insert(3, 3);

        let mut iter = scroller.iter_mut(&mut btree);
        assert_eq!(iter.next(), Some((&1, &mut 1)));
        assert_eq!(iter.next(), Some((&2, &mut 2)));
        drop(iter);
        let mut iter = scroller.iter_mut(&mut btree);
        assert_eq!(iter.next(), Some((&3, &mut 3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_btree_set_iter() {
        let mut scroller = BTreeSetScroller::new();

        let mut btree = BTreeSet::new();
        btree.insert(1);
        btree.insert(2);
        btree.insert(3);

        let mut iter = scroller.iter(&btree);
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        drop(iter);
        let mut iter = scroller.iter(&btree);
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }
}
