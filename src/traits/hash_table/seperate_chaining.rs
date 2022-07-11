use crate::prelude::*;

use core::{
    alloc::{AllocError, Allocator},
    hash::Hash,
};

pub trait Bucket<K, V, A>
where
    K: Eq,
    A: Allocator + Clone,
{
    fn new_in(alloc: A) -> Self;
    fn insert(&mut self, key: K, value: V) -> Result<Option<(K, V)>, AllocError>;
    unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Result<(), AllocError>;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<(K, V)>;
}

pub trait BucketIters<'a, K, V, A>:
    Bucket<K, V, A> + Iterable<Item = (K, V)> + IterableMut<Item = (K, V)> + Drainable<Item = (K, V)>
where
    K: Eq + Hash,
    V: 'a,
    A: Allocator + Clone,
{
}

pub trait BucketIter<'a, K, V, A>: Bucket<K, V, A> + Iterable<Item = (K, V)>
where
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone,
{
}
// todo: iter mut and drain should be combined
pub trait BucketIterMut<'a, K, V, A>:
    Bucket<K, V, A> + Iterable<Item = (K, V)> + IterableMut
where
    V: 'a,
    K: Eq + Hash + 'a,
    A: Allocator + Clone + 'a,
{
}

pub trait BucketDrain<'a, K, V, A>: Bucket<K, V, A> + Drainable<Item = (K, V)>
where
    Self: 'a,
    K: Eq,
    A: Allocator + Clone,
{
}

impl<'a, B, K, V, A> BucketDrain<'a, K, V, A> for B
where
    Self: 'a,
    B: Drainable<Item = (K, V)> + Bucket<K, V, A>,
    K: Eq,
    A: Allocator + Clone,
{
}
impl<'a, B, K, V, A> BucketIter<'a, K, V, A> for B
where
    Self: 'a,
    B: Iterable<Item = (K, V)> + Bucket<K, V, A>,
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone,
{
}
impl<'a, B, K, V, A> BucketIterMut<'a, K, V, A> for B
where
    Self: 'a,
    B: Iterable<Item = (K, V)> + IterableMut + Bucket<K, V, A>,
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone + 'a,
{
}

/*
impl<'a, K, V, A> BucketIter<'a, K, V, A> for SinglyLinkedList<(K, V), A>
where
    Self: 'a,
    K: Eq + Hash,
    A: Allocator + Clone,{}

impl<'a, K, V, A> BucketIterMut<'a, K, V, A> for SinglyLinkedList<(K, V), A>
where
    Self: 'a,
    K: Eq + Hash,
    A: Allocator + Clone,{}
*/
/*
pub trait BucketIter<'a, K, V, A>: Bucket<K, V, A>
where
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone,
{
    type Iter: 'a + Iterator<Item = &'a (K, V)>;
    fn iter(&'a self) -> Self::Iter;
}

// todo: iter mut and drain should be combined
pub trait BucketIterMut<'a, K, V, A>: Bucket<K, V, A>
where
    V: 'a,
    K: Eq + Hash + 'a,
    A: Allocator + Clone + 'a,
{
    type IterMut: Iterator<Item = &'a mut (K, V)>;
    fn iter_mut(&mut self) -> Self::IterMut;
}

pub trait BucketDrain<'a, K, V, A>: Bucket<K, V, A>
where
    Self: 'a,
    K: Eq,
    A: Allocator + Clone,
{
    type DrainIter: 'a + Iterator<Item = (K, V)>;
    fn drain(&'a mut self) -> Self::DrainIter;
}*/
