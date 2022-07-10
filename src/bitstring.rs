use alloc::alloc::Global;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::fmt::{Binary, Debug, Formatter};
use core::ops::*;

// used to work with endianness
const SHIFT_OP: &dyn Fn(u8, usize) -> u8 = if cfg!(target_endian = "little") {
    &core::ops::Shl::shl
} else {
    &core::ops::Shr::shr
};
#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct BitContainer(u8);

impl BitContainer {
    /// sets the bit at `idx` to whatever
    #[inline]
    pub fn set(&mut self, idx: usize, val: bool) {
        if val {
            self.0 |= SHIFT_OP(1, idx % 8)
        } else {
            self.0 &= !SHIFT_OP(1, idx % 8)
        };
    }
    #[inline]
    pub fn get(&self, idx: usize) -> bool {
        self.0 & SHIFT_OP(0x01, idx % 8) != 0
    }
    #[inline]
    pub fn toggle(&mut self, index: usize) {
        self.0 ^= SHIFT_OP(1, index % 8)
    }
}
/*
impl Debug for BitContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
} */
impl Binary for BitContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

impl Deref for BitContainer {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct BitString<A: Allocator = Global> {
    data: Vec<BitContainer, A>,
    len: usize,
}

impl BitString {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            len: 0,
        }
    }
}

impl<A: Allocator> BitString<A> {
    pub fn iter(&self) -> BitStringIterator<'_> {
        BitStringIterator {
            inner: self.data.as_slice(),
            idx: 0,
        }
    }
    pub fn iter_mut(&mut self) -> BitStringIteratorMut<'_> {
        BitStringIteratorMut {
            inner: self.data.as_mut_slice(),
            idx: 0,
        }
    }
}

pub struct FixedBitString<const N: usize> {
    inner: [BitContainer; N],
    cursor: usize,
    len: usize,
}

impl<const N: usize> Default for FixedBitString<N> {
    fn default() -> Self {
        FixedBitString {
            inner: [BitContainer::default(); N],
            cursor: 0,
            len: 0,
        }
    }
}

impl<const N: usize> FixedBitString<N> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn toggle(&mut self, index: usize) {
        self.inner[index.div_floor(8)].toggle(index)
    }

    #[inline]
    pub fn reset(&mut self) {
        self.inner = [BitContainer::default(); N];
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.iter().all(|f| f.deref() == &0)
    }

    /// sets the bit at `idx` to whatever
    #[inline]
    pub fn set(&mut self, idx: usize, val: bool) {
        self.inner[idx.div_floor(8)].set(idx, val)
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<bool> {
        (idx < N * 8).then(|| self.inner[idx.div_floor(8)].get(idx))
    }

    #[inline]
    pub fn iter(&self) -> BitStringIterator<'_> {
        BitStringIterator {
            inner: self.inner.as_slice(),
            idx: 0,
        }
    }

    pub fn copy_from_bool_slice(&mut self, idx: usize, slice: &[bool]) {
        BitStringIteratorMut {
            idx,
            inner: self.inner.as_mut_slice(),
        }
        .zip(slice.iter())
        .for_each(|(mut a, b)| a.set(*b));
    }
    pub fn copy_from_slice(&mut self, idx: usize, slice: &[u8]) {
        // not sure if this is right :/. too tired.
        if N - idx > slice.len() {
            panic!("out of bounds");
        }
        // this transmute should be fine since Bitcontainer is `#[repr(transparent)]`
        let slice: &[BitContainer] = unsafe { core::mem::transmute(slice) };
        self.inner[idx..idx + slice.len()].copy_from_slice(slice);
    }
}

impl<const N: usize> Debug for FixedBitString<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let Self { inner, cursor, len } = self;
        write!(
            f,
            "FixedBitString<{N}> {{ len: {len}, cursor: {cursor}, data: ["
        )?;
        let mut iter = inner.iter();
        if let Some(i) = iter.next() {
            write!(f, "{:b}", *i)?;
        }
        for i in iter {
            write!(f, ",{:b}", *i)?;
        }
        write!(f, "] }}")
    }
}

pub struct BitStringIterator<'a> {
    inner: &'a [BitContainer],
    idx: usize,
}
impl<'a> Iterator for BitStringIterator<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.inner.len() * 8 {
            return None;
        }
        let r = self.inner[self.idx.div_floor(8)].get(self.idx);
        self.idx += 1;
        Some(r)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.inner.len() * 8) - self.idx;
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for BitStringIterator<'a> {}

pub struct BitStringIteratorMut<'a> {
    idx: usize,
    inner: &'a mut [BitContainer],
}

impl<'a> Iterator for BitStringIteratorMut<'a> {
    type Item = BitRefMut<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.inner.len() * 8 {
            return None;
        }
        let inner: &'a mut BitContainer =
            unsafe { core::mem::transmute(self.inner.get_mut(self.idx).unwrap()) };

        let r = BitRefMut {
            inner,
            index: (self.idx % 8) as u8,
        };
        self.idx += 1;
        Some(r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.inner.len() * 8) - self.idx;
        (len, Some(len))
    }
}
impl<'a> ExactSizeIterator for BitStringIteratorMut<'a> {}

pub struct BitRefMut<'a> {
    inner: &'a mut BitContainer,
    index: u8,
}

impl<'a> BitRefMut<'a> {
    #[inline]
    pub fn get(&self) -> bool {
        self.inner.get(self.index as usize)
    }
    #[inline]
    pub fn set(&mut self, val: bool) {
        self.inner.set(self.index as usize, val)
    }
    #[inline]
    pub fn toggle(&mut self) {
        self.inner.toggle(self.index as usize)
    }
}

impl<'a> From<BitRefMut<'a>> for bool {
    fn from(x: BitRefMut<'a>) -> Self {
        x.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_bit_string_1() {
        let mut a = FixedBitString::<8>::default();
        a.set(7, true);
        println!("{a:?}");
        a.set(6, true);
        println!("{a:?}");
        a.set(5, true);
        println!("{a:?}");
        a.set(7, false);
        println!("{a:?}");
        a.set(6, false);
        println!("{a:?}");
        a.set(5, false);
    }

    #[test]
    fn fixed_bit_string_2() {
        let mut a = FixedBitString::<8>::default();
        a.set(2, true);
        println!("{a:?}");
        a.set(2, false);
        println!("{a:?}");
        assert!(!a.get(2).unwrap());
    }

    #[test]
    fn fixed_bit_string_iter() {
        let mut a = FixedBitString::<8>::default();
        a.set(2, true);
        println!("{a:?}");
        a.set(2, false);
        println!("{a:?}");
        assert!(!a.get(2).unwrap());
    }
}
