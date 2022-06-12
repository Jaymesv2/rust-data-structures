use std::{
    alloc::{alloc, dealloc, Layout},
    //mem,
    collections::hash_map::RandomState,
    fmt::Formatter,
    fmt::{self, Debug},
    hash::{BuildHasher, Hash, Hasher},
    ptr::{self, *},
};

pub type ElementsPtr<K, V> = NonNull<Option<ElementPtr<K, V>>>;
pub type ElementPtr<K, V> = NonNull<SinglyLinkedList<K, V>>;
const DEFAULT_SIZE: usize = 50;
#[allow(dead_code)]
/// This hashtable uses singly linked lists for its elements
pub struct HashTableInner<K, V, S = RandomState> {
    ptr: ElementsPtr<K, V>,
    capacity: usize,
    size: usize,
    hash_builder: S,
}

impl<K: Debug, V: Debug, H> Debug for HashTableInner<K, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hashtable, capacity: {}, size: {}, {{",
            &self.capacity, &self.size
        )?;
        let mut item = self.ptr.as_ptr();
        // print elements of the list
        unsafe {
            for _ in 0..self.capacity {
                write!(f, "{{")?;
                let mut head = *item;
                while let Some(elem) = head {
                    elem.as_ref().write(f)?;
                    write!(f, ", ")?;
                    head = elem.as_ref().next;
                }
                item = item.add(1);
                write!(f, "}}, ")?;
            }
        }
        write!(f, "}}")
    }
}

#[allow(dead_code)]
impl<K, V, S> HashTableInner<K, V, S>
where
    S: BuildHasher,
{
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let ptr = if capacity != 0 {
            let layout = Layout::array::<ElementsPtr<K, V>>(capacity).unwrap();
            unsafe {
                let ptr = NonNull::new(alloc(layout))
                    .expect("allocator returned null ptr")
                    .cast();
                Self::clear_mem(ptr, capacity);
                ptr
            }
        } else {
            NonNull::dangling()
        };

        Self {
            ptr,
            capacity,
            size: 0,
            hash_builder,
        }
    }

    unsafe fn clear_mem(ptr: ElementsPtr<K, V>, capacity: usize) {
        let mut ptr = ptr.as_ptr();
        for _ in 0..capacity {
            *ptr = None;
            ptr = ptr.add(1);
        }
    }
}

impl<K, V, S> HashTableInner<K, V, S>
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

        let layout = Layout::array::<ElementsPtr<K, V>>(new_capacity).unwrap();
        let new_ptr = unsafe {
            let p = alloc(layout) as *mut Option<NonNull<SinglyLinkedList<K, V>>>;
            NonNull::new(p).unwrap()
        };

        unsafe { Self::clear_mem(new_ptr, new_capacity) };

        let (old_ptr, old_capacity) = (self.ptr, self.capacity);

        self.ptr = new_ptr;
        self.capacity = new_capacity;

        

        // move elements from old area to new area and dealloc old area
        if old_capacity != 0 {
            unsafe {
                let mut current = old_ptr.as_ptr();
                for _ in 0..old_capacity {
                    let mut head = *current;
                    while let Some(mut elem) = head {
                        head = elem.as_ref().next;
                        
                        let index = self.key_index(&elem.as_ref().key);

                        let p = self.ptr.as_ptr().add(index);
                        elem.as_mut().next = *p;
                        *p = Some(elem);
                    }

                    current = current.add(1)
                }
                dealloc(
                    old_ptr.cast().as_ptr(),
                    Layout::array::<ElementsPtr<K, V>>(old_capacity).unwrap(),
                );
            }
        }
    }

    pub fn insert(&mut self, ptr: ElementPtr<K, V>) -> Option<ElementPtr<K, V>> {
        if self.size + 1 > self.capacity {
            self.grow();
        }
        unsafe {
            let a = self.insert_node(ptr);
            self.size += 1;
            a
        }
    }

    pub fn get(&self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }
        let index = self.key_index(k);
        unsafe {
            let mut head = *self.ptr.as_ptr().add(index);
            while let Some(elem) = head {
                let r = elem.as_ref();
                if &r.key == k {
                    return Some(elem);
                }
                head = elem.as_ref().next;
            }
        }
        None
    }

    fn key_index(&self, k: &K) -> usize {
        let mut hasher = self.hash_builder.build_hasher();
        k.hash(&mut hasher);
        hasher.finish() as usize % self.capacity
    }

    /// clears the next element from the sll
    unsafe fn insert_node(&mut self, mut o_node: ElementPtr<K, V>) -> Option<ElementPtr<K, V>> {
        let p = self.ptr.as_ptr().add(self.key_index(&o_node.as_ref().key));
        match *p {
            Some(mut prev) if prev.as_ref().key != o_node.as_ref().key => {
                while let Some(mut cur) = prev.as_ref().next {
                    if cur.as_ref().key == o_node.as_ref().key {
                        o_node.as_mut().next = cur.as_ref().next;
                        cur.as_mut().next = None;
                        prev.as_mut().next = Some(o_node);
                        
                        return Some(cur);
                    }
                    prev = cur;
                }
                
                debug_assert!(prev.as_ref().next.is_none());
                prev.as_mut().next = Some(o_node);
                o_node.as_mut().next = None;
                None
            }
            s => {
                o_node.as_mut().next = None;
                *p = Some(o_node);
                s
            }
        }
    }

    pub fn remove(&mut self, k: &K) -> Option<ElementPtr<K, V>> {
        if self.capacity == 0 {
            return None;
        }

        let index = self.key_index(k);

        unsafe {
            let mut prev: Option<ElementPtr<K, V>> = None;
            let mut head = *self.ptr.as_ptr().add(index);
            while let Some(elem) = head {
                if &elem.as_ref().key == k {
                    let next = elem.as_ref().next;
                    if let Some(mut parent_ptr) = prev {
                        parent_ptr.as_mut().next = next;
                    } else {
                        *self.ptr.as_ptr().add(index) = next;
                    }
                    return Some(elem);
                }
                prev = Some(elem);
                head = elem.as_ref().next;
            }
        }
        None
    }

    pub fn iter<'a>(&'a self) -> HashTableInnerIter<'a, K,V,S> {
        let mut index = 0;

        let current_node = if self.capacity != 0 {
            loop {
                if index+1 > self.capacity {
                    break None
                }
                if let Some(s) = unsafe {*self.ptr.as_ptr().add(index)} {
                    index += 1;
                    break Some(s)
                }
                index += 1;
            }
        } else {
            None
        };

        HashTableInnerIter { 
            table: self, 
            index, 
            current_node, 
        }
    }
}

impl<K, V, H> Drop for HashTableInner<K, V, H> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                let mut ptr = self.ptr.as_ptr();
                for _ in 0..self.capacity {
                    let mut head = *ptr;
                    while let Some(s) = head {
                        head = s.as_ref().next;
                        let layout = Layout::new::<SinglyLinkedList<K, V>>();
                        ptr::drop_in_place(s.as_ptr());
                        dealloc(s.cast().as_ptr(), layout);
                    }
                    ptr = ptr.add(1);
                }
                let self_layout = Layout::array::<ElementsPtr<K, V>>(self.capacity).unwrap();
                dealloc(self.ptr.cast().as_ptr(), self_layout);
            }
        }
    }
}

pub struct HashTableInnerIter<'a, K,V,S> {
    table: &'a HashTableInner<K,V,S>,
    index: usize,
    current_node: Option<ElementPtr<K,V>>,
}

impl<'a, K, V,S> Iterator for HashTableInnerIter<'a, K,V,S> {
    type Item = &'a SinglyLinkedList<K,V>;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(x) = self.current_node.map(|x| x.as_ref()) {
                if let Some(y) = x.next {
                    self.current_node = Some(y)
                } else {
                    self.current_node = loop {
                        if self.index+1 > self.table.capacity {
                            break None
                        }
                        if let Some(s) = *self.table.ptr.as_ptr().add(self.index) {
                            self.index += 1;
                            break Some(s)
                        }
                        self.index += 1;
                    };
                }
                Some(x)
            } else {
                None
            }
        }
    }
}

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

    pub unsafe fn extract_val_from_sll(mut s: ElementPtr<K,V>) -> V {
        let v: V = ptr::read(&s.as_ref().val);
        ptr::drop_in_place(&mut s.as_mut().key as *mut K);
        dealloc(s.cast().as_ptr(), Layout::new::<SinglyLinkedList<K, V>>());
        v
    }
}


impl<K: Debug, V: Debug> SinglyLinkedList<K, V> {
    fn write(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", &self.key, &self.val)
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
            a.insert(ptr);
        }
    }
    
    #[test]
    fn insert_drop_inner() {
        let mut a = hashtable_with_capacity(0);
        let ptr = unsafe { SinglyLinkedList::ptr_to_new(1, 1) };
        a.insert(ptr);
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
        a.insert(ptr);
        let ptr = a.remove(&1).unwrap();
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

            assert_eq!(v,u);
        }
    }

    #[test]
    #[cfg(miri)]
    fn empty_iter() {
        let a: HashTableInner<(), ()> = hashtable_with_capacity(0);
        let _u = a.iter().collect::<Vec<_>>();
    }

    /// set capacity to 0 to get the equivivelant of Hashtable::new()
    fn hashtable_with_capacity<K, V>(capacity: usize) -> HashTableInner<K, V, RandomState> {
        HashTableInner::with_capacity_and_hasher(capacity, RandomState::new())
    }
}
