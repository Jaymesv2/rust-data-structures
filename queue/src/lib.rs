#![no_std]
#![feature(generic_associated_types, allocator_api, const_option, let_chains)]
#![warn(unsafe_code)]

extern crate alloc;
use alloc::{alloc::Global, string::String, vec::Vec};
use core::{
    alloc::{AllocError, Allocator, Layout},
    fmt::{self, Debug, Formatter},
    iter::{self, Extend, FromIterator},
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::{self, drop_in_place, NonNull},
};
use iters::*;

/// small vecdeque implementation
///
/// use `push_back` to add to the queue and `pop_front` to remove
pub struct ArrayQueue<T, A: Allocator = Global> {
    len: usize,
    capacity: usize,
    start: usize,
    ptr: NonNull<T>,
    alloc: A,
}

const DEFAULT_SIZE: NonZeroUsize = NonZeroUsize::new(16).unwrap();

impl<T> ArrayQueue<T, Global> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, Global).expect("failed to allocate")
    }
}
impl<T, A: Allocator> ArrayQueue<T, A> {
    pub fn new_in(alloc: A) -> Self {
        ArrayQueue {
            len: 0,
            capacity: 0,
            start: 0,
            ptr: NonNull::dangling(),
            alloc,
        }
    }
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, AllocError> {
        let mut queue = Self::new_in(alloc);
        if let Some(s) = NonZeroUsize::new(capacity) {
            queue.grow_to(s)?;
        }
        Ok(queue)
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        /*
        // if the current capacity is zero then do nothing
        if self.capacity == 0 || self.len == 0 {
            return (&[], &[]);
        }*/
        // i'm still not sure if this is right. it passes all the tests but i feel like there are edge cases where it wont work
        let len1 = self.capacity - self.start;
        unsafe {
            (
                core::slice::from_raw_parts(self.ptr.as_ptr().add(self.start), len1),
                core::slice::from_raw_parts(self.ptr.as_ptr(), self.len.saturating_sub(len1)),
            )
        }
    }

    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        let len1 = self.capacity - self.start;
        unsafe {
            (
                core::slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.start), len1),
                core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len.saturating_sub(len1)),
            )
        }
    }

    /// # SAFETY
    /// First, `new_ptr` have a layout of `Layout::array::<T>(capacity)` where `capacity` must be greater than or equal to the capacity than the current capacity a
    /// Second, `new_ptr` cannot be equal to `self.ptr`
    unsafe fn copy_elements_to(&self, new_ptr: NonNull<T>) {
        debug_assert!(new_ptr != self.ptr);
        // if the current capacity is zero then do nothing
        /*if self.capacity == 0 || self.len == 0 {
            return;
        }*/

        let (fst, snd) = self.as_slices();
        ptr::copy_nonoverlapping(fst.as_ptr(), new_ptr.as_ptr(), fst.len());
        if !snd.is_empty() {
            ptr::copy_nonoverlapping(snd.as_ptr(), new_ptr.as_ptr().add(fst.len()), snd.len());
        }
    }

    pub fn shrink_to(&mut self, size: usize) -> Result<(), AllocError> {
        let _ = self.make_contiguous();
        let size = core::cmp::max(size, self.len);
        unsafe {
            self.alloc.shrink(
                self.ptr.cast(),
                Layout::array::<T>(self.capacity).expect(""),
                Layout::array::<T>(size).expect(""),
            )?;
        }
        Ok(())
    }

    /// new_capacity must be greater than or equal to the current capacity
    fn grow_to(&mut self, new_capacity: NonZeroUsize) -> Result<(), AllocError> {
        let layout = Layout::array::<T>(new_capacity.get()).expect("failed to create layout");
        let new_ptr: NonNull<T> = self.alloc.allocate(layout)?.cast();

        debug_assert!(new_capacity.get() >= self.capacity);
        unsafe {
            self.copy_elements_to(new_ptr);
            self.alloc.deallocate(
                self.ptr.cast(),
                Layout::array::<T>(self.capacity).expect("failed to get current layout"),
            );
        }
        self.start = 0;
        self.capacity = new_capacity.get();
        self.ptr = new_ptr;
        Ok(())
    }
    /// add to the back of the queue
    pub fn push_back(&mut self, item: T) -> Result<(), AllocError> {
        if self.len + 1 > self.capacity {
            self.grow()?;
        }
        unsafe {ptr::write(self.ptr_to_mut(self.len), item)}
        /*unsafe {
            let push_ptr = self
                .ptr
                .as_ptr()
                .add((self.start + self.len) % self.capacity);
            ptr::write(push_ptr, item);
        }*/
        self.len += 1;
        Ok(())
    }

    pub fn push_front(&mut self, item: T) -> Result<(), AllocError> {
        if self.len + 1 > self.capacity {
            self.grow()?;
        }
        self.start = self.start.checked_sub(1).unwrap_or(self.capacity - 1);
        unsafe {ptr::write(self.ptr_to_mut(0), item)}
        /*unsafe {
            let push_ptr = self.ptr.as_ptr().add(self.start);
            ptr::write(push_ptr, item);
        };*/
        self.len += 1;
        Ok(())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        (self.len != 0).then(|| {
            self.len -= 1;
            unsafe {ptr::read::<T>(self.ptr_to(self.len))}
            /*unsafe {
                ptr::read::<T>(
                    self.ptr
                        .as_ptr()
                        .add((self.start + self.len) % self.capacity),
                )
            }*/
        })
    }

    // remove from the front
    pub fn pop_front(&mut self) -> Option<T> {
        (self.len != 0).then(|| {
            let item = unsafe { ptr::read(self.ptr_to(0)) };
            self.start = (self.start + 1) % (self.capacity);
            self.len -= 1;
            item
        })
    }
    pub fn clear(&mut self) {
        self.iter_mut().for_each(|ptr| unsafe {
            drop_in_place(ptr);
        });
        self.len = 0;
        self.start = 0;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        (index <= self.len).then(|| unsafe { self.get_unchecked(index) })
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        (index <= self.len).then(|| unsafe { self.get_unchecked_mut(index) })
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        &*self.ptr_to(index)
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut *self.ptr_to_mut(index)
    }

    // gets a pointer to the nth element 
    unsafe fn ptr_to(&self, index: usize) -> *const T {
        self.ptr.as_ptr().add((self.start + index) % self.capacity)
    }

    unsafe fn ptr_to_mut(&mut self, index: usize) -> *mut T {
        self.ptr.as_ptr().add((self.start + index) % self.capacity)
    }

    pub fn reserve(&mut self, count: usize) -> Result<(), AllocError> {
        if count != 0 && self.len+count > self.capacity && let Some(new_cap) = NonZeroUsize::new(count+self.len)  {
            self.grow_to(new_cap)?;
        }
        Ok(())
    }

    pub fn make_contiguous(&mut self) -> &mut [T] {
        todo!()
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), AllocError> {
        self.shrink_to(self.len)
    }

    pub fn truncate(&mut self, size: usize) {
        if size < self.len {
            let num_to_drop = size - self.len;
            self.drain().rev().take(num_to_drop).for_each(|f| drop(f));
        }
    }

    fn grow(&mut self) -> Result<(), AllocError> {
        let new_capacity = NonZeroUsize::new(self.capacity).unwrap_or(DEFAULT_SIZE);
        self.grow_to(new_capacity)
    }

    /// i have no clue if this is correct.
    fn layout_iter(&self) -> impl Iterator<Item = bool> + '_ {
        let mut pos = 0;
        let cap = self.capacity;
        let start = self.start;
        let len = self.len;
        iter::from_fn(move || {
            (pos != cap).then(|| {
                // one segment  | -------xxxxxxxxxxxxx----------|
                let a = if start + len <= cap {
                    start <= pos && pos < start + len
                // two segments |xxxxxx--------------------xxxxx|
                } else {
                    (0 < pos && pos < (start + len) % cap) || (start <= pos && pos <= cap)
                };
                pos += 1;
                a
            })
        })
    }

    pub fn insert(&mut self, index: usize, value: T) {
        todo!()
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        todo!()
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    pub fn contains<Q: PartialEq<T>>(&self, item: &Q) -> bool {
        self.iter().any(|i| item == i)
    }

    pub fn allocator(&self) -> &A {
        &self.alloc
    }

    pub fn back(&self) -> Option<&T> {
        self.get(self.len())
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, T, A> {
        Iter::from(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T, A> {
        IterMut::from(self)
    }

    pub fn drain(&mut self) -> Drain<'_, T, A> {
        Drain::from(self)
    }
}

impl<T, A: Allocator> Drop for ArrayQueue<T, A> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            // this should never happen since the layout needs to be created in `grow` before it can be recreated here.
            unsafe {
                self.alloc.deallocate(
                    self.ptr.cast(),
                    Layout::array::<T>(self.capacity).unwrap_unchecked(),
                )
            };
        }
    }
}

impl<T, A: Allocator> Index<usize> for ArrayQueue<T, A> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T, A: Allocator> IndexMut<usize> for ArrayQueue<T, A> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<T: Debug> Debug for ArrayQueue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let layout = self
            .layout_iter()
            .map(|c| if c { 'x' } else { '-' })
            .collect::<String>();
        //let layout = self.layout_iter().enumerate().map(|(i, c)| if i == self. c {'x'} else {'-'}).collect::<String>();

        write!(f, "ArrayQueue {{ len: {}, capacity: {}, first_item_index: {}, layout: |{layout}|, items: {{", self.len, self.capacity, self.start)?;
        let mut iter = self.iter();
        if let Some(elem) = iter.next() {
            write!(f, "{elem:?}")?;
        }
        for elem in iter {
            write!(f, ", {elem:?}")?;
        }
        write!(f, "}} }}")
    }
}

impl<T: Clone, A: Allocator + Clone> Clone for ArrayQueue<T, A> {
    fn clone(&self) -> Self {
        // i could probably make this a bit better but oh well
        let mut v = Self::new_in(self.alloc.clone());
        v.extend(self.iter().cloned());
        v
    }
}

impl<T: PartialEq + Eq, A: Allocator> Eq for ArrayQueue<T, A> {}

impl<T: PartialEq, A: Allocator> PartialEq for ArrayQueue<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.len != other.len && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<T> Default for ArrayQueue<T, Global> {
    fn default() -> Self {
        Self::new_in(Global)
    }
}
impl<T, A: Allocator + Clone> IntoIterator for ArrayQueue<T, A> {
    type IntoIter = IntoIter<T, A>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from(self)
    }
}

impl<T, A: Allocator + Clone> Extend<T> for ArrayQueue<T, A> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter.into_iter() {
            self.push_back(elem)
                .expect("failed to allocate while extending ArrayQueue");
        }
    }
}

impl<T> FromIterator<T> for ArrayQueue<T, Global> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let mut queue = Self::with_capacity_in(upper.unwrap_or(lower), Global)
            .expect("failed to allocate while creating ArrayQueue from an iterator");
        queue.extend(iter);
        queue
    }
}

impl<T, A: Allocator + Clone> From<Vec<T, A>> for ArrayQueue<T, A> {
    // TODO: look at vec layout
    fn from(v: Vec<T, A>) -> Self {
        let (ptr, len, capacity, alloc) = v.into_raw_parts_with_alloc();
        Self {
            len,
            capacity,
            start: 0,
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            alloc,
        }
    }
}

mod iters {
    use super::ArrayQueue;
    use alloc::alloc::Global;
    use core::{alloc::Allocator, iter::FusedIterator, ptr, mem::transmute};

    pub struct Iter<'a, T, A: Allocator = Global> {
        inner: &'a ArrayQueue<T, A>,
        current_ind: usize,
        remaining: usize,
    }

    impl<'a, T, A: Allocator> From<&'a ArrayQueue<T, A>> for Iter<'a, T, A> {
        fn from(inner: &'a ArrayQueue<T, A>) -> Self {
            Self {
                inner,
                current_ind: 0,
                remaining: inner.len(),
            }
        }
    }

    impl<'a, T: 'a, A: Allocator> Iterator for Iter<'a, T, A> {
        type Item = &'a T;
        fn next(&mut self) -> Option<Self::Item> {
            (self.remaining != 0).then(|| {
                self.remaining -= 1;
                self.current_ind += 1;
                unsafe { self.inner.get_unchecked(self.current_ind-1) }
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining, Some(self.remaining))
        }
    }

    impl<'a, T: 'a, A: Allocator> DoubleEndedIterator for Iter<'a, T, A> {
        fn next_back(&mut self) -> Option<Self::Item> {
            (self.remaining != 0).then(|| {
                self.remaining -= 1;
                let ind = self.current_ind + self.remaining;
                unsafe { self.inner.get_unchecked(ind) }
            })
        }
    }

    impl<'a, T: 'a, A: Allocator> ExactSizeIterator for Iter<'a, T, A> {
        fn len(&self) -> usize {
            self.remaining
        }
    }

    impl<'a, T: 'a, A: Allocator> FusedIterator for Iter<'a, T, A> {}

    pub struct IterMut<'a, T, A: Allocator = Global> {
        inner: &'a mut ArrayQueue<T, A>,
        current_ind: usize,
        remaining: usize,
    }

    impl<'a, T, A: Allocator> From<&'a mut ArrayQueue<T, A>> for IterMut<'a, T, A> {
        fn from(inner: &'a mut ArrayQueue<T, A>) -> Self {
            Self {
                current_ind: 0,
                remaining: inner.len(),
                inner,
            }
        }
    }

    impl<'a, T: 'a, A: Allocator> Iterator for IterMut<'a, T, A> {
        type Item = &'a mut T;
        fn next(&mut self) -> Option<Self::Item> {
            (self.remaining != 0).then(|| {
                self.remaining -= 1;
                self.current_ind += 1;
                // the transmute makes the lifetimes happy
                unsafe { transmute(self.inner.get_unchecked_mut(self.current_ind-1)) }
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining, Some(self.remaining))
        }
    }

    impl<'a, T: 'a, A: Allocator> DoubleEndedIterator for IterMut<'a, T, A> {
        fn next_back(&mut self) -> Option<Self::Item> {
            (self.remaining != 0).then(|| {
                self.remaining -= 1;
                let ind = self.current_ind + self.remaining;
                unsafe { transmute(self.inner.get_unchecked_mut(ind)) }
            })
        }
    }

    impl<'a, T: 'a, A: Allocator> ExactSizeIterator for IterMut<'a, T, A> {
        fn len(&self) -> usize {
            self.remaining
        }
    }

    impl<'a, T: 'a, A: Allocator> FusedIterator for IterMut<'a, T, A> {}

    pub struct Drain<'a, T, A: Allocator = Global> {
        inner: &'a mut ArrayQueue<T, A>,
        len: usize,
        start: usize,
    }

    impl<'a, T, A: Allocator> From<&'a mut ArrayQueue<T, A>> for Drain<'a, T, A> {
        fn from(inner: &'a mut ArrayQueue<T, A>) -> Self {
            let len = inner.len;
            inner.len = 0;
            Self {
                len,
                start: inner.start,
                inner,
            }
        }
    }

    impl<'a, T: 'a, A: Allocator> Iterator for Drain<'a, T, A> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            (self.len != 0).then(|| {
                let ptr = unsafe { self.inner.ptr.as_ptr().add(self.start) };
                self.start = (self.start + 1) % (self.inner.capacity);
                self.len -= 1;
                unsafe { ptr::read(ptr) }
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl<'a, T: 'a, A: Allocator> DoubleEndedIterator for Drain<'a, T, A> {
        fn next_back(&mut self) -> Option<Self::Item> {
            (self.len != 0).then(|| {
                self.len -= 1;
                unsafe {
                    let ind = (self.start + self.len) % self.inner.capacity;
                    ptr::read(self.inner.ptr.as_ptr().add(ind))
                }
            })
        }
    }

    impl<'a, T: 'a, A: Allocator> ExactSizeIterator for Drain<'a, T, A> {
        fn len(&self) -> usize {
            self.len
        }
    }

    impl<'a, T: 'a, A: Allocator> FusedIterator for Drain<'a, T, A> {}

    impl<'a, T, A: Allocator> Drop for Drain<'a, T, A> {
        fn drop(&mut self) {
            self.inner.start = self.start;
            self.inner.len = self.len;
        }
    }

    pub struct IntoIter<T, A: Allocator = Global> {
        inner: ArrayQueue<T, A>,
    }

    impl<T, A: Allocator> From<ArrayQueue<T, A>> for IntoIter<T, A> {
        fn from(inner: ArrayQueue<T, A>) -> Self {
            Self { inner }
        }
    }

    impl<'a, T: 'a, A: Allocator> Iterator for IntoIter<T, A> {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            self.inner.pop_front()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.inner.len, Some(self.inner.len))
        }
    }

    impl<'a, T: 'a, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.inner.pop_back()
        }
    }

    impl<'a, T: 'a, A: Allocator> ExactSizeIterator for IntoIter<T, A> {
        fn len(&self) -> usize {
            self.inner.len
        }
    }

    impl<'a, T: 'a, A: Allocator> FusedIterator for IntoIter<T, A> {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_new() {
        let _queue: ArrayQueue<i32> =
            ArrayQueue::with_capacity_in(32, Global).expect("alloc failed");
    }

    #[test]
    fn queue_grow() {
        let mut queue: ArrayQueue<i32> =
            ArrayQueue::with_capacity_in(12, Global).expect("alloc failed");
        for i in 0..12 {
            queue.push_back(i).expect("failed to alloc");
        }
        assert_eq!(queue.pop_front(), Some(0));
        assert_eq!(queue.pop_front(), Some(1));
        let len = queue.len();
        queue
            .grow_to(NonZeroUsize::new(14).unwrap())
            .expect("failed to alloc");
        assert_eq!(queue.len(), len);
    }

    #[test]
    fn queue_push_back_10() {
        let mut queue: ArrayQueue<i32> =
            ArrayQueue::with_capacity_in(32, Global).expect("alloc failed");
        for i in 0..10 {
            queue.push_back(i).expect("failed to alloc");
        }
        for i in 0..10 {
            assert_eq!(queue.pop_front().unwrap(), i);
        }
    }

    #[test]
    fn queue_push_front_10() {
        let mut queue: ArrayQueue<i32> =
            ArrayQueue::with_capacity_in(32, Global).expect("alloc failed");
        for i in 0..10 {
            queue.push_front(i).expect("failed to alloc");
        }
        for i in 0..10 {
            assert_eq!(queue.pop_back().unwrap(), i);
        }
    }

    #[test]
    fn queue_enqueue_dequeue_loop() {
        const STEP: usize = 5;
        let mut queue: ArrayQueue<usize> =
            ArrayQueue::with_capacity_in(32, Global).expect("alloc failed");
        let range = 0..1000;
        let mut items = range.clone().rev().collect::<Vec<usize>>();
        for i in range.step_by(STEP) {
            for i in i..i + STEP {
                queue.push_back(i).expect("failed to alloc");
                assert_eq!(queue.pop_front(), items.pop());
            }
        }
    }

    #[test]
    fn drain() {
        let mut queue: ArrayQueue<usize> = (0..100).collect();
        let fst_fifty: Vec<usize> = queue.drain().take(50).collect();
        assert_eq!(fst_fifty, (0..50).collect::<Vec<_>>());
        let snd_fifty: Vec<usize> = queue.iter().copied().collect();
        assert_eq!(snd_fifty, (50..100).collect::<Vec<_>>());
    }

    #[test]
    fn drain_rev() {
        let mut queue: ArrayQueue<usize> = (0..100).collect();
        let fst_fifty: Vec<usize> = queue.drain().rev().take(50).collect();
        assert_eq!(fst_fifty, (50..100).rev().collect::<Vec<_>>());
        let snd_fifty: Vec<usize> = queue.iter().copied().collect();
        assert_eq!(snd_fifty, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn iter() {
        let queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).collect::<Vec<_>>();
        let b = queue.iter().copied().collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn iter_rev() {
        let queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).rev().collect::<Vec<_>>();
        let b = queue.iter().rev().copied().collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn into_iter() {
        let queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).collect::<Vec<_>>();
        let b = queue.into_iter().collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn into_iter_rev() {
        let queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).rev().collect::<Vec<_>>();
        let b = queue.into_iter().rev().collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn iter_mut() {
        let mut queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).collect::<Vec<_>>();
        let b = queue.iter_mut().map(|c| *c).collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn iter_mut_rev() {
        let mut queue: ArrayQueue<usize> = (0..1000).collect();
        let a = (0..1000).rev().collect::<Vec<_>>();
        let b = queue.iter_mut().map(|c| *c).rev().collect::<Vec<_>>();
        assert_eq!(a, b);
    }

    #[test]
    fn slices() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(100);
        queue.extend(0..100);
        let (fst, snd) = queue.as_slices();
        let v = (0..100).collect::<Vec<_>>();
        assert!(snd.is_empty());
        assert_eq!(fst, &v)
    }

    #[test]
    fn slices2() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(25);
        queue.extend(0..25);
        queue.drain().take(10).for_each(drop);
        queue.extend(25..35);
        let (fst, snd) = queue.as_slices();
        let v1 = (10..25).collect::<Vec<_>>();
        let v2 = (25..35).collect::<Vec<_>>();
        assert_eq!(fst, &v1);
        assert_eq!(snd, &v2);
    }
    #[test]
    fn slices3() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(25);
        queue.extend(0..25);
        queue.drain().take(15).for_each(drop);
        queue.extend(25..30);
        let (fst, snd) = queue.as_slices();
        let v1 = (15..25).collect::<Vec<_>>();
        let v2 = (25..30).collect::<Vec<_>>();
        assert_eq!(fst, &v1);
        assert_eq!(snd, &v2);
    }
}
