use alloc::boxed::Box;

use core::{
    alloc::{AllocError, Allocator},
    fmt::{self, Debug, Display, Formatter},
    hint::unreachable_unchecked,
    iter::Extend,
    ops::{Deref, DerefMut},
};

use traits::hash_table::seperate_chaining::bucket::*;

mod r#unsafe;

#[derive(Clone, PartialEq, Eq)]
pub struct SinglyLinkedList<T, A: Allocator + Clone = alloc::alloc::Global> {
    head: Option<Box<SinglyLinkedListNode<T, A>, A>>,
    alloc: A, // A should be a zst
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: T) {
        self.try_push(item).expect("failed_to_push")
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn last(&self) -> Option<&T> {
        self.iter().last()
    }

    pub fn contains<Q: PartialEq<T>>(&self, item: &Q) -> bool {
        self.iter().any(|s| item.eq(s))
    }
}

impl<T, A: Allocator + Clone> SinglyLinkedList<T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self { head: None, alloc }
    }

    /// Inserts an element at the beginning of the list
    pub fn try_push(&mut self, item: T) -> Result<(), AllocError> {
        let node = SinglyLinkedListNode {
            value: item,
            next: self.head.take(),
        };
        self.head = Some(Box::try_new_in(node, self.alloc.clone())?);
        Ok(())
    }

    /// inserts an element at the end of the list
    pub fn pop(&mut self) -> Option<T> {
        if let Some(SinglyLinkedListNode { value, next }) = self.head.take().map(Box::into_inner) {
            self.head = next;
            Some(value)
        } else {
            None
        }
    }

    /// Find by will only return Some if the inner mutable reference also returns None
    fn find_ref_mut_by<F: Fn(&T) -> bool>(
        &mut self,
        f: F,
    ) -> Option<&mut Option<Box<SinglyLinkedListNode<T, A>, A>>> {
        let mut current = &mut self.head;
        loop {
            match current {
                None => return None,
                Some(node) if (f)(&node.deref().value) => return Some(current),
                Some(node) => {
                    current = &mut node.next;
                }
            }
        }
        //self.iter_mut().find(f)
    }

    /// Find by will only return Some if the inner mutable reference also returns None
    fn find_ref_by<F: Fn(&T) -> bool>(
        &self,
        f: F,
    ) -> Option<&Option<Box<SinglyLinkedListNode<T, A>, A>>> {
        let mut current = &self.head;
        loop {
            match current {
                None => return None,
                Some(node) if (f)(&node.deref().value) => return Some(current),
                Some(node) => {
                    current = &node.next;
                }
            }
        }
        //self.iter_mut().find(f)
    }

    fn remove_by<F: Fn(&T) -> bool>(&mut self, f: F) -> Option<T> {
        let ptr = self.find_ref_mut_by(f)?;

        if let Some(mut node) = ptr.take() {
            *ptr = node.next.take();
            Some(node.value)
        } else {
            unsafe {
                // this is fine since find_by will only return Some if the inner option is also Some
                unreachable_unchecked()
            }
        }
    }

    pub fn get_mut_by<F: Fn(&T) -> bool>(&mut self, f: F) -> Option<&mut T> {
        if let Some(ref mut s) = self.find_ref_mut_by(f)? {
            let c = s.deref_mut();
            Some(&mut c.value)
        } else {
            None
        }
    }
    pub fn get_by<F: Fn(&T) -> bool>(&self, f: F) -> Option<&T> {
        if let Some(ref s) = self.find_ref_by(f)? {
            let c = s.deref();
            Some(&c.value)
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<'_, T, A> {
        Iter { node: &self.head }
    }

    pub fn drain(&mut self) -> Drain<'_, T, A> {
        let item = self.head.take();
        Drain {
            head_ref: &mut self.head,
            item_ref: item,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T, A> {
        IterMut {
            node: Some(&mut self.head),
        }
    }
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: None,
            alloc: alloc::alloc::Global,
        }
    }
}

impl<T: Debug, A: Allocator + Clone> Debug for SinglyLinkedList<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.iter().count();
        write!(f, "SinglyLinkedList {{ length: {len}, items: {{")?;
        let mut iter = self.iter();
        if let Some(elem) = iter.next() {
            write!(f, "{elem:?}")?
        }
        for elem in iter {
            write!(f, ", {elem:?}")?;
        }
        write!(f, "}} }}")
    }
}

impl<T> FromIterator<T> for SinglyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut lst = SinglyLinkedList::new();
        for i in iter.into_iter() {
            lst.push(i);
        }
        lst
    }
}

use iters::*;
mod iters {
    use super::*;
    use core::alloc::Allocator;

    impl<T, A: Allocator + Clone> IntoIterator for SinglyLinkedList<T, A> {
        type Item = T;
        type IntoIter = IntoIter<T, A>;
        fn into_iter(self) -> Self::IntoIter {
            IntoIter { head: self.head }
        }
    }

    pub struct IntoIter<T, A: Allocator + Clone> {
        pub(crate) head: Option<Box<SinglyLinkedListNode<T, A>, A>>,
    }

    impl<T, A: Allocator + Clone> Iterator for IntoIter<T, A> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(s) = self.head.take() {
                self.head = s.next;
                Some(s.value)
            } else {
                None
            }
        }
    }

    pub struct Iter<'a, T, A: Allocator + Clone> {
        pub(crate) node: &'a Option<Box<SinglyLinkedListNode<T, A>, A>>,
    }

    impl<'a, T, A: Allocator + Clone> Iterator for Iter<'a, T, A> {
        type Item = &'a T;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(s) = self.node {
                self.node = &s.deref().next;
                Some(&s.deref().value)
            } else {
                None
            }
        }
    }

    pub struct IterMut<'a, T, A: Allocator + Clone> {
        pub(crate) node: Option<&'a mut Option<Box<SinglyLinkedListNode<T, A>, A>>>,
    }

    impl<'a, T, A: Allocator + Clone> Iterator for IterMut<'a, T, A> {
        type Item = &'a mut T;
        fn next(&mut self) -> Option<Self::Item> {
            match self.node.take() {
                Some(Some(s)) => {
                    let SinglyLinkedListNode {
                        ref mut value,
                        ref mut next,
                    } = s.as_mut();
                    self.node = Some(next);
                    Some(value)
                }
                Some(None) => None,
                None => None,
            }
        }
    }

    pub struct Drain<'a, T, A: Allocator + Clone> {
        pub(crate) head_ref: &'a mut Option<Box<SinglyLinkedListNode<T, A>, A>>,
        pub(crate) item_ref: Option<Box<SinglyLinkedListNode<T, A>, A>>,
    }

    impl<'a, T, A: Allocator + Clone> Iterator for Drain<'a, T, A> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(s) = self.item_ref.take() {
                self.item_ref = s.next;
                Some(s.value)
            } else {
                None
            }
        }
    }

    impl<'a, T, A: Allocator + Clone> Drop for Drain<'a, T, A> {
        fn drop(&mut self) {
            *self.head_ref = self.item_ref.take();
        }
    }
}

impl<T, A: Allocator + Clone> Extend<T> for SinglyLinkedList<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for i in iter.into_iter() {
            self.try_push(i).expect("failed to allocate");
        }
    }
}

impl<K: Eq, V, A: Allocator + Clone> Bucket<K, V, A> for SinglyLinkedList<(K, V), A> {
    fn new_in(alloc: A) -> Self {
        Self::new_in(alloc)
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.get_by(|(k, _)| key == k).map(|(_, v)| v)
    }
    fn insert(&mut self, key: K, mut value: V) -> Result<Option<(K, V)>, AllocError> {
        // if the value exists then swap it, else push it to the front
        if let Some((_, v)) = self.iter_mut().find(|(k, _)| k == &key) {
            core::mem::swap(v, &mut value);
            Ok(Some((key, value)))
        } else {
            self.try_push((key, value))?;
            Ok(None)
        }
    }
    unsafe fn insert_unchecked(&mut self, key: K, value: V) -> Result<(), AllocError> {
        self.try_push((key,value))
    }
    fn clear(&mut self) {
        self.head = None;
    }
    fn is_empty(&self) -> bool {
        self.iter().count() == 0
    }
    fn remove(&mut self, key: &K) -> Option<(K, V)> {
        self.remove_by(|(k, _)| k == key)
    }
}

/*
impl<'a,K,V,A> BucketIter<'a, K, V, A> for SinglyLinkedList<(K,V), A>
where
    K: Eq + Hash + 'a,
    V: 'a,
    A: 'a,
    A: Allocator + Clone,
{
    type Iter = iters::Iter<'a, (K,V),A>;
    fn iter(&'a self) -> Self::Iter {
        self.iter().map(|(k,v)| (k,v))
    }
}

pub trait BucketIterMut<'a, K, V, A>: Bucket<K, V, A>
where
    V: 'a,
    K: Eq + Hash + 'a,
    A: Allocator + Clone + 'a,
{
    type IterMut: Iterator<Item = (&'a mut K, &'a mut V)>;
    fn iter_mut(&mut self) -> Self::IterMut;
}*/

impl<'a, K, V, A> BucketDrain<'a, K, V, A> for SinglyLinkedList<(K, V), A>
where
    Self: 'a,
    K: Eq,
    A: Allocator + Clone,
{
    type DrainIter = iters::Drain<'a, (K, V), A>;
    fn drain(&'a mut self) -> Self::DrainIter {
        self.drain()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SinglyLinkedListNode<T, A: Allocator> {
    value: T,
    next: Option<Box<SinglyLinkedListNode<T, A>, A>>,
}

impl<T: Debug, A: Allocator + Clone> Debug for SinglyLinkedListNode<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display, A: Allocator + Clone> Display for SinglyLinkedListNode<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
#[cfg(test)]
mod tests {
    use super::SinglyLinkedList;
    use alloc::alloc::Global;
    use test::Bencher;

    #[bench]
    fn insert_bench(b: &mut Bencher) {
        let lst: SinglyLinkedList<i32, Global> =
            (0..1000).collect::<SinglyLinkedList<i32, Global>>();

        b.iter(|| {
            let _c = lst.clone();
        });
    }

    #[test]
    fn new_and_insert() {
        let mut a = SinglyLinkedList::new();
        for i in 0..10 {
            a.push(i)
        }
        let mut b = vec![];
        for i in a.iter() {
            b.push(i.clone());
        }
        for i in 0..10 {
            assert!(b.contains(&i));
        }
        println!("list: {a:?}")
    }

    #[test]
    fn insert_remove() {
        let mut lst: SinglyLinkedList<i32, Global> = (0..10).collect();
        assert!(lst.remove_by(|c| c == &5).is_some());
        let b = (0..10).rev().filter(|c| c != &5).collect::<Vec<_>>();
        assert_eq!(b, lst.iter().copied().collect::<Vec<_>>());
    }

    #[test]
    fn iter_mut_test() {
        let mut lst: SinglyLinkedList<i32, Global> = (0..10).collect();
        for i in lst.iter_mut() {
            *i += 1;
        }
        assert_eq!(
            lst.iter().copied().collect::<Vec<_>>(),
            (1..11).rev().collect::<Vec<_>>()
        );
    }

    #[test]
    fn drain_test() {
        let mut lst: SinglyLinkedList<i32, Global> = (0..100).collect();
        assert_eq!(lst.len(), 100);
        let drain = lst.drain();
        let b = drain.take(50).collect::<Vec<_>>();
        assert_eq!(b.len(), 50);
        assert_eq!(lst.len(), 50);
        assert_eq!(
            (0..50).rev().collect::<Vec<_>>(),
            lst.into_iter().collect::<Vec<_>>()
        );
        assert_eq!((50..100).rev().collect::<Vec<_>>(), b);
    }
}
