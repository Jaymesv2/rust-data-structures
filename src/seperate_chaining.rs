use core::{
    fmt::{self, Debug, Formatter},
    ptr::{self, *}, 
    marker::PhantomData,
    mem::MaybeUninit,
    slice
};

use std::{
    alloc::{alloc, dealloc, Layout},
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash, Hasher},
};

use super::HashTable;

impl<K, V, S> HashTable<K, V, S> for SCHashTable<K, V, S>
where
    K: Hash + Eq + Debug,
    S: BuildHasher + Default,
    V: Debug,
{
    fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self::with_capacity_and_hasher(capacity, hash_builder)
    }
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        unsafe {
            self
                .insert_node(SinglyLinkedList::ptr_to_new(k, v))
                .map(|v| SinglyLinkedList::extract_val_from_sll(v))
        }
    }

    fn get<'a>(&'a self, k: &K) -> Option<&'a V> {
        self.get_node(k).map(|s| &unsafe { s.as_ref() }.val)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        self
            .remove_node(k)
            .map(|v| unsafe { SinglyLinkedList::extract_val_from_sll(v) })
    }

    fn len(&self) -> usize {
        self.len()
    }
}
/* 
 fn iter<'a>(&'a self) -> HashTableIter<'a, K, V> {
        HashTableIter {
            inner: self.inner.iter(),
        }
    }
*/

type ElementsPtr<K, V> = NonNull<Bucket<K,V>>;
pub type ElementPtr<K, V> = NonNull<SinglyLinkedList<K, V>>;
const DEFAULT_SIZE: usize = 50;
#[allow(dead_code)]
/// This hashtable uses singly linked lists for its elements
pub struct SCHashTable<K, V, S = RandomState> {
    ptr: NonNull<Bucket<K,V>>,
    capacity: usize,
    len: usize,
    hash_builder: S,
}

impl<K,V,S: Default + BuildHasher> Default for SCHashTable<K,V,S> {
    fn default() -> Self {
        Self::with_capacity_and_hasher(50, S::default())
    }
}

//impl <K,V,RandomState> Default for HashTableInner<K,V,RandomState> {}


impl<K: Debug, V: Debug, H> Debug for SCHashTable<K, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
impl<K, V, S> SCHashTable<K, V, S>
where
    S: BuildHasher,
{
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let ptr = if capacity != 0 {
            unsafe { Self::new_mem(capacity)}
        } else {
            NonNull::dangling()
        };

        Self {
            ptr,
            capacity,
            len: 0,
            hash_builder,
        }
    }

    unsafe fn new_mem(capacity: usize) -> ElementsPtr<K,V> {
        debug_assert!(capacity != 0);
        let layout = Layout::array::<Bucket<K,V>>(capacity).unwrap();
        
        let ptr = alloc(layout) as *mut MaybeUninit<Bucket<K,V>>;
        let slice: &mut [MaybeUninit<Bucket<K,V>>] = slice::from_raw_parts_mut(ptr, capacity);
        for i in 0..capacity {
            *slice.get_unchecked_mut(i) = MaybeUninit::new(Bucket::default());
        }
        NonNull::new(ptr).unwrap().cast()
        
    }
}

impl<K,V,H> SCHashTable<K,V,H> {
    pub fn iter<'a>(&'a self) -> HashTableInnerIter<'a, K, V> {
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

impl<K, V, S> SCHashTable<K, V, S>
where
    S: BuildHasher,
    K: Hash + Eq + Debug,
{
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            DEFAULT_SIZE
        } else {
            self.capacity * 2
        };

        let new_ptr = unsafe {Self::new_mem(new_capacity)};
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
                dealloc(
                    old_ptr.cast().as_ptr(),
                    Layout::array::<NonNull<Bucket<K,V>>>(old_capacity).unwrap(),
                );
            }
        }
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
            self.grow();
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

impl<K, V, H> Drop for SCHashTable<K, V, H> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                for bucket in slice::from_raw_parts_mut(self.ptr.as_ptr(), self.capacity) {                
                    bucket.iter().for_each(|s| {
                        let layout = Layout::new::<SinglyLinkedList<K, V>>();
                        ptr::drop_in_place(s.as_ptr());
                        dealloc(s.cast().as_ptr(), layout);
                    })
                }
                let self_layout = Layout::array::<ElementsPtr<K, V>>(self.capacity).unwrap();
                dealloc(self.ptr.cast().as_ptr(), self_layout);
            }
        }
    }
}

pub struct HashTableInnerIter<'a, K, V> {
    table: &'a [Bucket<K,V>],
    index: usize,
    capacity: usize,
    current_iter: Option<BucketIter<'a, K,V>>,
}

impl<'a, K, V> Iterator for HashTableInnerIter<'a, K, V> {
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

/*
trait Bucket<K,V>: Default {
    type Iter: Iterator<Item = ElementPtr<K,V>>;
    fn insert(&mut self, node: ElementPtr<K, V>) -> Option<ElementPtr<K,V>>;
    fn get(&self, key: &K) -> Option<ElementPtr<K, V>>;
    fn remove(&self, key: &K) -> Option<ElementPtr<K, V>>;
    fn contains(&self, key: &K) -> bool;
    fn iter(&self) -> Self::Iter;
    //pub fn drain(&mut self) -> impl Iterator<Item = ElementPtr<K,V>> {todo!()}
}*/

#[repr(transparent)]
struct Bucket<K, V> {
    head: Option<ElementPtr<K, V>>,
}

impl<K,V> Default for Bucket<K,V> {
    fn default() -> Self {
        Self { head: None }
    }
}

impl<K,V> Bucket<K,V> {
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    
    pub fn iter<'a>(&'a self) -> BucketIter<'a, K, V> {
        BucketIter {
            head: self.head,
            marker: PhantomData
        }
    }
}

impl<K: Eq, V> Bucket<K, V> {
    pub fn insert(&mut self, node: ElementPtr<K, V>) -> Option<ElementPtr<K,V>> {
        unsafe {
            let rem = self.remove(&node.as_ref().key);
            self.insert_unchecked(node);
            rem
        }
    }
    /// inserts an element at the front of the list without checking for duplicates
    pub fn insert_unchecked(&mut self, mut node: ElementPtr<K,V>) {
        unsafe {node.as_mut().next = self.head};
        self.head = Some(node);
    }

    pub fn get(&self, key: &K) -> Option<ElementPtr<K, V>> {
        self.iter().find(|elem| unsafe{&elem.as_ref().key} == key)
    }

    pub fn remove(&mut self, key: &K) -> Option<ElementPtr<K, V>> {
        unsafe {
            let mut prev: Option<ElementPtr<K, V>> = None;
            let mut head = self.head;
            while let Some(elem) = head {
                if &elem.as_ref().key == key {
                    let next = elem.as_ref().next;
                    if let Some(mut parent_ptr) = prev {
                        parent_ptr.as_mut().next = next;
                    } else {
                        self.head = next;
                    }
                    return Some(elem);
                }
                prev = Some(elem);
                head = elem.as_ref().next;
            }
        }
        None
    }

    //pub fn contains(&self, key: &K) -> bool {self.get(key).is_some()}
}

struct BucketIter<'a, K,V> {
    head: Option<ElementPtr<K,V>>,
    marker: PhantomData<&'a Bucket<K,V>>
}

impl<'a, K,V> Iterator for BucketIter<'a, K,V> {
    type Item = ElementPtr<K,V>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.head {
            self.head = unsafe {s.as_ref().next};
            Some(s)
        } else {
            None
        }
    }
}

/* 
impl<K,V> Drop for Bucket<K,V> {
    fn drop(&mut self) {
        unsafe {
            let mut head = self.head;
            while let Some(s) = head {
                head = s.as_ref().next;
                let layout = Layout::new::<SinglyLinkedList<K, V>>();
                ptr::drop_in_place(s.as_ptr());
                dealloc(s.cast().as_ptr(), layout);
            }
        }
    }
}*/

pub struct SinglyLinkedList<K, V> {
    pub key: K,
    pub val: V,
    next: Option<ElementPtr<K, V>>,
}

impl<K, V> SinglyLinkedList<K, V> {
    pub fn new(k: K, v: V) -> Self {
        Self {
            key: k,
            val: v,
            next: None,
        }
    }
    /// # Safety
    /// the caller is in charge of deallocation
    pub unsafe fn ptr_to_new(k: K, v: V) -> ElementPtr<K, V> {
        let layout = Layout::new::<Self>();
        let ptr = alloc(layout) as *mut Self;
        let i = Self::new(k, v);
        std::ptr::write(ptr, i);
        NonNull::new_unchecked(ptr)
    }

    pub unsafe fn extract_val_from_sll(mut s: ElementPtr<K, V>) -> V {
        let v: V = ptr::read(&s.as_ref().val);
        ptr::drop_in_place(&mut s.as_mut().key as *mut K);
        dealloc(s.cast().as_ptr(), Layout::new::<SinglyLinkedList<K, V>>());
        v
    }
}

impl<K: Debug, V: Debug> SinglyLinkedList<K, V> {
    fn write(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", &self.key, &self.val)?;
        if let Some(s) = self.next.map(|c| unsafe {c.as_ref()}) {
            write!(f, ", next_key: {:?}", &s.key)
        } else {
            write!(f, ")")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let ptr = unsafe { SinglyLinkedList::ptr_to_new(1, 1) };
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
        let ptr = unsafe { SinglyLinkedList::ptr_to_new(1, 1) };
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
    /* 
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
    }*/

    #[test]
    #[cfg(miri)]
    fn empty_iter() {
        let a: SCHashTable<(), ()> = hashtable_with_capacity(0);
        let _u = a.iter().collect::<Vec<_>>();
    }

    /// set capacity to 0 to get the equivivelant of Hashtable::new()
    fn hashtable_with_capacity<K, V>(capacity: usize) -> SCHashTable<K, V, RandomState> {
        SCHashTable::with_capacity_and_hasher(capacity, RandomState::new())
    }
}


#[cfg(test)]
#[cfg(not(miri))]
mod bench {
    use super::*;
    use crate::seperate_chaining::SCHashTable;
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
    fn local_insert_bench(b: &mut Bencher) {
        let mut h: SCHashTable<i64, i64, RandomState> = SCHashTable::default();
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

    #[bench]
    fn local_get_bench(b: &mut Bencher) {
        let mut h: SCHashTable<i64, i64, RandomState> = SCHashTable::new();
        for i in 0..10000 {
            h.insert(i, i);
        }
        let mut i: i64 = 0;
        b.iter(|| {
            h.get(&i);
            i += 1;
        })
    }
}

#[cfg(test)]
mod pub_api_tests {
    use super::*;

    #[test]
    #[cfg(miri)]
    fn create_and_drop_inner() {
        let _a = SCHashTable::<i32, i32>::new();
    }

    #[test]
    #[cfg(miri)]
    fn create_with_capacity_inner() {
        let _a = SCHashTable::<i32, i32>::with_capacity(100);
    }

    #[test]
    #[cfg(miri)]
    fn insert_growth_drop_inner() {
        let mut a = SCHashTable::with_capacity(5);
        for i in 0..5 {
            a.insert(1, 1);
        }
    }

    #[test]
    fn insert_drop_inner() {
        let mut a = SCHashTable::<_, _, RandomState>::new();
        a.insert(1, 1);
    }

    #[test]
    fn remove_nothing_inner() {
        let mut a = SCHashTable::<_, i32>::new();
        assert!(a.remove(&1).is_none());
    }

    #[test]
    fn insert_remove_inner() {
        let mut a = SCHashTable::<_, _, RandomState>::with_capacity(0);
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

        let mut a = SCHashTable::<_, _, RandomState>::with_capacity(CAPACITY);
        for i in v.iter() {
            a.insert(*i, *i);
        }

        let mut r: Vec<u64> = Vec::with_capacity(CAPACITY);
        for i in v.iter() {
            r.push(*a.get(i).unwrap());
        }

        assert_eq!(r, v);
    }
    /* 
    #[test]
    fn iter_test() {
        const CAPACITY: usize = 10;

        let mut h = SCHashTable::with_capacity(CAPACITY);
        let v = (0..CAPACITY).collect::<Vec<_>>();
        for i in v.iter().copied() {
            h.insert(i, i);
        }

        let mut u = h.iter().map(|c| c.1).copied().collect::<Vec<_>>();
        u.sort();

        assert_eq!(u, v)
    }*/
}
