use core::ops::RangeBounds;

pub trait Iterable {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item> + 'a
    where
        Self: 'a;
    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

pub trait IterableMut {
    type Item;
    type IterMut<'a>: Iterator<Item = &'a mut Self::Item> + 'a
    where
        Self: 'a;
    fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;
}

pub trait Drainable {
    type Item;
    type Drain<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain<'a>(&'a mut self) -> Self::Drain<'a>;
}

pub trait DrainableBy {
    type Item;
    type DrainBy<'a, F>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain_by<'a, F>(&'a mut self) -> Self::DrainBy<'a, F>
    where
        F: FnMut(&Self::Item) -> bool;
}

pub trait DrainableRange {
    type Item;
    type DrainBy<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn drain_range<'a, R>(&'a mut self, range: R) -> Self::DrainBy<'a>
    where
        R: RangeBounds<usize>;
}

pub trait RetainBy {
    type Item;
    type Retain<'a, F>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn retain_by<'a, F>(&'a mut self, f: F) -> Self::Retain<'a, F>
    where
        F: FnMut(&Self::Item) -> bool;
}

pub trait RetainRange {
    type Item;
    type Retain<'a>: Iterator<Item = Self::Item> + 'a
    where
        Self: 'a;
    fn retain_range<'a, R>(&'a mut self, range: R) -> Self::Retain<'a>
    where
        R: RangeBounds<usize>;
}

mod impls {
    use core::alloc::Allocator;

    use super::*;
    use crate::prelude::*;
    use alloc::vec::Vec;

    impl<T> Iterable for Vec<T> {
        type Item = T;
        type Iter<'a> = alloc::slice::Iter<'a, T> where T: 'a;
        fn iter<'a>(&'a self) -> Self::Iter<'a> {
            self.deref().iter()
        }
    }

    impl<T> IterableMut for Vec<T> {
        type Item = T;
        type IterMut<'a> = alloc::slice::IterMut<'a ,T> where T: 'a;
        fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a> {
            self.deref_mut().iter_mut()
        }
    }

    impl<T, A: Allocator> Drainable for Vec<T, A> {
        type Item = T;
        type Drain<'a> = alloc::vec::Drain<'a, T, A> where T: 'a, A: 'a;
        fn drain<'a>(&'a mut self) -> Self::Drain<'a> {
            self.drain(..)
        }
    }
}
