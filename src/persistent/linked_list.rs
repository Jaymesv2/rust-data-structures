/*
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};

pub struct SinglyLinkedList<T: Clone> {
    head: Arc<Option<SinglyLinkedListNode<T>>>,
}

impl<T: Clone> SinglyLinkedList<T> {
    pub fn new() -> SinglyLinkedList<T> {
        SinglyLinkedList { head: Arc::new(None) }
    }
    pub fn push(&mut self, val: T) {
        let mut n = SinglyLinkedListNode::new(val);
        n.next = self.head.clone();
        *Arc::make_mut(&mut self.head) = Some(n);
    }
}

impl<T: Clone> Debug for SinglyLinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SinglyLinkedList {{")?;

        write!(f, "}}")
    }
}

pub struct SinglyLinkedListIterator<T: Clone> {
    item: Arc<Option<SinglyLinkedListNode<T>>>
}

impl<T: Clone> Iterator for SinglyLinkedListIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.item.as_ref() {
            let b = s.val.clone();
            self.item = s.next.clone();
            Some(b)
        } else {
            None
        }
    }
}

use alloc::boxed::Box;

#[derive(Clone)]
struct SinglyLinkedListNode<T> {
    pub val: Arc<T>,
    pub next: Option<Box<SinglyLinkedListNode<T>>>
}

impl<T> SinglyLinkedListNode<T> {
    pub fn new(val: T) -> SinglyLinkedListNode<T> {
        SinglyLinkedListNode { val: Arc::new(val), next: None }
    }
}

 */
