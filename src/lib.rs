#![feature(
    test,
    variant_count,
    iter_intersperse,
    generic_associated_types,
    generators,
    allocator_api
)]

// TODO: optimize the insert and grow functions
#[cfg(test)]
mod tester;

extern crate test;

use std::{
    alloc::{Allocator, Global},
    collections::hash_map::{RandomState},
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use impls::traits::*;

pub type SCHashTable<K, V, S=RandomState, A=Global> =
    HashTable<K, V, S, A, impls::seperate_chaining::SLLHashTableImpl<K, V, S, A>>;

pub struct HashTable<K, V, S, A, T>
where
    K: Eq + Hash,
    S: BuildHasher,
    A: Allocator + Clone,
    T: HashTableImpl<K, V, S, A>,
{
    inner: T,
    marker: PhantomData<(K, V, S, A)>,
}

impl<K, V, S, A, T> Debug for HashTable<K, V, S, A, T>
where
    K: Eq + Hash,
    S: BuildHasher,
    A: Allocator + Clone,
    T: HashTableImpl<K, V, S, A>,
{
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        //write!(f, "hashtable :(")
    }
}

impl<K, V, T> Default for HashTable<K, V, RandomState, Global, T>
where
    K: Eq + Hash,
    T: HashTableImpl<K, V, RandomState, Global> + Default,
{
    fn default() -> Self {
        Self {
            inner: T::with_capacity_and_hasher_in(0, RandomState::new(), Global)
                .expect("failed to allocate"),
            marker: PhantomData,
        }
    }
}

impl<K, V, S, A, T> impls::traits::HashTable<K, V, S, A> for HashTable<K, V, S, A, T>
where
    K: Eq + Hash,
    S: BuildHasher,
    A: Allocator + Clone,
    T: HashTableImpl<K, V, S, A>,
{
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, core::alloc::AllocError> {
        self.inner.insert(key, value)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let res = self.inner.remove(key)?;
        Some(res)
    }

    fn with_capacity_and_hasher_in(
        capacity: usize,
        hash_builder: S,
        allocator: A,
    ) -> Result<Self, core::alloc::AllocError> {
        let inner = T::with_capacity_and_hasher_in(capacity, hash_builder, allocator)?;
        Ok(Self {
            inner,
            marker: PhantomData,
        })
    }
}

/*
trait HashTable<K: Hash + Eq + Debug, V: Debug, S: BuildHasher + Default>: Default {
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
    fn len(&self) -> usize;
}*/
/*
impl<K, V, S> HashTable<K, V, S> for std::collections::HashMap<K, V, S>
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
    fn len(&self) -> usize {
        self.len()
    }
}*/

/*
trait SCHashTableImplIters<'a>: SCHashTableImpl
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
/*
impl<K, V, S> SCHashTableImpl<K, V, S> for SCHashTable<K, V, S>
where
    K: Hash + Eq + Debug,
    S: BuildHasher + Default,
    V: Debug,
{
    fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        SCHashTable::with_capacity_and_hasher(capacity, hash_builder)
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        SCHashTable::insert(self, key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        SCHashTable::remove(self, key)
    }
    fn get(&self, key: &K) -> Option<&V> {
        SCHashTable::get(self, key)
    }
    fn len(&self) -> usize {
        SCHashTable::len(&self)
    }
}


pub struct SCHashTable<K, V, S = RandomState> {
    inner: SCHashTableInner<K, V, S>,
}

impl<K, V> SCHashTable<K, V, RandomState> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S: Default + BuildHasher> Default for SCHashTable<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

impl<K, V, S> SCHashTable<K, V, S>
where
    S: BuildHasher,
{
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn with_hasher(hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(0, hash_builder)
    }

    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            inner: SCHashTableInner::with_capacity_and_hasher(capacity, hash_builder),
        }
    }
}

impl<K, V, S> SCHashTable<K, V, S>
where
    S: BuildHasher,
    K: Hash + Eq + Debug,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        unsafe {
            self.inner
                .insert_node(SinglyLinkedList::ptr_to_new(k, v))
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

    pub fn iter<'a>(&'a self) -> SCHashTableIter<'a, K, V> {
        SCHashTableIter {
            inner: self.inner.iter(),
        }
    }
}

pub struct SCHashTableIter<'a, K, V> {
    inner: SCHashTableInnerIter<'a, K, V>,
}

impl<'a, K, V> Iterator for SCHashTableIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| unsafe{(&item.as_ref().key, &item.as_ref().val)})
    }
}


use std::fmt::{self, Debug, Display, Formatter};

impl<K: Debug, V: Debug, S> Debug for SCHashTable<K, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.inner)
    }
}
*/
