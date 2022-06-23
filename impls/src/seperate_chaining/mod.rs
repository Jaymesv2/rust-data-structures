use core::{
    alloc::{AllocError, Allocator, Layout},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, *},
    slice,
};
type ElementsPtr<K, V, A> = NonNull<SLLBucket<K, V, A>>;

mod buckets;
use buckets::*;

use crate::traits::*;

pub type SLLHashTableImpl<K, V, S, A> = SCHashTableImpl<K, V, S, SLLBucket<K, V, A>, A>;

const DEFAULT_SIZE: usize = 50;
#[allow(dead_code)]

/// This hashtable uses singly linked lists for its elements
pub struct SCHashTableImpl<K: Eq, V, S, B, A>
where
    B: Bucket<K, V, A>,
    A: Allocator + Clone,
{
    ptr: NonNull<B>,
    capacity: usize,
    len: usize,
    hash_builder: S,
    allocator: A,
    marker: PhantomData<(K, V)>,
}

impl<K, V, S, B, A> HashTableImpl<K, V, S, A> for SCHashTableImpl<K, V, S, B, A>
where
    S: BuildHasher,
    B: Bucket<K, V, A> + for<'a> BucketDrain<'a, K, V, A>,
    K: Eq + Hash,
    A: Allocator + Clone,
{
    fn with_capacity_and_hasher_in(
        capacity: usize,
        hash_builder: S,
        allocator: A,
    ) -> Result<Self, AllocError> {
        let ptr = if capacity != 0 {
            unsafe { Self::new_mem(capacity, allocator.clone()) }?
        } else {
            NonNull::dangling()
        };

        Ok(Self {
            ptr,
            capacity,
            len: 0,
            hash_builder,
            allocator,
            marker: PhantomData,
        })
    }
    fn grow(&mut self) -> Result<(), AllocError> {
        let new_capacity = if self.capacity == 0 {
            DEFAULT_SIZE
        } else {
            self.capacity * 2
        };

        let new_ptr = unsafe { Self::new_mem(new_capacity, self.allocator.clone())? };
        let (old_ptr, old_capacity) = (self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;

        // move elements from old area to new area and dealloc old area
        if old_capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(old_ptr.as_ptr(), old_capacity) {
                    if !bucket.is_empty() {
                        for (k, v) in bucket.drain() {
                            self.insert_unchecked(k, v)?;
                        }
                    }
                }
                self.allocator.deallocate(
                    old_ptr.cast(),
                    Layout::array::<NonNull<SLLBucket<K, V, A>>>(old_capacity).unwrap(),
                );
            }
        }
        Ok(())
    }
    unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Result<Option<V>, AllocError> {
        self.ptr
            .as_ptr()
            .add(self.key_index(&key))
            .as_mut()
            .unwrap_unchecked()
            .insert(key, value)
            .map(|s| s.map(|(_, v)| v))
    }

    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, AllocError> {
        if self.len + 1 > self.capacity() {
            self.grow()?;
        }
        let res = unsafe { self.insert_unchecked(key, value) }?;
        if res.is_none() {
            self.len += 1;
        }
        Ok(res)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = unsafe {
            self.ptr
                .as_ptr()
                .add(self.key_index(key))
                .as_mut()
                .unwrap_unchecked()
        };
        let res = bucket.remove(key).map(|c| c.1)?;
        self.len -= 1;
        Some(res)
    }
    fn get(&self, key: &K) -> Option<&V> {
        unsafe {
            self.ptr
                .as_ptr()
                .add(self.key_index(key))
                .as_mut()
                .unwrap_unchecked()
        }
        .get(key)
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn len(&self) -> usize {
        self.len
    }
    //fn set_capacity(&mut self) -> usize;
}

#[allow(dead_code)]
impl<K, V, S, B, A> SCHashTableImpl<K, V, S, B, A>
where
    S: BuildHasher,
    B: Bucket<K, V, A>,
    K: Eq,
    A: Allocator + Clone,
{
    unsafe fn new_mem(capacity: usize, allocator: A) -> Result<NonNull<B>, AllocError> {
        debug_assert!(capacity != 0);
        let layout = Layout::array::<B>(capacity).unwrap();

        let ptr: NonNull<MaybeUninit<B>> = allocator.allocate(layout)?.cast();
        let slice: &mut [MaybeUninit<B>] = slice::from_raw_parts_mut(ptr.as_ptr(), capacity);
        for i in 0..capacity {
            *slice.get_unchecked_mut(i) = MaybeUninit::new(B::new_in(allocator.clone()));
        }
        Ok(ptr.cast())
    }
}

impl<K: Eq + Hash, V, S: BuildHasher, B: Bucket<K, V, A>, A: Allocator + Clone>
    SCHashTableImpl<K, V, S, B, A>
{
    /*pub fn iter<'a>(&'a self) -> HashTableInnerIter<'a, K, V,A> {
        HashTableInnerIter {
            table: unsafe {slice::from_raw_parts(self.ptr.as_ptr(), self.capacity)},
            index: 0,
            capacity: self.capacity,
            current_iter: None,
        }
    }*/
    fn key_index(&self, k: &K) -> usize {
        let mut hasher = self.hash_builder.build_hasher();
        k.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/*

impl<K, V, S, A> SCHashTable<K, V, S, A>
where
    S: BuildHasher,
    K: Hash + Eq + Debug,
    A: Allocator + Clone
{
    fn key_index(&self, k: &K) -> usize {
        let mut hasher = self.hash_builder.build_hasher();
        k.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    fn grow(&mut self) -> Result<(), AllocError> {
        let new_capacity = if self.capacity == 0 {
            DEFAULT_SIZE
        } else {
            self.capacity * 2
        };

        let new_ptr = unsafe {Self::new_mem(new_capacity, &mut self.allocator)?};
        let (old_ptr, old_capacity) = (self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;

        // move elements from old area to new area and dealloc old area
        if old_capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(old_ptr.as_ptr(), old_capacity) {
                    if !bucket.is_empty() {
                        for elem in bucket.iter() {
                            self.insert_node_unchecked(elem);
                        }
                    }
                }
                self.allocator.deallocate(
                    old_ptr.cast(),
                    Layout::array::<NonNull<SLLBucket<K,V,A>>>(old_capacity).unwrap(),
                );
            }
        }
        Ok(())
    }

    pub fn get_node(&self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }
        let index = self.key_index(k);

        unsafe {
            let s = slice::from_raw_parts(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked(index);
            p.get(k)
        }
    }



    unsafe fn insert_node_unchecked(&mut self, o_node: ElementPtr<K,V>) {
        let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
        let p = s.get_unchecked_mut(self.key_index(&o_node.as_ref().key));
        p.insert_unchecked(o_node)
    }

    pub fn insert_node(&mut self, node: ElementPtr<K, V>) -> Option<ElementPtr<K, V>> {
        if self.len + 1 > self.capacity {
            self.grow().expect("failed");
        }
        unsafe {
            let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked_mut(self.key_index(&node.as_ref().key));
            let a = p.insert(node);
            if a.is_none() {
                self.len += 1;
            }
            a
        }
    }

    pub fn remove_node(&mut self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }

        let index = self.key_index(k);
        unsafe {
            let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked_mut(index);
            let a = p.remove(k);
            if a.is_some() {
                self.len -= 1;
            }
            a
        }
    }
}*/
/*impl<K,V,S: Default + BuildHasher, A: Allocator> Default for SCHashTable<K,V,S, A> {
    fn default() -> Self {
        Self::with_capacity_and_hasher_in(50, S::default(), De).unwrap()
    }
}

impl<K: Debug, V: Debug, H, A: Allocator+Clone> Debug for SCHashTable<K, V, H, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hashtable {{ capacity: {}, size: {}, elements: {{",
            &self.capacity, &self.len
        )?;
        unsafe {
            let mut iter = self.iter();
            if let Some(elem) = iter.next() {
                elem.as_ref().write(f)?;
            }
            for elem in iter {
                write!(f, ", ")?;
                elem.as_ref().write(f)?;
            }
        }

        write!(f, "}} }}")
    }
}*/

impl<K: Eq, V, S, B: Bucket<K, V, A>, A: Allocator + Clone> Drop
    for SCHashTableImpl<K, V, S, B, A>
{
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity) {
                    ptr::drop_in_place(bucket as *mut B);
                    /*bucket.iter().for_each(|s| {
                        let layout = Layout::new::<SinglyLinkedList<K, V,A>>();

                        self.allocator.deallocate(s.cast(), layout);
                    })*/
                }
                let self_layout = Layout::array::<ElementsPtr<K, V, A>>(self.capacity).unwrap();
                self.allocator.deallocate(self.ptr.cast(), self_layout);
            }
        }
    }
}
// ------------------------------------------ ITER ---------------------------------------------
/*
pub struct HashTableInnerIter<'a, K, V, A: Allocator+Clone> {
    table: &'a [SLLBucket<K,V,A>],
    index: usize,
    capacity: usize,
    current_iter: Option<BucketIter<'a, K,V,A>>,
}

impl<'a, K, V,A: Allocator+Clone> Iterator for HashTableInnerIter<'a, K, V,A> {
    type Item = ElementPtr<K,V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.capacity {
            return None
        }

        if let Some(iter) = self.current_iter.as_mut() {
            if let Some(elem) = iter.next() {
                return Some(elem)
            } else {
                self.current_iter = None;
            }
        }

        // only gets here if current_iter is none
        // since current_iter is None find the next one
        let mut iter = loop {
            // the if shouldn't get to the get_unchecked if its going to be out of bounds.
            if self.index < self.capacity {
                if unsafe {!self.table.get_unchecked(self.index).is_empty()} {
                    break unsafe{self.table.get_unchecked(self.index).iter()};
                } else {
                    self.index += 1;
                }
            } else {
                self.index += 1;
                return None;
            }
            self.index += 1;
        };
        // this should always be some since .is_empty() is checked
        let next = iter.next();
        self.current_iter = Some(iter);
        next
    }
}



*/

/*
use core::{
    fmt::{self, Debug, Formatter},
    ptr::{self, *},
    marker::PhantomData,
    mem::MaybeUninit,
    slice,
    alloc::{
        Allocator, Layout, AllocError
    },
    hash::{BuildHasher, Hash, Hasher},
};

mod buckets;
use buckets::*;

use super::HashTable;

impl<K, V, S, A> HashTable<K, V, S, A> for SCHashTable<K, V, S, A>
where
    A: Allocator + Clone,
    K: Hash + Eq + Debug,
    S: BuildHasher + Default,
    V: Debug,
{
    fn with_capacity_and_hasher_in(capacity: usize, hash_builder: S, alloc: A) -> Result<Self, AllocError> {
        Self::with_capacity_and_hasher_in(capacity, hash_builder, alloc)
    }
    fn insert(&mut self, k: K, v: V) -> Result<Option<V>, AllocError> {

        unsafe {
            let node = SinglyLinkedList::ptr_to_new(k, v, self.allocator.clone())?;
            Ok(self
                .insert_node(node)
                .map(|v| SinglyLinkedList::into_tuple(v, self.allocator.clone()).1))
        }
    }

    fn get<'a>(&'a self, k: &K) -> Option<&'a V> {
        self.get_node(k).map(|s| &unsafe { s.as_ref() }.val)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        self
            .remove_node(k)
            .map(|v| unsafe { SinglyLinkedList::into_tuple(v, self.allocator.clone()).1 })
    }

    fn len(&self) -> usize {
        self.len()
    }
}

type ElementsPtr<K, V,A> = NonNull<SLLBucket<K,V,A>>;
pub type ElementPtr<K, V> = NonNull<SinglyLinkedList<K, V>>;
const DEFAULT_SIZE: usize = 50;
#[allow(dead_code)]

/// This hashtable uses singly linked lists for its elements
pub struct SCHashTable<K, V, S, A>
where
    A: Allocator + Clone,
{
    ptr: NonNull<SLLBucket<K,V,A>>,
    capacity: usize,
    len: usize,
    hash_builder: S,
    allocator: A,
}

/*impl<K,V,S: Default + BuildHasher, A: Allocator> Default for SCHashTable<K,V,S, A> {
    fn default() -> Self {
        Self::with_capacity_and_hasher_in(50, S::default(), De).unwrap()
    }
}*/

impl<K: Debug, V: Debug, H, A: Allocator+Clone> Debug for SCHashTable<K, V, H, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hashtable {{ capacity: {}, size: {}, elements: {{",
            &self.capacity, &self.len
        )?;
        unsafe {
            let mut iter = self.iter();
            if let Some(elem) = iter.next() {
                elem.as_ref().write(f)?;
            }
            for elem in iter {
                write!(f, ", ")?;
                elem.as_ref().write(f)?;
            }
        }

        write!(f, "}} }}")
    }
}

#[allow(dead_code)]
impl<K, V, S, A> SCHashTable<K, V, S, A>
where
    S: BuildHasher,
    A: Allocator + Clone,
{
    pub fn with_capacity_and_hasher_in(capacity: usize, hash_builder: S, mut allocator: A) -> Result<Self, AllocError> {
        let ptr = if capacity != 0 {
            unsafe { Self::new_mem(capacity, &mut allocator)}?
        } else {
            NonNull::dangling()
        };

        Ok(Self {
            ptr,
            capacity,
            len: 0,
            hash_builder,
            allocator,
        })
    }

    unsafe fn new_mem(capacity: usize, allocator: &mut A) -> Result<ElementsPtr<K,V,A>, AllocError> {
        debug_assert!(capacity != 0);
        let layout = Layout::array::<SLLBucket<K,V,A>>(capacity).unwrap();

        let ptr: NonNull<MaybeUninit<SLLBucket<K,V,A>>> = allocator.allocate(layout)?.cast();
        let slice: &mut [MaybeUninit<SLLBucket<K,V,A>>] = slice::from_raw_parts_mut(ptr.as_ptr(), capacity);
        for i in 0..capacity {
            *slice.get_unchecked_mut(i) = MaybeUninit::new(SLLBucket::new_in(allocator.clone()));
        }
        Ok(ptr.cast())

    }
}

impl<K,V,H,A: Allocator+Clone> SCHashTable<K,V,H,A> {
    pub fn iter<'a>(&'a self) -> HashTableInnerIter<'a, K, V,A> {
        HashTableInnerIter {
            table: unsafe {slice::from_raw_parts(self.ptr.as_ptr(), self.capacity)},
            index: 0,
            capacity: self.capacity,
            current_iter: None,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K, V, S, A> SCHashTable<K, V, S, A>
where
    S: BuildHasher,
    K: Hash + Eq + Debug,
    A: Allocator + Clone
{
    fn grow(&mut self) -> Result<(), AllocError> {
        let new_capacity = if self.capacity == 0 {
            DEFAULT_SIZE
        } else {
            self.capacity * 2
        };

        let new_ptr = unsafe {Self::new_mem(new_capacity, &mut self.allocator)?};
        let (old_ptr, old_capacity) = (self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;

        // move elements from old area to new area and dealloc old area
        if old_capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(old_ptr.as_ptr(), old_capacity) {
                    if !bucket.is_empty() {
                        for elem in bucket.iter() {
                            self.insert_node_unchecked(elem);
                        }
                    }
                }
                self.allocator.deallocate(
                    old_ptr.cast(),
                    Layout::array::<NonNull<SLLBucket<K,V>>>(old_capacity).unwrap(),
                );
            }
        }
        Ok(())
    }



    pub fn get_node(&self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }
        let index = self.key_index(k);

        unsafe {
            let s = slice::from_raw_parts(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked(index);
            p.get(k)
        }
    }

    fn key_index(&self, k: &K) -> usize {
        let mut hasher = self.hash_builder.build_hasher();
        k.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    unsafe fn insert_node_unchecked(&mut self, o_node: ElementPtr<K,V>) {
        let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
        let p = s.get_unchecked_mut(self.key_index(&o_node.as_ref().key));
        p.insert_unchecked(o_node)
    }

    pub fn insert_node(&mut self, node: ElementPtr<K, V>) -> Option<ElementPtr<K, V>> {
        if self.len + 1 > self.capacity {
            self.grow().expect("failed");
        }
        unsafe {
            let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked_mut(self.key_index(&node.as_ref().key));
            let a = p.insert(node);
            if a.is_none() {
                self.len += 1;
            }
            a
        }
    }

    pub fn remove_node(&mut self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }

        let index = self.key_index(k);
        unsafe {
            let s = slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity);
            let p = s.get_unchecked_mut(index);
            let a = p.remove(k);
            if a.is_some() {
                self.len -= 1;
            }
            a
        }
    }
}

impl<K, V, H, A: Allocator+Clone> Drop for SCHashTable<K, V, H, A> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity) {
                    bucket.iter().for_each(|s| {
                        let layout = Layout::new::<SinglyLinkedList<K, V>>();
                        ptr::drop_in_place(s.as_ptr());
                        self.allocator.deallocate(s.cast(), layout);
                    })
                }
                let self_layout = Layout::array::<ElementsPtr<K, V,A>>(self.capacity).unwrap();
                self.allocator.deallocate(self.ptr.cast(), self_layout);
            }
        }
    }
}

pub struct HashTableInnerIter<'a, K, V, A: Allocator+Clone> {
    table: &'a [SLLBucket<K,V,A>],
    index: usize,
    capacity: usize,
    current_iter: Option<BucketIter<'a, K,V,A>>,
}

impl<'a, K, V,A: Allocator+Clone> Iterator for HashTableInnerIter<'a, K, V,A> {
    type Item = ElementPtr<K,V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.capacity {
            return None
        }

        if let Some(iter) = self.current_iter.as_mut() {
            if let Some(elem) = iter.next() {
                return Some(elem)
            } else {
                self.current_iter = None;
            }
        }

        // only gets here if current_iter is none
        // since current_iter is None find the next one
        let mut iter = loop {
            // the if shouldn't get to the get_unchecked if its going to be out of bounds.
            if self.index < self.capacity {
                if unsafe {!self.table.get_unchecked(self.index).is_empty()} {
                    break unsafe{self.table.get_unchecked(self.index).iter()};
                } else {
                    self.index += 1;
                }
            } else {
                self.index += 1;
                return None;
            }
            self.index += 1;
        };
        // this should always be some since .is_empty() is checked
        let next = iter.next();
        self.current_iter = Some(iter);
        next
    }
}*/

/*



#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::Global;
    use std::collections::hash_map::RandomState;
    use crate::seperate_chaining::singly_linked_list::*;

    #[test]
    #[cfg(miri)]
    fn create_and_drop_inner() {
        let _a = hashtable_with_capacity::<i32, i32>(0);
    }

    #[test]
    #[cfg(miri)]
    fn create_with_capacity_inner() {
        let _a = hashtable_with_capacity::<i32, i32>(100);
    }

    #[test]
    #[cfg(miri)]
    fn insert_growth_drop_inner() {
        let mut a = hashtable_with_capacity::<i32, i32>(5);
        for i in 0..5 {
            let ptr = unsafe { SinglyLinkedList::ptr_to_new(i, i) };
            a.insert_node(ptr);
        }
    }

    #[test]
    fn insert_drop_inner() {
        let mut a = hashtable_with_capacity(0);
        let ptr = unsafe { SinglyLinkedList::ptr_to_new(1, 1, Global) };
        a.insert_node(ptr);
    }

    #[test]
    fn remove_nothing_inner() {
        let mut a = hashtable_with_capacity::<i32, i32>(0);
        assert!(a.remove(&1).is_none());
    }

    #[test]
    fn insert_remove_inner() {
        type KV = i32;
        let mut a = hashtable_with_capacity(0);
        let ptr = unsafe { SinglyLinkedList::ptr_to_new(1, 1, Global) };
        a.insert_node(ptr);
        let ptr = a.remove_node(&1).unwrap();
        let v = unsafe { &ptr.as_ref().val };
        assert_eq!(v, &1);
        unsafe { ptr::drop_in_place(ptr.as_ptr()) };
        unsafe {
            std::alloc::dealloc(
                ptr.cast().as_ptr(),
                Layout::new::<SinglyLinkedList<KV, KV>>(),
            )
        };
    }

    #[test]
    fn inner_iter_test() {
        type KV = i32;

        unsafe {
            const CAP: KV = 25;
            let mut a = hashtable_with_capacity(0);
            let v: Vec<KV> = (0..CAP).collect::<Vec<_>>();
            for i in v.iter().copied() {
                let ptr = SinglyLinkedList::ptr_to_new(i, i);
                a.insert(ptr);
            }

            println!("{:?}", &a);

            let mut u: Vec<KV> = a.iter().map(|x| x.val).collect();
            u.sort();

            assert_eq!(v, u);
        }
    }

    #[test]
    #[cfg(miri)]
    fn empty_iter() {
        let a: SCHashTable<(), ()> = hashtable_with_capacity(0);
        let _u = a.iter().collect::<Vec<_>>();
    }

    /// set capacity to 0 to get the equivivelant of Hashtable::new()
    fn hashtable_with_capacity<K: Eq + Hash, V>(capacity: usize) -> SLLHashTableImpl<K, V, RandomState, Global> {
        SLLHashTableImpl::with_capacity_and_hasher_in(capacity, RandomState::new(), Global).expect("failed alloc")
    }
}*/

#[cfg(test)]
#[cfg(not(miri))]
mod bench {
    use std::collections::hash_map::RandomState;

    use super::*;
    use std::alloc::Global;
    use test::Bencher;
    #[bench]
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
    type SCSLLHashTableImpl<K, V, S, A> = SCHashTableImpl<K, V, S, SLLBucket<K, V, A>, A>;

    #[bench]
    fn seperate_chaining_linked_list_insert_bench(b: &mut Bencher) {
        let mut h: SCSLLHashTableImpl<i64, i64, RandomState, Global> =
            SCSLLHashTableImpl::with_capacity_and_hasher_in(0, RandomState::new(), Global)
                .expect("failed_alloc");
        let mut i: i64 = 0;
        b.iter(|| {
            h.insert(i, i).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn seperate_chaining_linked_list_insert_bench_get_bench(b: &mut Bencher) {
        let mut h: SCSLLHashTableImpl<i64, i64, RandomState, Global> =
            SCSLLHashTableImpl::with_capacity_and_hasher_in(0, RandomState::new(), Global)
                .expect("failed_alloc");
        for i in 0..10000 {
            h.insert(i, i).unwrap();
        }
        let mut i: i64 = 0;
        b.iter(|| {
            h.get(&i);
            i += 1;
        })
    }
}
