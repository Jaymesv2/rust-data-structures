#![feature(test)]

// TODO: optimize the insert and grow functions

mod tester;

extern crate test;

mod seperate_chaining;

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
};

use seperate_chaining::{HashTableInner, SinglyLinkedList, HashTableInnerIter};


trait HashTableImpl: Default {
    type Key: Hash + Eq;
    type Value;
    type HashBuilder: BuildHasher;

    fn with_hasher(hash_builder: Self::HashBuilder) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }
    fn with_capacity_and_hasher(capacity: usize, hash_builder: Self::HashBuilder) -> Self;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, key: &Self::Key) -> Option<&Self::Value>;
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
}

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
}

trait HashTableImplDefaultHasher: HashTableImpl<HashBuilder = RandomState> {
    fn new() -> Self {
        Self::with_capacity(0)
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<T: HashTableImpl<HashBuilder = RandomState>> HashTableImplDefaultHasher for T {}

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

    pub fn iter<'a>(&'a self) -> HashTableIter<'a, K,V,S> {
        HashTableIter {
            inner: self.inner.iter()
        }
    }
}

use std::fmt::Debug;

pub struct HashTableIter<'a, K,V,S> {
    inner: HashTableInnerIter<'a, K,V,S> 
}

impl<'a, K,V,S> Iterator for HashTableIter<'a, K,V,S> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (&item.key, &item.val))
        /*if let Some(item) = self.inner.next() {
            Some((&item.key, &item.val))
        } else {
            None
        }*/
    }
}


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

    #[test]
    fn insert_and_remove() {
        use rand::*;
        const CAPACITY: usize = 1000;

        let mut v: Vec<u64> = vec![0;CAPACITY];
        thread_rng().try_fill(&mut v[0..CAPACITY]).unwrap();
        v.sort();

        let mut a = HashTable::with_capacity(CAPACITY);
        for i in v.iter() {
            a.insert(*i,*i);
        }

        let mut r: Vec<u64> = Vec::with_capacity(CAPACITY);
        for i in v.iter() {
            r.push(*a.get(i).unwrap());
        }

        assert_eq!(r,v);
    }

    #[test]
    fn iter_test() {
        const CAPACITY: usize = 1000;

        let mut h = HashTable::with_capacity(CAPACITY);
        let v = (0..CAPACITY).collect::<Vec<_>>();
        for i in v.iter().copied() {
            h.insert(i,i);
        }

        let mut u = h.iter().map(|c| c.1).copied().collect::<Vec<_>>();
        u.sort();

        assert_eq!(u,v)
    }

     #[test]
    fn run() {
        let mut h: HashTable<i32, i32> = HashTable::new();
        
        assert_eq!(*h.get(&79).unwrap_or(&-1), -1);
h.insert(72, 7);
h.insert(77, 1);
h.insert(10, 21);
let _ = h.remove(&26);
h.insert(94, 5);
h.insert(53, 35);
h.insert(34, 9);
assert_eq!(*h.get(&94).unwrap_or(&-1), 5);
h.insert(96, 8);
h.insert(73, 79);
h.insert(7, 60);
h.insert(84, 79);
assert_eq!(*h.get(&94).unwrap_or(&-1), 5);
h.insert(18, 13);
assert_eq!(*h.get(&18).unwrap_or(&-1), 13);
h.insert(69, 34);
h.insert(21, 82);
h.insert(57, 64);
h.insert(23, 60);
let _ = h.remove(&0);
h.insert(12, 97);
h.insert(56, 90);
h.insert(44, 57);
h.insert(30, 12);
h.insert(17, 10);
h.insert(42, 13);
h.insert(62, 6);
assert_eq!(*h.get(&34).unwrap_or(&-1), 9);
h.insert(70, 16);
h.insert(51, 39);
h.insert(22, 98);
h.insert(82, 42);
h.insert(84, 7);
h.insert(29, 32);
h.insert(96, 54);
h.insert(57, 36);
h.insert(85, 82);
h.insert(49, 33);
h.insert(22, 14);
h.insert(63, 8);
h.insert(56, 8);
let _ = h.remove(&94);
h.insert(78, 77);
let _ = h.remove(&51);
h.insert(20, 89);
let _ = h.remove(&51);
h.insert(9, 38);
let _ = h.remove(&20);
h.insert(29, 64);
h.insert(92, 69);
h.insert(72, 25);
let _ = h.remove(&73);
h.insert(6, 90);
h.insert(1, 67);
h.insert(70, 83);
h.insert(58, 49);
assert_eq!(*h.get(&79).unwrap_or(&-1), -1);
h.insert(73, 2);
h.insert(56, 16);
h.insert(58, 26);
assert_eq!(*h.get(&53).unwrap_or(&-1), 35);
let _ = h.remove(&7);
h.insert(27, 17);
h.insert(55, 40);
h.insert(55, 13);
h.insert(89, 32);
let _ = h.remove(&49);
h.insert(75, 75);
h.insert(64, 52);
h.insert(94, 74);
assert_eq!(*h.get(&81).unwrap_or(&-1), -1);
h.insert(39, 82);
h.insert(47, 36);
assert_eq!(*h.get(&57).unwrap_or(&-1), 36);
assert_eq!(*h.get(&66).unwrap_or(&-1), -1);
h.insert(3, 7);
h.insert(54, 34);
h.insert(56, 46);
h.insert(58, 64);
h.insert(22, 81);
h.insert(3, 1);
h.insert(21, 96);
h.insert(6, 19);
assert_eq!(*h.get(&77).unwrap_or(&-1), 1);
h.insert(60, 66);
h.insert(48, 85);
h.insert(77, 16);
assert_eq!(*h.get(&78).unwrap_or(&-1), 77);
assert_eq!(*h.get(&23).unwrap_or(&-1), 60);
let _ = h.remove(&72);
let _ = h.remove(&27);
h.insert(20, 80);
assert_eq!(*h.get(&30).unwrap_or(&-1), 12);
assert_eq!(*h.get(&94).unwrap_or(&-1), 74);
h.insert(74, 85);
assert_eq!(*h.get(&49).unwrap_or(&-1), -1);
h.insert(79, 59);
h.insert(15, 15);
assert_eq!(*h.get(&26).unwrap_or(&-1), -1);
    }

}
