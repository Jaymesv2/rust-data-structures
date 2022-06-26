use core::{
    alloc::{Allocator, AllocError, Layout},
    ptr::{self, drop_in_place, NonNull},
};
use alloc::alloc::Global;

type NodePtr<T,A> = NonNull<DoublyLinkedListNode<T,A>>;

pub struct DoublyLinkedList<T, A: Allocator + Clone = Global> {
    head: Option<NodePtr<T,A>>,
    tail: Option<NodePtr<T,A>>,
    len: usize,
    alloc: A
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> usize {
        todo!()
    }
}

impl<T,A: Allocator + Clone> DoublyLinkedList<T,A> {
    pub fn new_in(alloc: A) -> Self {
        Self { head: None, tail: None, alloc ,len: 0}
    }

    pub fn push_front(&mut self, item: T) -> Result<(), AllocError> {
        todo!()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        todo!()
    }

    pub fn push_back(&mut self, item: T) -> Result<(), AllocError> {
        todo!()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        todo!()
    }

    pub fn front(&self) -> Option<&T> {
        todo!()
    }

    pub fn back(&self) -> Option<&T> {
        todo!()
    }

    //pub fn iter(&self) -> Iter<'_, T,A> {todo!()}
    //pub fn iter_mut(&self) -> IterMut<'_, T,A> {todo!()}
    //pub fn drain(&self) -> Drain<'_, T,A> {todo!()}
    //pub fn iter(&self) -> IntoIter<T,A> {todo!()}
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new_in(Global)
    }
}

pub struct DoublyLinkedListNode<T,A: Allocator + Clone> {
    value: T,
    prev: Option<NodePtr<T,A>>,
    next: Option<NodePtr<T,A>>,
    alloc: A,
}

impl<T,A: Allocator + Clone> DoublyLinkedListNode<T,A> {
    const LAYOUT: Layout = Layout::new::<Self>();

    pub unsafe fn drop(ptr: NonNull<Self>) {
        let alloc = ptr.as_ref().alloc.clone();
        drop_in_place(ptr.as_ptr());
        alloc.deallocate(ptr.cast(), Self::LAYOUT);
    }
    /// consumes the value at ptr
    pub unsafe fn to_owned(ptr: NonNull<Self>) -> Self {
        let node = ptr::read(ptr.as_ptr());
        node.alloc.deallocate(ptr.cast(), Self::LAYOUT);
        node
    }

    pub unsafe fn unwrap(ptr: NonNull<Self>) -> T {
        Self::to_owned(ptr).value
    }

    pub unsafe fn new(alloc: A, value: T, next: Option<NonNull<DoublyLinkedListNode<T,A>>>, prev: Option<NonNull<DoublyLinkedListNode<T,A>>>) -> Result<NonNull<Self>, AllocError> {
        let ptr: NonNull<Self> = alloc.allocate(Self::LAYOUT)?.cast();
        
        ptr::write(ptr.as_ptr(), Self {
            value,
            prev,
            next,
            alloc,
        });

        Ok(ptr)
    }
/* 
    pub unsafe fn deref(ptr: NonNull<Self>) -> &Self {
        todo!()
    }

    pub unsafe fn deref_mut(ptr: NonNull<Self>) -> &mut Self {
        todo!()
    }*/
}

//use iters::*;
mod iters {

}


#[cfg(test)]
mod tests {

}