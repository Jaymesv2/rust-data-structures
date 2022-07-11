use crate::prelude::*;
use alloc::{alloc::Global, string::String, vec::Vec};
use core::{
    alloc::{AllocError, Allocator, Layout},
    fmt::{self, Debug, Formatter},
    iter::{self, Extend, FromIterator},
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::{self, drop_in_place, NonNull},
};
mod iters;
use iters::*;

/// A double ended queue using a growable ring buffer.
///
/// Inspired by the stdlib implementaiton.
///
/// use `push_back` to add to the queue and `pop_front` to remove
pub struct ArrayQueue<T, A = Global>
where
    A: Allocator,
{
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
impl<T, A> ArrayQueue<T, A>
where
    A: Allocator,
{
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
        let len1 = core::cmp::min(self.len, self.capacity - self.start);
        unsafe {
            (
                core::slice::from_raw_parts(self.ptr.as_ptr().add(self.start), len1),
                core::slice::from_raw_parts(self.ptr.as_ptr(), self.len.saturating_sub(len1)),
            )
        }
    }

    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        let len1 = core::cmp::min(self.len, self.capacity - self.start);
        unsafe {
            (
                core::slice::from_raw_parts_mut(self.ptr.as_ptr().add(self.start), len1),
                core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len.saturating_sub(len1)),
            )
        }
    }

    /// Copies the elements from self to `new_ptr`
    ///
    /// # Safety
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

    /// Add to the back of the queue
    pub fn push_back(&mut self, item: T) -> Result<(), AllocError> {
        if self.len + 1 > self.capacity {
            self.grow()?;
        }
        unsafe { ptr::write(self.ptr_to_mut(self.len), item) }
        self.len += 1;
        Ok(())
    }
    /// Adds and element to the front of the queue
    pub fn push_front(&mut self, item: T) -> Result<(), AllocError> {
        if self.len + 1 > self.capacity {
            self.grow()?;
        }
        self.start = self.start.checked_sub(1).unwrap_or(self.capacity - 1);
        unsafe { ptr::write(self.ptr_to_mut(0), item) }
        self.len += 1;
        Ok(())
    }
    /// removes the element at the back of the queue.
    /// # Examples
    /// ```
    /// use queue::ArrayQueue;
    /// let mut queue: ArrayQueue<usize> = (0..10).collect();
    /// for i in (0..10).rev() {
    ///     assert_eq!(queue.pop_back(), Some(i));
    /// }
    /// assert!(queue.is_empty());
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        (self.len != 0).then(|| {
            self.len -= 1;
            unsafe { ptr::read::<T>(self.ptr_to(self.len)) }
        })
    }

    /// remove from the front of the queue.
    pub fn pop_front(&mut self) -> Option<T> {
        (self.len != 0).then(|| {
            let item = unsafe { ptr::read(self.ptr_to(0)) };
            self.start = (self.start + 1) % (self.capacity);
            self.len -= 1;
            item
        })
    }
    /// Drops all elements in the queue leaving it empty.
    /// # Examples
    /// ```
    /// use queue::ArrayQueue;
    /// let mut queue: ArrayQueue<usize> = (0..10).collect();
    /// assert_eq!(queue.len(), 10);
    /// queue.clear();
    /// assert!(queue.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.iter_mut().for_each(|ptr| unsafe {
            drop_in_place(ptr);
        });
        self.len = 0;
        self.start = 0;
    }

    /// Gets a reference to the element at `index`.
    /// # Examples
    /// ```
    /// use queue::ArrayQueue;
    /// let mut queue: ArrayQueue<usize> = (0..10).collect();
    /// assert_eq!(queue.get(5), Some(&5));
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        (index < self.len).then(|| unsafe { self.get_unchecked(index) })
    }

    /// Gets a mutable reference to the element at `index`
    /// # Examples
    /// ```
    /// use queue::ArrayQueue;
    /// let mut queue: ArrayQueue<usize> = (0..10).collect();
    /// if let Some(s) = queue.get_mut(5) {
    ///     *s += 10;
    /// }
    /// assert_eq!(queue.get(5), Some(&15));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        (index < self.len).then(|| unsafe { self.get_unchecked_mut(index) })
    }

    /// Gets a reference to the element at `index` without checking bounds.
    ///
    /// # Safety
    /// `index` must be less than `self.index`
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        &*self.ptr_to(index)
    }

    /// Gets a mutable reference to the element at `index` without checking bounds.
    ///
    /// # Safety
    /// `index` must be less than `self.index`
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut *self.ptr_to_mut(index)
    }

    /// gets a pointer to the element at the specified index.
    ///
    /// # Safety
    /// `index` must be less than `self.index`
    unsafe fn ptr_to(&self, index: usize) -> *const T {
        self.ptr.as_ptr().add((self.start + index) % self.capacity)
    }

    /// gets a mutable pointer to the element at the specified index.
    ///
    /// # Safety
    /// `index` must be less than `self.index`
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
        // 2 slices
        if self.start + self.len > self.capacity {
            unsafe {
                // this is terrible
                self.grow_to(NonZeroUsize::new_unchecked(self.capacity))
                    .unwrap();
            }
            /*

            let dist = self.start;
            let count = (self.start + self.len) % self.capacity;
            for i in 0..count {
                unsafe {
                    ptr::swap(self.ptr.as_ptr().add(i),self.ptr.as_ptr().add(dist+i));
                }
            }
            unsafe {
                let a = self.len - count;
                ptr::copy(self.ptr.as_ptr().add(dist+count), self.ptr.as_ptr().add(), a);
            }
            todo!() */
            /*let end = (self.start+self.len())%self.capacity;
            let snd_len = self.len - end;

            unsafe {ptr::copy(self.ptr.as_ptr().add(self.start), self.ptr.as_ptr().add(end), snd_len)};

            for i in 0..snd_len {
                unsafe{ptr::swap(self.ptr.as_ptr().add(i), self.ptr.as_ptr().add(i+snd_len))}
            }*/
        } else {
            unsafe {
                ptr::copy(
                    self.ptr.as_ptr().add(self.start),
                    self.ptr.as_ptr(),
                    self.len,
                )
            };
        }
        self.start = 0;
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn insert(&mut self, index: usize, value: T) -> Result<(), AllocError> {
        assert!(index <= self.len + 1, "out of bounds");
        if self.len + 1 >= self.capacity {
            self.grow()?;
        }
        let real_index = (self.start + index) % self.capacity;
        let ptr = self.ptr.as_ptr();

        // this is only here so that the debugger can see the slice.
        //#[cfg(test)]
        //let a = unsafe { core::slice::from_raw_parts(ptr, self.capacity) };
        /*
            if there will be 2 segments then
                if the real index is in the second segment (the one at the "front" of the ring) then
                    move the elements in front forward.
                else if the real index is in the first segment("back of the ring") then
                    move the front elements forward, copy the back elem to front
                    if real index is not at the end of the buffer then
                        copy remaining elements after the real index forward
                    end
                end
            else
                copy elements after real index forward.

            write to real index.
        */

        // 2 segments
        // its one big unsafe block since its just copies and ifs
        unsafe {
            if self.start + real_index >= self.capacity {
                // in the second segment (front of buf)
                if real_index > (self.start + self.len) % self.capacity {
                    // move the elems at the front forward
                    ptr::copy(
                        ptr,
                        ptr.add(1),
                        //self.len - index,
                        (self.start + self.len) % self.capacity, // # elems at front
                    );
                    // this might be non overlapping
                    // move back element to front
                    ptr::copy_nonoverlapping(ptr.add(self.capacity - 1), ptr, 1);
                    if real_index + 1 != self.capacity {
                        ptr::copy(
                            ptr.add(real_index),
                            ptr.add(real_index + 1),
                            self.capacity.saturating_sub(real_index).saturating_sub(1),
                        );
                    }
                    // in the first segment (back of buf)
                }
            // one segment
            } else {
                ptr::copy(
                    ptr.add(real_index),
                    ptr.add(real_index + 1),
                    self.len - index,
                );
            }
            ptr::write(ptr.add(real_index), value);
        }

        self.len += 1;

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index > self.len {
            return None;
        }
        let elem = unsafe { ptr::read(self.ptr_to(index)) };
        // move all the other elements
        // if there are 2 segments
        if self.start + self.len > self.capacity {
            todo!()
        } else {
            let num_to_move = self.len - index - 1;

            unsafe {
                ptr::copy(self.ptr_to(index + 1), self.ptr_to_mut(index), num_to_move);
            }
        }
        self.len -= 1;

        Some(elem)
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
        let new_capacity = NonZeroUsize::new(self.capacity * 2).unwrap_or(DEFAULT_SIZE);
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
        self.get(self.len() - 1)
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len() - 1)
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

    /*pub fn iter(&self) -> Iter<'_, T, A> {
        Iter { inner: self, current_ind: 0, remaining: self.len }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T, A> {
        IterMut { current_ind: 0, remaining: self.len, inner: self }
    }

    pub fn drain(&mut self) -> Drain<'_, T, A> {
        let len = self.len;
        self.len = 0;
        Drain {
            len,
            start: self.start,
            inner: self,
        }
    }*/
}

impl<T, A: Allocator> Iterable for ArrayQueue<T, A> {
    type Iter<'a> = Iter<'a, T,A> where Self: 'a;
    type Item = T;

    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            inner: self,
            current_ind: 0,
            remaining: self.len,
        }
    }
}

impl<T, A: Allocator> IterableMut for ArrayQueue<T, A> {
    type IterMut<'a> = IterMut<'a, T, A> where T: 'a, A: 'a;
    type Item = T;
    fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a> {
        IterMut {
            current_ind: 0,
            remaining: self.len,
            inner: self,
        }
    }
}

impl<T, A: Allocator> Drainable for ArrayQueue<T, A> {
    type Drain<'a> = Drain<'a, T,A> where Self: 'a;
    type Item = T;
    fn drain<'a>(&'a mut self) -> Self::Drain<'a> {
        let len = self.len;
        self.len = 0;
        Drain {
            len,
            start: self.start,
            inner: self,
        }
    }
}

impl<T, A: Allocator + Clone> IntoIterator for ArrayQueue<T, A> {
    type IntoIter = IntoIter<T, A>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self }
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

#[cfg(all(test, not(miri)))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn create_insert_100_bench(b: &mut Bencher) {
        b.iter(|| {
            let mut a = ArrayQueue::with_capacity(100);
            for i in 0..100 {
                a.push_back(i).expect("alloc failed");
            }
        });
    }

    #[bench]
    fn create_insert_100_drain_bench(b: &mut Bencher) {
        b.iter(|| {
            let mut a = ArrayQueue::with_capacity(100);
            for i in 0..100 {
                a.push_back(i).expect("alloc failed");
            }
            a.drain().for_each(drop)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::Range;
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

    #[test]
    fn get() {
        const RANGE: Range<usize> = 0..100;
        let queue: ArrayQueue<usize> = (RANGE).collect();
        for i in RANGE {
            assert_eq!(i, *queue.get(i).unwrap())
        }
    }

    #[test]
    fn get2() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(25);
        queue.extend(0..25);
        queue.drain().take(15).for_each(drop);
        queue.extend(25..40);

        for i in 0..25 {
            assert_eq!(i + 15, *queue.get(i).unwrap())
        }
    }

    #[test]
    fn order() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(25);
        queue.extend(0..25);
        queue.drain().take(15).for_each(drop);
        queue.extend(25..40);

        for _ in 0..25 {
            let a = *queue.get(0).unwrap();
            let b = *queue.front().unwrap();
            let popped = queue.pop_front().unwrap();
            assert_eq!(a, b);
            assert_eq!(a, popped);
        }
    }

    #[test]
    fn order2() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(25);
        queue.extend(0..25);
        queue.drain().take(15).for_each(drop);
        queue.extend(25..40);
        println!("{queue:?}");
        for _ in 0..25 {
            let a = *queue.get(queue.len() - 1).unwrap();
            let b = *queue.back().unwrap();
            let popped = queue.pop_back().unwrap();
            assert_eq!(a, b);
            assert_eq!(b, popped);
        }
    }

    #[test]
    fn make_contiguous_single() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(50);
        queue.extend(iter::repeat(0).take(25));
        queue.extend(0..25);
        queue.drain().take(25).for_each(drop);
        let a = queue.make_contiguous();
        for i in 0..25 {
            assert_eq!(i, a[i])
        }
    }

    #[test]
    fn make_contiguous_double() {
        let mut queue = queue_starting_at(20, 10, 0..15);
        //let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(20);
        //queue.extend(iter::repeat(0).take(10));
        //queue.drain().take(10).for_each(drop);
        //queue.extend(0..15);
        let a = queue.make_contiguous();
        for i in 0..15 {
            assert_eq!(i, a[i])
        }
    }

    #[test]
    fn large_extend() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(50);
        queue.extend(iter::repeat(0).take(10000));
    }

    #[test]
    fn remove_single() {
        let mut queue: ArrayQueue<usize> = (0..10).collect();
        assert_eq!(queue.remove(5), Some(5));
        assert_eq!(
            queue.as_slices(),
            ([0, 1, 2, 3, 4, 6, 7, 8, 9].as_ref(), [].as_ref())
        )
    }

    #[test]
    fn remove_double() {
        let mut queue = queue_starting_at(10, 5, 0..9);
        assert_eq!(queue.remove(5), Some(5));
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 6].as_ref(), [7, 8].as_ref())
        );
    }

    #[test]
    fn insert_single() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(10);
        queue.extend(0..=4);
        println!("{queue:?}");
        queue.insert(2, 0).expect("failed to allocate");
        println!("{queue:?}");
        queue.insert(4, 10).expect("failed to allocate");
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([0, 1, 0, 2, 10, 3, 4].as_ref(), [].as_ref())
        )
    }

    #[test]
    fn insert_single_at_end() {
        let mut queue = queue_starting_at(10, 5, 1..5);
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4].as_ref(), [].as_ref()));
        queue.insert(4, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4, 99].as_ref(), [].as_ref()))
    }

    #[test]
    fn insert_single_to_double_fst() {
        let mut queue = queue_starting_at(10, 5, 1..6);
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4, 5].as_ref(), [].as_ref()));
        queue.insert(0, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([99, 1, 2, 3, 4].as_ref(), [5].as_ref()))
    }
    #[test]
    fn insert_single_to_double() {
        let mut queue = queue_starting_at(10, 5, 1..6);
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4, 5].as_ref(), [].as_ref()));
        queue.insert(5, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4, 5].as_ref(), [99].as_ref()));
    }

    #[test]
    fn insert_double() {
        let mut queue = queue_starting_at(10, 5, 1..7);
        println!("{queue:?}");
        assert_eq!(queue.as_slices(), ([1, 2, 3, 4, 5].as_ref(), [6].as_ref()));
        queue.insert(3, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 99, 4].as_ref(), [5, 6].as_ref())
        );
    }

    #[test]
    fn insert_double_in_snd_at_start() {
        let mut queue = queue_starting_at(10, 5, 1..9);
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [6, 7, 8].as_ref())
        );
        queue.insert(5, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [99, 6, 7, 8].as_ref())
        );
    }

    #[test]
    fn insert_double_in_snd_at_end() {
        let mut queue = queue_starting_at(10, 5, 1..9);
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [6, 7, 8].as_ref())
        );
        queue.insert(8, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [6, 7, 8, 99].as_ref())
        );
    }

    #[test]
    fn insert_double_in_snd_at_mid() {
        /*let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(10);
        queue.extend(iter::repeat(0).take(5));
        queue.drain().take(5).for_each(drop);
        queue.extend(1..=8);*/
        let mut queue = queue_starting_at(10, 5, 1..9);
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [6, 7, 8].as_ref())
        );
        queue.insert(7, 99).expect("failed to alloc");
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5].as_ref(), [6, 7, 99, 8].as_ref())
        );
    }

    #[test]
    fn insert_grow_fst() {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(10);
        queue.extend(1..11);
        println!("{queue:?}");
        assert_eq!(
            queue.as_slices(),
            ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10].as_ref(), [].as_ref())
        );
        queue.insert(0, 0).expect("failed to alloc");
        assert_eq!(
            queue.as_slices(),
            ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].as_ref(), [].as_ref())
        );
    }

    fn queue_starting_at(capacity: usize, start: usize, range: Range<usize>) -> ArrayQueue<usize> {
        let mut queue: ArrayQueue<usize> = ArrayQueue::with_capacity(capacity);
        queue.extend(iter::repeat(0).take(start));
        queue.drain().take(start).for_each(drop);
        queue.extend(range);
        queue
    }
}
