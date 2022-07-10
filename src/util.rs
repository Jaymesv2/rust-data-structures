pub trait IteratorExactExt: Iterator + ExactSizeIterator {
    fn groups<const N: usize>(&mut self) -> Groups<'_, Self, N>
    where
        Self: Sized,
    {
        Groups {
            inner: self,
            iterations: 0,
        }
    }

    fn collect_to_array<const N: usize>(mut self) -> Option<[Self::Item; N]>
    where
        Self: Sized,
        [Self::Item; N - 1]: Sized,
    {
        if self.len() != N {
            return None;
        }
        Some(core::array::from_fn(|_| self.next().unwrap()))
    }
}

impl<T: ExactSizeIterator + Iterator + Sized> IteratorExactExt for T {}

pub struct Groups<'a, I: Iterator + ExactSizeIterator, const N: usize> {
    inner: &'a mut I,
    iterations: usize,
}
/*
impl<'a, I: Iterator + ExactSizeIterator, const N: usize> Groups<'a, I, N> {
    pub fn new(inner: &'a mut I) -> Groups<'a, I, N> {
        Self {
            inner,
            iterations: 0,
        }
    }
}*/

impl<'a, I: Iterator + ExactSizeIterator, const N: usize> Iterator for Groups<'a, I, N> {
    type Item = [I::Item; N];
    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.len() < N {
            return None;
        }
        self.iterations += 1;
        Some(core::array::from_fn(|_| self.inner.next().unwrap()))
    }
}

impl<'a, I: Iterator + ExactSizeIterator, const N: usize> ExactSizeIterator for Groups<'a, I, N> {
    fn len(&self) -> usize {
        self.inner.len().div_floor(N)
    }
}

/*
const fn type_eq<T, U, const N: usize>() -> bool {
    size_of::<[T; N]>() == size_of::<[U; N]>()
        && size_of::<T>() == size_of::<U>()
        && align_of::<[T; N]>() == align_of::<[U; N]>()
        && align_of::<T>() == align_of::<U>()
}*/

pub trait IteratorExt: Iterator {
    /// tries to collect
    unsafe fn collect_to_array_lossy_unchecked<const N: usize>(mut self) -> [Self::Item; N]
    where
        Self: Sized,
        [Self::Item; N - 1]: Sized,
    {
        core::array::from_fn(|_| self.next().unwrap_unchecked())
    }
    fn collect_to_array_lossy<const N: usize>(mut self) -> Option<[Self::Item; N]>
    where
        Self: Sized,
        [Self::Item; N - 1]: Sized,
        /*
        MaybeUninit<Self::Item>: Sized,
        Self::Item: Sized,
        [MaybeUninit<Self::Item>; N - 1]: Sized,
        [(); type_eq::<Self::Item, MaybeUninit<Self::Item>, N>() as usize - 1]: Sized,*/
    {
        // i was hoping that this would work but oh well
        /*
        let mut data: [MaybeUninit<Self::Item>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut i = 0;
        while i < N {
            let elem = if let Some(s) = self.next() {
                s
            } else {
                break;
            };
            unsafe { *data.as_mut_ptr().add(i) = MaybeUninit::new(elem) };
            i += 1;
        }

        if i != N {
            let x = unsafe { transmute::<[MaybeUninit<Self::Item>; N], [Self::Item; N]>(data) };
            Some(x)
        } else {
            for i2 in 0..i {
                unsafe { data.get_unchecked_mut(i2).assume_init_drop() };
            }
            core::mem::forget(data);
            None
        }*/
        // this is easy :/
        core::array::try_from_fn(|_| self.next())
    }
}

impl<T: Iterator + Sized> IteratorExt for T {}
