use crate::prelude::*;

use core::{
    alloc::{AllocError, Allocator, Layout},
    hash::Hash,
    marker::PhantomData,
    ptr::{self, drop_in_place, NonNull},
};

use crate::traits::hash_table::seperate_chaining::bucket::*;

pub type ElementPtr<T> = NonNull<SinglyLinkedListNode<T>>;

pub struct UnsafeSinglyLinkedList<T, A: Allocator + Clone> {
    head: Option<ElementPtr<T>>,
    alloc: A,
}

impl<K, V, A> Bucket<K, V, A> for UnsafeSinglyLinkedList<(K, V), A>
where
    K: Eq + Hash,
    A: Allocator + Clone,
{
    fn new_in(alloc: A) -> Self {
        Self { head: None, alloc }
    }

    fn get<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.iter().find(|elem| &elem.0 == key).map(|(_, v)| v)
    }

    fn insert(&mut self, key: K, value: V) -> Result<Option<(K, V)>, AllocError> {
        let mut node =
            unsafe { SinglyLinkedListNode::ptr_to_new((key, value), self.alloc.clone()) }?;
        Ok(unsafe {
            let rem = self.remove(&node.as_ref().val.0);
            node.as_mut().next = self.head;
            self.head = Some(node);
            rem
        })
    }

    unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Result<(), AllocError> {
        let mut node = SinglyLinkedListNode::ptr_to_new((key, value), self.alloc.clone())?;
        node.as_mut().next = self.head;
        self.head = Some(node);
        Ok(())
    }

    fn remove(&mut self, key: &K) -> Option<(K, V)> {
        let mut ptr = None;
        unsafe {
            let mut prev: Option<ElementPtr<(K, V)>> = None;
            let mut head = self.head;
            while let Some(elem) = head {
                if &elem.as_ref().val.0 == key {
                    let next = elem.as_ref().next;
                    if let Some(mut parent_ptr) = prev {
                        parent_ptr.as_mut().next = next;
                    } else {
                        self.head = next;
                    }
                    ptr = Some(elem);
                    break;
                }
                prev = Some(elem);
                head = elem.as_ref().next;
            }
        };
        ptr.map(|c| unsafe { SinglyLinkedListNode::into_tuple(c, self.alloc.clone()) })
    }

    fn clear(&mut self) {
        let mut head: Option<ElementPtr<(K, V)>> = self.head;
        while let Some(ptr) = head {
            head = unsafe { ptr.as_ref().next };
            unsafe {
                core::ptr::drop_in_place(ptr.as_ptr());
                self.alloc.deallocate(ptr.cast(), Layout::new::<Self>())
            }
        }
        self.head = None;
    }

    fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl<T, A: Allocator + Clone> Drop for UnsafeSinglyLinkedList<T, A> {
    fn drop(&mut self) {
        let mut head = self.head;
        while let Some(s) = head {
            unsafe {
                head = s.as_ref().next;
                let layout = Layout::new::<SinglyLinkedListNode<T>>();
                drop_in_place(s.as_ptr());
                self.alloc.deallocate(s.cast(), layout);
            }
        }
    }
}
/*
impl<'a, K, V, A> BucketIter<'a, K, V, A> for UnsafeSinglyLinkedList<(K, V), A>
where
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone + 'a,
{
    type Iter = SLLBucketIter<'a, K, V, A>;
    fn iter(&'a self) -> Self::Iter {
        SLLBucketIter {
            head: self.head,
            marker: PhantomData,
        }
    }
}*/

impl<T, A: Allocator + Clone> Iterable for UnsafeSinglyLinkedList<T, A> {
    type Item = T;
    type Iter<'a> = SLLBucketIter<'a, T, A>
    where
        T: 'a, A: 'a
    ;
    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        SLLBucketIter {
            marker: PhantomData,
            head: self.head,
        }
    }
}
impl<T, A: Allocator + Clone> Drainable for UnsafeSinglyLinkedList<T, A> {
    type Item = T;
    type Drain<'a> = SLLBucketDrain<'a, T, A>
    where
        T: 'a, A: 'a
    ;
    fn drain<'a>(&'a mut self) -> Self::Drain<'a> {
        SLLBucketDrain {
            alloc: self.alloc.clone(),
            head: &mut self.head,
        }
    }
}
/*
impl<'a, K, V, A> BucketIter<'a, K, V, A> for UnsafeSinglyLinkedList<(K, V), A>
where
    K: Eq + Hash + 'a,
    V: 'a,
    A: Allocator + Clone + 'a,
{}


impl<'a, K, V, A> BucketDrain<'a, K, V, A> for UnsafeSinglyLinkedList<(K, V), A>
where
    Self: 'a,
    K: Eq + Hash,
    A: Allocator + Clone,
{}*/

pub struct SLLBucketIter<'a, T, A: Allocator + Clone> {
    head: Option<ElementPtr<T>>,
    marker: PhantomData<(&'a UnsafeSinglyLinkedList<T, A>, A)>,
}

impl<'a, T: 'a, A: Allocator + Clone> Iterator for SLLBucketIter<'a, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.head {
            unsafe {
                self.head = s.as_ref().next;
                Some(&s.as_ref().val)
            }
        } else {
            None
        }
    }
}

pub struct SLLBucketDrain<'a, T, A: Allocator + Clone> {
    head: &'a mut Option<ElementPtr<T>>,
    alloc: A,
}

impl<'a, T, A> Iterator for SLLBucketDrain<'a, T, A>
where
    A: Allocator + Clone,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.head {
            let a = *s;
            *self.head = unsafe { s.as_ref().next };
            Some(unsafe { SinglyLinkedListNode::into_tuple(a, self.alloc.clone()) })
        } else {
            None
        }
    }
}

pub struct SinglyLinkedListNode<T> {
    pub val: T,
    next: Option<ElementPtr<T>>,
}

/// WARNING: this struct does not drop any other list elements.
impl<T> SinglyLinkedListNode<T> {
    /// # Safety
    /// the caller is in charge of deallocation
    pub unsafe fn ptr_to_new<A: Allocator>(item: T, alloc: A) -> Result<ElementPtr<T>, AllocError> {
        let layout = Layout::new::<Self>();
        let ptr = alloc.allocate(layout)?.cast();
        let i = Self {
            val: item,
            next: None,
        };
        ptr::write(ptr.as_ptr(), i);
        Ok(ptr)
    }

    pub unsafe fn into_tuple<A: Allocator>(s: ElementPtr<T>, alloc: A) -> T {
        let v = ptr::read(s.as_ptr());
        alloc.deallocate(s.cast(), Layout::new::<SinglyLinkedListNode<T>>());
        v.val
    }
}

#[cfg(test)]
mod pub_api_tests {
    use super::*;
    use std::alloc::Global;

    #[test]
    fn insert_drop() {
        let mut a = UnsafeSinglyLinkedList::<(i32, i32), _>::new_in(Global);
        a.insert(1, 1).expect("alloc failed");
        a.insert(2, 2).expect("alloc failed");
    }

    #[test]
    fn remove_nothing() {
        let mut a = UnsafeSinglyLinkedList::<(i32, i32), _>::new_in(Global);
        assert!(a.remove(&1).is_none());
    }

    #[test]
    fn insert_remove() {
        let mut a = UnsafeSinglyLinkedList::<(i32, i32), _>::new_in(Global);
        a.insert(1, 1).expect("alloc failed");
        let i = a.remove(&1).unwrap();
        assert_eq!(i.1, 1);
    }

    /*
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
