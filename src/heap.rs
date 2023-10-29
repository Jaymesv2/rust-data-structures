use core::cmp::{PartialEq, Eq, Ord, Ordering::*};

pub mod binary_heap;


trait Heap<T> {
    fn new() -> Self;
    //fn heapify(arr: Vec<T>) -> Self;
    //fn merge(&mut self, other: Self);
    fn peek(&self) -> Option<&T>;
    fn insert(&mut self, val: T);
    fn extract(&mut self) -> Option<T>;
    //fn meld(&self, other: &Self) -> Self;
    //fn pushpop(&mut self, val: T) -> Option<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/*
trait MergableHeap<T>: Heap<T>{

}

trait MeldableHeap<T>: Heap<T> {

}
*/


















use core::ops::{Deref, DerefMut};

// Inverts the Ord implementation on T
pub struct InverseOrd<T>(T);

impl<T> Deref for InverseOrd<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for InverseOrd<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl<T: PartialOrd> PartialOrd for InverseOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0).map(|s| {
            match s {
                Less => Greater,
                Equal => Equal,
                Greater => Less,
            }
        })
    }
}

impl<T: Ord> Ord for InverseOrd<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.0.cmp(&other.0) {
            Less => Greater,
            Equal => Equal,
            Greater => Less,
        }
    }   
}

impl<T: Eq> Eq for InverseOrd<T> {}

impl<T: PartialEq> PartialEq for InverseOrd<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

