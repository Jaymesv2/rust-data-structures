#![feature(test, variant_count)]

// TODO: optimize the insert and grow functions
#[cfg(test)]
mod tester;

extern crate test;

mod seperate_chaining;

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
};

use seperate_chaining::{HashTableInner, HashTableInnerIter, SinglyLinkedList};

trait HashTableImpl<K: Hash + Eq + Debug, V: Debug, S: BuildHasher + Default>: Default {
    fn new() -> Self {
        Self::with_capacity(0)
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, S::default())
    }
    fn with_hasher(hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }
    fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
}

impl<K, V, S> HashTableImpl<K, V, S> for std::collections::HashMap<K, V, S>
where
    K: Hash + Eq + Debug,
    S: BuildHasher + Default,
    V: Debug,
{
    fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(capacity, hash_builder)
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
}

impl<K, V, S> HashTableImpl<K, V, S> for HashTable<K, V, S>
where
    K: Hash + Eq + Debug,
    S: BuildHasher + Default,
    V: Debug,
{
    fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        HashTable::with_capacity_and_hasher(capacity, hash_builder)
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashTable::insert(self, key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        HashTable::remove(self, key)
    }
    fn get(&self, key: &K) -> Option<&V> {
        HashTable::get(self, key)
    }
}

/*
trait HashTableImplIters<'a>: HashTableImpl
where
    Self: 'a
{
    type Iter: Iterator<Item = (&'a Self::Key, &'a Self::Value)>;
    type IterMut: Iterator<Item = (&'a mut Self::Key, &'a mut Self::Value)>;
    type DrainIter: Iterator<Item = (Self::Key, Self::Value)>;
    type KeysIter: Iterator<Item = &'a Self::Key>;
    type ValuesIter: Iterator<Item = &'a Self::Value>;
    fn iter(&self) -> Self::Iter;
    fn iter_mut(&mut self) -> Self::IterMut;
    fn drain(&mut self) -> Self::DrainIter;
    fn keys(&self) -> Self::Key;
    fn values(&self) -> Self::Value;
}*/

pub struct HashTable<K, V, S = RandomState> {
    inner: HashTableInner<K, V, S>,
}

impl<K, V> HashTable<K, V, RandomState> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S: Default + BuildHasher> Default for HashTable<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> HashTable<K, V, S>
where
    S: BuildHasher,
{
    pub fn with_hasher(hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            inner: HashTableInner::with_capacity_and_hasher(capacity, hash_builder),
        }
    }
}

impl<K, V, S> HashTable<K, V, S>
where
    S: BuildHasher,
    K: Hash + Eq + Debug,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        unsafe {
            self.inner
                .insert(SinglyLinkedList::ptr_to_new(k, v))
                .map(|v| SinglyLinkedList::extract_val_from_sll(v))
        }
    }

    pub fn get<'a>(&'a self, k: &K) -> Option<&'a V> {
        self.inner.get(k).map(|s| &unsafe { s.as_ref() }.val)
    }

    pub fn get_mut<'a>(&'a self, k: &K) -> Option<&'a mut V> {
        self.inner
            .get(k)
            .map(|mut s| &mut unsafe { s.as_mut() }.val)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.inner
            .remove(k)
            .map(|v| unsafe { SinglyLinkedList::extract_val_from_sll(v) })
    }

    pub fn iter<'a>(&'a self) -> HashTableIter<'a, K, V, S> {
        HashTableIter {
            inner: self.inner.iter(),
        }
    }
}

pub struct HashTableIter<'a, K, V, S> {
    inner: HashTableInnerIter<'a, K, V, S>,
}

impl<'a, K, V, S> Iterator for HashTableIter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (&item.key, &item.val))
    }
}

/*
use std::iter::{
    self, DoubleEndedIterator, ExactSizeIterator, Extend, FromIterator, IntoIterator, Iterator,
};*/

use std::fmt::{self, Debug, Display, Formatter};

impl<K: Debug, V: Debug, S> Debug for HashTable<K, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(miri))]
    use test::Bencher;
    #[bench]
    #[cfg(not(miri))]
    fn std_insert_bench(b: &mut Bencher) {
        use std::collections::HashMap;
        let mut h = HashMap::new();
        let mut i: i64 = 0;
        b.iter(|| {
            h.insert(i, i);
            i += 1;
        })
    }

    #[bench]
    #[cfg(not(miri))]
    fn local_insert_bench(b: &mut Bencher) {
        let mut h = HashTable::new();
        let mut i: i64 = 0;
        b.iter(|| {
            h.insert(i, i);
            i += 1;
        })
    }

    #[bench]
    #[cfg(not(miri))]
    fn std_get_bench(b: &mut Bencher) {
        use std::collections::HashMap;
        let mut h = HashMap::new();
        for i in 0..10000 {
            h.insert(i, i);
        }
        let mut i: i64 = 0;
        b.iter(|| {
            h.get(&i);
            i += 1;
        })
    }

    #[bench]
    #[cfg(not(miri))]
    fn local_get_bench(b: &mut Bencher) {
        let mut h = HashTable::new();
        for i in 0..10000 {
            h.insert(i, i);
        }
        let mut i: i64 = 0;
        b.iter(|| {
            h.get(&i);
            i += 1;
        })
    }

    #[test]
    #[cfg(miri)]
    fn create_and_drop_inner() {
        let _a = HashTable::<i32, i32>::new();
    }

    #[test]
    #[cfg(miri)]
    fn create_with_capacity_inner() {
        let _a = HashTable::<i32, i32>::with_capacity(100);
    }

    #[test]
    #[cfg(miri)]
    fn insert_growth_drop_inner() {
        let mut a = HashTable::with_capacity(5);
        for i in 0..5 {
            a.insert(1, 1);
        }
    }

    #[test]
    fn insert_drop_inner() {
        let mut a = HashTable::new();
        a.insert(1, 1);
    }

    #[test]
    fn remove_nothing_inner() {
        let mut a = HashTable::<_, i32>::new();
        assert!(a.remove(&1).is_none());
    }

    #[test]
    fn insert_remove_inner() {
        let mut a = HashTable::with_capacity(0);
        a.insert(1, 1);
        let i = a.remove(&1).unwrap();
        assert_eq!(i, 1);
    }

    #[test]
    fn insert_and_remove() {
        use rand::*;
        const CAPACITY: usize = 1000;

        let mut v: Vec<u64> = vec![0; CAPACITY];
        thread_rng().try_fill(&mut v[0..CAPACITY]).unwrap();
        v.sort();

        let mut a = HashTable::with_capacity(CAPACITY);
        for i in v.iter() {
            a.insert(*i, *i);
        }

        let mut r: Vec<u64> = Vec::with_capacity(CAPACITY);
        for i in v.iter() {
            r.push(*a.get(i).unwrap());
        }

        assert_eq!(r, v);
    }

    #[test]
    fn iter_test() {
        const CAPACITY: usize = 1000;

        let mut h = HashTable::with_capacity(CAPACITY);
        let v = (0..CAPACITY).collect::<Vec<_>>();
        for i in v.iter().copied() {
            h.insert(i, i);
        }

        let mut u = h.iter().map(|c| c.1).copied().collect::<Vec<_>>();
        u.sort();

        assert_eq!(u, v)
    }
}
