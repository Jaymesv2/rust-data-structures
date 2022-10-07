//! A generic reference counter over atomic and non atomic reference counting

use core::pin::Pin;
use core::ops::Deref;
//use core::ops::CoerceUnsized;


/*
    shared:
    --AsRef<T>
    --Borrow<T>
    --Clone
    --CoerceUnsized<Arc<U>>
    --Debug
    --Default
    --Deref
    --DispatchFromDyn<Arc<U>>
    --Display
    --Drop
    --Eq
    --From<&[T]>
    --From<&CStr>
    --From<&str>
    --From<Box<T, Global>>
    --From<CString>
    --From<String>
    --From<T>
    --From<Vec<T, Global>>
    --FromIterator<T>
    --Hash
    --Ord
    --PartialEq<Arc<T>>
    --PartialOrd<Arc<T>>
    --Pointer
    --TryFrom<Arc<[T]>>
    --Unpin
    --UnwindSafe
    
    arc has:
    Error
    From<Arc<W>>
    From<Arc<W>>
    From<Arc<str>>
    From<Cow<'a, B>>
    Send
    Sync
    
    rc has:
    From<Rc<str>>
    RefUnwindSafe
    !Send
    !Sync
*/
#[repr(transparent)]
pub struct Rc<T: ?Sized, S: RcStub = StRc>(<S as RcStub>::Counter<T>);

impl<T: ?Sized, S: RcStub> Rc<T, S> {
    #[inline(always)]
    pub fn as_ptr(this: &Self) -> *const T {
        S::Counter::as_ptr(&this.0)
    }

    #[inline(always)]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        S::Counter::get_mut(&mut this.0)
    }

    #[inline(always)]
    pub fn into_raw(this: Self) -> *const T {
        S::Counter::into_raw(this.0)
    }

    #[inline(always)]
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        Self(S::Counter::from_raw(ptr))
    }

    #[inline(always)]
    pub fn downgrade(this: &Self) -> WeakStub<T,S> {
        WeakStub(S::Counter::downgrade(&this.0))
    }

    #[inline(always)]
    pub fn weak_count(this: &Self) -> usize {
        S::Counter::weak_count(&this.0)
    }

    #[inline(always)]
    pub fn strong_count(this: &Self) -> usize {
        S::Counter::strong_count(&this.0)
    }

    #[inline(always)]
    pub unsafe fn increment_strong_count(ptr: *const T) {
        S::Counter::increment_strong_count(ptr)
    }

    #[inline(always)]
    pub unsafe fn decrement_strong_count(ptr: *const T) {
        S::Counter::decrement_strong_count(ptr)
    }

    #[inline(always)]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        S::Counter::ptr_eq(&this.0, &other.0)
    }
}

impl<T, S: RcStub> Rc<T, S>
where
    S::Counter<T>: RctSizedT<T> 
{
    pub fn new(item: T) -> Self {
        Self(S::Counter::new(item))
    }

    pub fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&<S::Counter<T> as Rct<T>>::Weak) -> T,
    {
        Self(S::Counter::new_cyclic(data_fn))
    }
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        S::Counter::try_unwrap(this.0).map_err(|e| Self(e))
    }
    pub fn pin(data: T) -> Pin<Self> {
        unsafe {Pin::new_unchecked(Self::new(data))}
    }
}

impl<T: Clone, S: RcStub> Rc<T, S>
where
    S::Counter<T>: RctCloneableT<T>,
{
    pub fn make_mut(this: &mut Self) -> &mut T {
        S::Counter::make_mut(&mut this.0)
    }
}

impl<T: ?Sized, S: RcStub> Deref for Rc<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        S::Counter::deref(&self.0)
    }
}

impl<T: ?Sized, S: RcStub> Clone for Rc<T, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/*impl<T, S:RcStub> RctSizedT<T> for RcStubb<T,S> {
    
}*/
#[repr(transparent)]
pub struct WeakStub<T: ?Sized, S: RcStub = StRc>(<S as RcStub>::Weak<T>);

use traits::*;
pub use arc_impl::AtRc;
pub use rc_impl::StRc;

mod traits {
    use core::pin::Pin;
    use core::ops::Deref;
    pub trait RcStub {
        type Counter<T: ?Sized>: Rct<T, Weak = Self::Weak<T>>;
        type Weak<T: ?Sized>: Weakt<T, Parent = Self::Counter<T>>;
    }

    pub trait Rct<T: ?Sized>: Clone + AsRef<T> + Deref<Target = T>/*+ Borrow<T>*/ {
        type Weak: Weakt<T, Parent = Self>;
        fn as_ptr(this: &Self) -> *const T;
        fn get_mut(this: &mut Self) -> Option<&mut T>;
        fn into_raw(this: Self) -> *const T;
        unsafe fn from_raw(ptr: *const T) -> Self;
        fn downgrade(this: &Self) -> Self::Weak;
        fn weak_count(this: &Self) -> usize;
        fn strong_count(this: &Self) -> usize;
        unsafe fn increment_strong_count(ptr: *const T);
        unsafe fn decrement_strong_count(ptr: *const T);
        fn ptr_eq(this: &Self, other: &Self) -> bool;
    }

    pub trait RctSizedT<T>: Rct<T> {
        fn new(item: T) -> Self;
        fn try_unwrap(this: Self) -> Result<T, Self>;
        fn pin(data: T) -> Pin<Self>;
        fn new_cyclic<F>(data_fn: F) -> Self
        where
            F: FnOnce(&Self::Weak) -> T;
    }

    pub trait RctCloneableT<T: Clone>: Rct<T> {
        fn make_mut(this: &mut Self) -> &mut T;
    }

    pub trait Weakt<T: ?Sized> {
        type Parent: Rct<T, Weak = Self>;
        fn upgrade(&self) -> Option<Self::Parent>;
        fn as_ptr(&self) -> *const T;
        fn into_raw(self) -> *const T;
        unsafe fn from_raw(ptr: *const T) -> Self;
        fn strong_count(&self) -> usize;
        fn weak_count(&self) -> usize;
        fn ptr_eq(&self, other: &Self) -> bool;
    }

    pub trait WeaktSizedT<T>: Weakt<T> {
        fn new() -> Self;
    }
}


mod rc_impl {
    use super::*;
    use alloc::rc::{Rc, Weak};

    pub enum StRc {}

    impl RcStub for StRc {
        type Counter<T: ?Sized> = Rc<T>;
        type Weak<T: ?Sized> = Weak<T>;
    }

    impl<T: Clone> RctCloneableT<T> for Rc<T> {
        #[inline(always)]
        fn make_mut(this: &mut Self) -> &mut T {
            Rc::make_mut(this)
        }
    }

    impl<T: ?Sized> Rct<T> for Rc<T> {
        type Weak = Weak<T>;
        #[inline(always)]
        fn as_ptr(this: &Self) -> *const T {
            Rc::as_ptr(this)
        }
        #[inline(always)]
        fn get_mut(this: &mut Self) -> Option<&mut T> {
            Rc::get_mut(this)
        }
        #[inline(always)]
        fn into_raw(this: Self) -> *const T {
            Rc::into_raw(this)
        }
        #[inline(always)]
        unsafe fn from_raw(ptr: *const T) -> Self {
            Rc::from_raw(ptr)
        }
        #[inline(always)]
        fn downgrade(this: &Self) -> Self::Weak {
            Rc::downgrade(this)
        }
        #[inline(always)]
        fn weak_count(this: &Self) -> usize {
            Rc::weak_count(this)
        }
        #[inline(always)]
        fn strong_count(this: &Self) -> usize {
            Rc::strong_count(this)
        }
        #[inline(always)]
        unsafe fn increment_strong_count(ptr: *const T) {
            Rc::increment_strong_count(ptr)
        }
        #[inline(always)]
        unsafe fn decrement_strong_count(ptr: *const T) {
            Rc::decrement_strong_count(ptr)
        }
        #[inline(always)]
        fn ptr_eq(this: &Self, other: &Self) -> bool {
            Rc::ptr_eq(this, other)
        }
        
    }

    impl<T> RctSizedT<T> for Rc<T> {
        #[inline(always)]
        fn new(item: T) -> Self {
            Rc::new(item)
        }
        #[inline(always)]
        fn new_cyclic<F>(data_fn: F) -> Self
        where
            F: FnOnce(&Self::Weak) -> T,
        {
            Rc::new_cyclic(data_fn)
        }
        #[inline(always)]
        fn try_unwrap(this: Self) -> Result<T, Self> {
            Rc::try_unwrap(this)
        }
        #[inline(always)]
        fn pin(data: T) -> Pin<Self> {
            Rc::pin(data)
        }
    }

    impl<T: ?Sized> Weakt<T> for Weak<T> {
        type Parent = Rc<T>;
        #[inline(always)]
        fn upgrade(&self) -> Option<Self::Parent> {
            Weak::upgrade(&self)
        }
        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            Weak::as_ptr(&self)
        }
        #[inline(always)]
        fn into_raw(self) -> *const T {
            Weak::into_raw(self)
        }
        #[inline(always)]
        unsafe fn from_raw(ptr: *const T) -> Self {
            Weak::from_raw(ptr)
        }
        #[inline(always)]
        fn strong_count(&self) -> usize {
            Weak::strong_count(self)
        }
        #[inline(always)]
        fn weak_count(&self) -> usize {
            Weak::weak_count(&self)
        }
        #[inline(always)]
        fn ptr_eq(&self, other: &Self) -> bool {
            Weak::ptr_eq(&self, other)
        }
    }

    impl<T> WeaktSizedT<T> for Weak<T> {
        #[inline(always)]
        fn new() -> Self {
            Weak::new()
        }
    }
}

mod arc_impl {
    use super::*;
    pub enum AtRc {}
    use alloc::sync::{Arc, Weak as WeakA};

    impl RcStub for AtRc {
        type Counter<T: ?Sized> = Arc<T>;
        type Weak<T: ?Sized> = WeakA<T>;
    }

    impl<T: ?Sized> Rct<T> for Arc<T> {
        type Weak = WeakA<T>;
        #[inline(always)]
        fn as_ptr(this: &Self) -> *const T {
            Arc::as_ptr(this)
        }
        #[inline(always)]
        fn get_mut(this: &mut Self) -> Option<&mut T> {
            Arc::get_mut(this)
        }
        #[inline(always)]
        fn into_raw(this: Self) -> *const T {
            Arc::into_raw(this)
        }
        #[inline(always)]
        unsafe fn from_raw(ptr: *const T) -> Self {
            Arc::from_raw(ptr)
        }
        #[inline(always)]
        fn downgrade(this: &Self) -> Self::Weak {
            Arc::downgrade(this)
        }
        #[inline(always)]
        fn weak_count(this: &Self) -> usize {
            Arc::weak_count(this)
        }
        #[inline(always)]
        fn strong_count(this: &Self) -> usize {
            Arc::strong_count(this)
        }
        #[inline(always)]
        unsafe fn increment_strong_count(ptr: *const T) {
            Arc::increment_strong_count(ptr)
        }
        #[inline(always)]
        unsafe fn decrement_strong_count(ptr: *const T) {
            Arc::decrement_strong_count(ptr)
        }
        #[inline(always)]
        fn ptr_eq(this: &Self, other: &Self) -> bool {
            Arc::ptr_eq(this, other)
        }
    }

    impl<T> RctSizedT<T> for Arc<T> {
        #[inline(always)]
        fn new(item: T) -> Self {
            Arc::new(item)
        }
        #[inline(always)]
        fn new_cyclic<F>(data_fn: F) -> Self
        where
            F: FnOnce(&Self::Weak) -> T,
        {
            Arc::new_cyclic(data_fn)
        }
        #[inline(always)]
        fn try_unwrap(this: Self) -> Result<T, Self> {
            Arc::try_unwrap(this)
        }
        #[inline(always)]
        fn pin(data: T) -> Pin<Self> {
            Arc::pin(data)
        }
    }
    
    impl<T: Clone> RctCloneableT<T> for Arc<T> {
        #[inline(always)]
        fn make_mut(this: &mut Self) -> &mut T {
            Arc::make_mut(this)
        }
    }

    impl<T: ?Sized> Weakt<T> for WeakA<T> {
        type Parent = Arc<T>;
        #[inline(always)]
        fn upgrade(&self) -> Option<Self::Parent> {
            WeakA::upgrade(&self)
        }
        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            WeakA::as_ptr(&self)
        }
        #[inline(always)]
        fn into_raw(self) -> *const T {
            WeakA::into_raw(self)
        }
        #[inline(always)]
        unsafe fn from_raw(ptr: *const T) -> Self {
            WeakA::from_raw(ptr)
        }
        #[inline(always)]
        fn strong_count(&self) -> usize {
            WeakA::strong_count(&self)
        }
        #[inline(always)]
        fn weak_count(&self) -> usize {
            WeakA::weak_count(&self)
        }
        #[inline(always)]
        fn ptr_eq(&self, other: &Self) -> bool {
            WeakA::ptr_eq(&self, other)
        }
    }
    
    impl<T> WeaktSizedT<T> for WeakA<T> {
        #[inline(always)]
        fn new() -> Self {
            WeakA::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[derive(Debug)]
    struct TestStruct<T, R = StRc>
    where
        R: RcStub,
    {
        inner: Rc<T, R>,
    }
    /*
    impl<T, R: RcStub> TestStruct<T, R> {
        fn new(item: T) -> Self {
            Self {
                inner: Rc::new(item),
            }
        }
    }
    */

    impl<T, R: RcStub> Clone for TestStruct<T, R> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    #[test]
    fn rc_test() {
        use std::thread::spawn;
        let non_safe_struct: Rc<_, AtRc> = Rc::new(5);
        for _ in 0..5 {
            let mut x = non_safe_struct.clone();
            spawn(move || {
                let u: &mut i32 = Rc::make_mut(&mut x);
                println!("{u}");
            });
        }
    }
}
