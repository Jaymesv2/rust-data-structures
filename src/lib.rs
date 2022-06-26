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
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use traits::hash_table::*;

pub type SCHashTable<K, V, S = RandomState, A = Global> =
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

impl<K, V, S, A, T> traits::hash_table::HashTable<K, V, S, A> for HashTable<K, V, S, A, T>
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
    
    fn clear(&mut self) {
        self.inner.clear()
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
