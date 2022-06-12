#![feature(test)]

// TODO: optimize the insert and grow functions



extern crate test;

mod inner;

//use std::alloc::{alloc, dealloc, Layout};
//use std::ptr::{self};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};


use inner::{HashTableInner, SinglyLinkedList};

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

impl<K,V> Default for HashTable<K,V, RandomState> {
    fn default() -> Self {
        Self::with_hasher(RandomState::new())
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
            self.inner.insert( SinglyLinkedList::ptr_to_new(k, v) ).map(|v| SinglyLinkedList::extract_val_from_sll(v))
        }
    }

    pub fn get<'a>(&'a self, k: &K) -> Option<&'a V> {
        self.inner.get(k).map(|s| &unsafe{s.as_ref()}.val)
    }

    pub fn get_mut<'a>(&'a self, k: &K) -> Option<&'a mut V> {
        self.inner.get(k).map(|mut s| &mut unsafe{s.as_mut()}.val)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.inner.remove(k).map(|v| unsafe {SinglyLinkedList::extract_val_from_sll(v)})
    }
}
use std::fmt::Debug;
/* 
use std::fmt::{self, Debug, Display, Formatter};

use std::iter::{
    self, DoubleEndedIterator, ExactSizeIterator, Extend, FromIterator, IntoIterator, Iterator,
};

impl<K, V, S> Debug for HashTable<K, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}*/

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
        a.insert(1,1);
        let i = a.remove(&1).unwrap();
        assert_eq!(i, 1);
    }
}
