use core::{
    alloc::{AllocError, Allocator},
    hash::{BuildHasher, Hash},
};

/// High level hash table
pub trait HashTable<K, V, S: BuildHasher, A: Allocator + Clone>: Sized {
    fn with_capacity_and_hasher_in(
        capacity: usize,
        hash_builder: S,
        allocator: A,
    ) -> Result<Self, AllocError>;
    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, AllocError>;
    //unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Option<V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn capacity(&self) -> usize;
}

/// implementors of this trait should not track the number of elements.
pub trait HashTableImpl<K: Eq + Hash, V, S: BuildHasher, A: Allocator>: Sized {
    fn with_capacity_and_hasher_in(
        capacity: usize,
        hash_builder: S,
        allocator: A,
    ) -> Result<Self, AllocError>;
    fn grow(&mut self) -> Result<(), AllocError>;
    /// # Safety
    /// This method does not do bounds checks.
    //#[deprecated]
    unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Result<Option<V>, AllocError>;
    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, AllocError>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    //fn set_capacity(&mut self) -> usize;
}

pub trait HashTableImplIter<'a, K: Eq + Hash + 'a, V: 'a, S: BuildHasher, A: Allocator>:
    Sized + HashTableImpl<K, V, S, A>
{
    type Iter: 'a + Iterator<Item = (&'a K, &'a V)>;
    fn iter(&'a self) -> Self::Iter;
}
