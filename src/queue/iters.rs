use super::ArrayQueue;
use alloc::alloc::Global;
use core::{alloc::Allocator, iter::FusedIterator, mem::transmute, ptr};

pub struct Iter<'a, T, A: Allocator = Global> {
    pub(crate) inner: &'a ArrayQueue<T, A>,
    pub(crate) current_ind: usize,
    pub(crate) remaining: usize,
}

impl<'a, T: 'a, A: Allocator> Iterator for Iter<'a, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        (self.remaining != 0).then(|| {
            self.remaining -= 1;
            self.current_ind += 1;
            unsafe { self.inner.get_unchecked(self.current_ind - 1) }
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
    pub(crate) inner: &'a mut ArrayQueue<T, A>,
    pub(crate) current_ind: usize,
    pub(crate) remaining: usize,
}

impl<'a, T: 'a, A: Allocator> Iterator for IterMut<'a, T, A> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        (self.remaining != 0).then(|| {
            self.remaining -= 1;
            self.current_ind += 1;
            // the transmute makes the lifetimes happy
            unsafe { transmute(self.inner.get_unchecked_mut(self.current_ind - 1)) }
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
    pub(crate) inner: &'a mut ArrayQueue<T, A>,
    pub(crate) len: usize,
    pub(crate) start: usize,
}

impl<'a, T: 'a, A: Allocator> Iterator for Drain<'a, T, A> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        (self.len != 0).then(|| {
            let elem = unsafe { ptr::read(self.inner.ptr.as_ptr().add(self.start)) };
            self.start = (self.start + 1) % (self.inner.capacity);
            self.len -= 1;
            elem
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
                ptr::read(
                    self.inner
                        .ptr
                        .as_ptr()
                        .add((self.start + self.len) % self.inner.capacity),
                )
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
    pub(crate) inner: ArrayQueue<T, A>,
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
