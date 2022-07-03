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
}
