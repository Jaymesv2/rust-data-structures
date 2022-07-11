use core::ops::RangeBounds;

pub trait Iterable {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item> + 'a
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait IterableMut {
    type Item;
    type IterMut<'a>: Iterator<Item = &'a mut Self::Item> + 'a
    where
        Self: 'a;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}

pub trait Drainable {
    type Item;
    type Drain<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain(&mut self) -> Self::Drain<'_>;
}

pub trait DrainableBy {
    type Item;
    type DrainBy<'a, F>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain_by<F>(&mut self) -> Self::DrainBy<'_, F>
    where
        F: FnMut(&Self::Item) -> bool;
}

pub trait DrainableRange {
    type Item;
    type DrainBy<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain_range<R>(&mut self, range: R) -> Self::DrainBy<'_>
    where
        R: RangeBounds<usize>;
}

pub trait RetainRange {
    type Item;
    type Retain<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn retain_range<R>(&mut self, range: R) -> Self::Retain<'_>
    where
        R: RangeBounds<usize>;
}

mod impls {
    use core::alloc::Allocator;

    use super::*;
    use crate::prelude::*;
    use alloc::collections::*;
    use alloc::slice;
    use alloc::vec::{self, Vec};

    // vec
    impl<T> Iterable for Vec<T> {
        type Item = T;
        type Iter<'a> = slice::Iter<'a, T> where T: 'a;
        fn iter(&self) -> Self::Iter<'_> {
            self.deref().iter()
        }
    }

    impl<T> IterableMut for Vec<T> {
        type Item = T;
        type IterMut<'a> = slice::IterMut<'a ,T> where T: 'a;
        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.deref_mut().iter_mut()
        }
    }

    impl<T, A: Allocator> Drainable for Vec<T, A> {
        type Item = T;
        type Drain<'a> = vec::Drain<'a, T, A> where T: 'a, A: 'a;
        fn drain(&mut self) -> Self::Drain<'_> {
            self.drain(..)
        }
    }

    impl<T> Iterable for VecDeque<T> {
        type Item = T;
        type Iter<'a> = vec_deque::Iter<'a, T> where T: 'a;
        fn iter(&self) -> Self::Iter<'_> {
            self.iter()
        }
    }

    impl<T> IterableMut for VecDeque<T> {
        type Item = T;
        type IterMut<'a> = vec_deque::IterMut<'a ,T> where T: 'a;
        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.iter_mut()
        }
    }

    impl<T, A: Allocator> Drainable for VecDeque<T, A> {
        type Item = T;
        type Drain<'a> = vec_deque::Drain<'a, T, A> where T: 'a, A: 'a;
        fn drain(&mut self) -> Self::Drain<'_> {
            self.drain(..)
        }
    }

    impl<T> Iterable for LinkedList<T> {
        type Item = T;
        type Iter<'a> = linked_list::Iter<'a, T> where T: 'a;
        fn iter(&self) -> Self::Iter<'_> {
            self.iter()
        }
    }

    impl<T> IterableMut for LinkedList<T> {
        type Item = T;
        type IterMut<'a> = linked_list::IterMut<'a ,T> where T: 'a;
        fn iter_mut(&mut self) -> Self::IterMut<'_> {
            self.iter_mut()
        }
    }

    impl<T> Iterable for BTreeSet<T> {
        type Item = T;
        type Iter<'a> = btree_set::Iter<'a, T> where T: 'a;
        fn iter(&self) -> Self::Iter<'_> {
            self.iter()
        }
    }
}
