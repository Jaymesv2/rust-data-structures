#![allow(dead_code)]
use alloc::alloc::Global;
use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::{self, drop_in_place, NonNull},
};

type NodePtr<T, A> = NonNull<DoublyLinkedListNode<T, A>>;

pub struct DoublyLinkedList<T, A: Allocator + Clone = Global> {
    head: Option<NodePtr<T, A>>,
    tail: Option<NodePtr<T, A>>,
    len: usize,
    alloc: A,
}

/*
a is head, b is a's next and a is b's prev
so for b
next goes toward the tail
 a <-prev-> b <-next-> c

*/

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

use core::fmt::Debug;

impl<T: Debug, A: Allocator + Clone> Debug for DoublyLinkedList<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DoublyLinkedList {{ ")?;
        let mut cur = self.head;
        while let Some(s) = cur {
            let r = unsafe { s.as_ref() };
            cur = r.next;
            write!(f, "{:?} ", &r.value)?;
        }
        write!(f, "}}")
    }
}

impl<T, A: Allocator + Clone> DoublyLinkedList<T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self {
            head: None,
            tail: None,
            alloc,
            len: 0,
        }
    }

    pub fn push_front(&mut self, item: T) -> Result<(), AllocError> {
        if self.len == 0 {
            let node = unsafe { DoublyLinkedListNode::new(self.alloc.clone(), item, None, None) }?;
            self.head = Some(node);
            self.tail = Some(node);
        } else {
            // pushing putting on ahead of the head
            let node =
                unsafe { DoublyLinkedListNode::new(self.alloc.clone(), item, self.head, None) }?;
            unsafe { self.head.unwrap().as_mut() }.prev = Some(node);
            self.head = Some(node);
        }
        self.len += 1;
        Ok(())
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let r = match self.head {
            Some(node) => unsafe {
                let v: T = core::ptr::read(&node.as_ref().value);
                self.head = node.as_ref().next;
                if let Some(mut node) = self.head {
                    node.as_mut().prev = None;
                }
                DoublyLinkedListNode::drop(node);
                Some(v)
            },
            None => None,
        };
        self.len -= 1;
        if self.len == 0 {
            self.tail = None;
        }
        r
    }

    pub fn push_back(&mut self, item: T) -> Result<(), AllocError> {
        if self.len == 0 {
            let node = unsafe { DoublyLinkedListNode::new(self.alloc.clone(), item, None, None) }?;
            self.head = Some(node);
            self.tail = Some(node);
        } else {
            // pushing putting on ahead of the head
            let node =
                unsafe { DoublyLinkedListNode::new(self.alloc.clone(), item, None, self.tail) }?;
            unsafe { self.tail.unwrap().as_mut() }.next = Some(node);
            self.tail = Some(node);
        }
        self.len += 1;
        Ok(())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let r = match self.tail {
            Some(node) => unsafe {
                let v: T = core::ptr::read(&node.as_ref().value);
                self.tail = node.as_ref().prev;
                if let Some(mut node) = self.tail {
                    node.as_mut().next = None;
                }
                DoublyLinkedListNode::drop(node);
                Some(v)
            },
            None => None,
        };
        self.len -= 1;
        if self.len == 0 {
            self.head = None;
        }
        r
    }

    fn front(&self) -> Option<&T> {
        self.head.map(|x| unsafe { &x.as_ref().value })
    }

    fn front_mut(&mut self) -> Option<&mut T> {
        self.head.map(|mut x| unsafe { &mut x.as_mut().value })
    }

    fn back(&self) -> Option<&T> {
        self.tail.map(|x| unsafe { &x.as_ref().value })
    }

    fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut x| unsafe { &mut x.as_mut().value })
    }

    fn get(&self, idx: usize) -> Option<&T> {
        let mut node = self.head;
        for _ in 0..idx {
            if let Some(s) = node {
                node = unsafe { s.as_ref().next };
            } else {
                break;
            }
        }
        node.map(|x| unsafe { &x.as_ref().value })
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        let mut node = self.head;
        for _ in 0..idx {
            if let Some(s) = node {
                node = unsafe { s.as_ref().next };
            } else {
                break;
            }
        }
        node.map(|mut x| unsafe { &mut x.as_mut().value })
    }

    //pub fn iter(&self) -> Iter<'_, T,A> {todo!()}
    //pub fn iter_mut(&self) -> IterMut<'_, T,A> {todo!()}
    //pub fn drain(&self) -> Drain<'_, T,A> {todo!()}
    //pub fn iter(&self) -> IntoIter<T,A> {todo!()}
}

pub struct Cursor<'a, T, A: Allocator + Clone> {
    list: &'a DoublyLinkedList<T, A>,
    node: Option<NodePtr<T, A>>,
    index: usize,
}

impl<'a, T, A: Allocator + Clone> Cursor<'a, T, A> {
    pub fn next(&mut self) {
        if let Some(n) = self.node {
            self.node = unsafe { n.as_ref().next };
            if self.node.is_some() {
                self.index += 1;
            }
        }
    }

    pub fn prev(&mut self) {
        if let Some(n) = self.node {
            self.node = unsafe { n.as_ref().prev };
            if self.node.is_some() {
                self.index -= 1;
            }
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn peek(&self) -> Option<&'a T> {
        self.node.map(|x| unsafe { &x.as_ref().value })
    }

    pub fn jump_to_head(&mut self) {
        self.node = self.list.head;
        self.index = 0;
    }
    pub fn jump_to_tail(&mut self) {
        self.node = self.list.tail;
        self.index = self.list.len;
    }
}

pub struct CursorMut<'a, T, A: Allocator + Clone> {
    list: &'a mut DoublyLinkedList<T, A>,
    node: Option<NodePtr<T, A>>,
    index: usize,
}

impl<'a, T, A: Allocator + Clone> CursorMut<'a, T, A> {
    pub fn next(&mut self) {
        if let Some(n) = self.node {
            self.node = unsafe { n.as_ref().next };
            if self.node.is_some() {
                self.index += 1;
            }
        }
    }

    pub fn prev(&mut self) {
        if let Some(n) = self.node {
            self.node = unsafe { n.as_ref().prev };
            if self.node.is_some() {
                self.index -= 1;
            }
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn get(&self) -> Option<&'a T> {
        self.node.map(|x| unsafe { &x.as_ref().value })
    }

    pub fn get_mut(&mut self) -> Option<&'a mut T> {
        self.node.map(|mut x| unsafe { &mut x.as_mut().value })
    }

    pub fn remove_next(&mut self) -> Option<T> {
        todo!()
    }
    pub fn remove_prev(&mut self) -> Option<T> {
        todo!()
    }
    pub fn remove_current(&mut self) -> Option<T> {
        todo!()
    }
    pub fn append_next(&mut self, _other: DoublyLinkedList<T>) {
        todo!()
    }
    pub fn append_prev(&mut self, _other: DoublyLinkedList<T>) {
        todo!()
    }
    pub fn split(&mut self) -> DoublyLinkedList<T> {
        todo!()
    }

    /*
    pub fn remove(&mut self) -> Result<Direction, ()> {
        let ptr = unsafe {self.node.and_then(|mut n| DoublyLinkedListNode::delete_node(n))};
    }
     */

    //pub fn goto_index(&mut self) {}

    pub fn jump_to_head(&mut self) {
        self.node = self.list.head;
        self.index = 0;
    }
    pub fn jump_to_tail(&mut self) {
        self.node = self.list.tail;
        self.index = self.list.len;
    }
}

pub struct Iter<'a, T, A: Allocator + Clone> {
    list: &'a DoublyLinkedList<T, A>,
    node: Option<NodePtr<T, A>>,
}

impl<'a, T, A: Allocator + Clone> Iterator for Iter<'a, T, A> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.node {
            let r: &'a DoublyLinkedListNode<T, A> = unsafe { s.as_ref() };
            self.node = r.next;
            Some(&r.value)
        } else {
            None
        }
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new_in(Global)
    }
}

pub struct DoublyLinkedListNode<T, A: Allocator + Clone> {
    value: T,
    prev: Option<NodePtr<T, A>>,
    next: Option<NodePtr<T, A>>,
    alloc: A,
}

impl<T, A: Allocator + Clone> DoublyLinkedListNode<T, A> {
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
    /// Deletes the node and updates its neighbors next and prev values to point to eachother.
    pub unsafe fn delete_node(mut ptr: NonNull<Self>) -> Option<NonNull<Self>> {
        let (next, prev) = {
            let r = unsafe { ptr.as_mut() };
            (r.next, r.prev)
        };
        let a = next.and_then(|mut x| {
            x.as_mut().prev = prev;
            prev
        });
        let b = prev.and_then(|mut x| {
            x.as_mut().next = next;
            next
        });
        Self::drop(ptr);
        a.or(b)
    }

    pub unsafe fn new(
        alloc: A,
        value: T,
        next: Option<NonNull<DoublyLinkedListNode<T, A>>>,
        prev: Option<NonNull<DoublyLinkedListNode<T, A>>>,
    ) -> Result<NonNull<Self>, AllocError> {
        let ptr: NonNull<Self> = alloc.allocate(Self::LAYOUT)?.cast();

        ptr::write(
            ptr.as_ptr(),
            Self {
                value,
                prev,
                next,
                alloc,
            },
        );

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
mod iters {}

#[cfg(test)]
mod tests {
    use super::DoublyLinkedList;

    #[test]
    fn push_front() {
        let mut list = DoublyLinkedList::new();
        for i in 0..5 {
            list.push_front(i).unwrap();
        }
        list.pop_front();
    }
    #[test]
    fn push_back() {
        let mut list = DoublyLinkedList::new();
        for i in 0..5 {
            list.push_back(i).unwrap();
        }
        list.pop_back();
    }
}
