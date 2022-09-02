//#![recursion_limit = "1000"]

/// Abstract Data Type Traits
trait Creatable {
    fn new() -> Self;
}

trait List<T> {
    fn new() -> Self;
    fn is_empty(&self) -> bool;
    fn append(&mut self, item: T);
    fn prepend(&mut self, item: T);
    fn head(&self) -> Option<&T>;
    fn tail(&mut self); // simply drops the first element
    fn get(&self, index: usize);
}

trait PriorityQueue<T, P: Ord> {
    fn is_empty(&self) -> bool;
    fn insert_with_priority(&mut self, item: T, priority: P);
    fn pull_highest_priority_element(&mut self) -> (T, P);
}

/*
trait Tree<T> {
    fn value(&self)
    children
    nil
    node
}
*/

trait HeapKind {}
struct MinHeap;
struct MaxHeap;
impl HeapKind for MinHeap {}
impl HeapKind for MaxHeap {}

trait Heap<T, K>
where
    Self: Sized,
    K: HeapKind,
{
    //basic:
    fn find_max(&self) -> T;
    fn insert(&mut self, item: T);
    fn extract_max(&mut self) -> Option<T>;
    fn delete_max(&mut self);
    fn replace(&mut self);
    //creation:
    fn new() -> Self;
    //fn heapify() impl from iter
    /*
    fn merge(&self, other: &Self) -> Self where Self: Clone + IntoIterator<Item = T> + FromIterator<T>, T: Clone {
        Self::from_iter(self.clone().into_iter().chain(other.clone().into_iter()))
    }*/
    fn merge(&self, other: &Self) -> Self
    where
        Self: Clone,
        T: Clone,
    {
        let mut new = Self::new();
        new.meld(self.clone());
        new.meld(other.clone());
        new
    }
    fn meld(&mut self, mut other: Self) {
        while let Some(s) = other.extract_max() {
            self.insert(s);
        }
    }
    //inspection:
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
    //internal:
    //fn increase_key()
    /*
    increase-key or decrease-key: updating a key within a max- or min-heap, respectively
    delete: delete an arbitrary node (followed by moving last node and sifting to maintain heap)
    sift-up: move a node up in the tree, as long as needed; used to restore heap condition after insertion. Called "sift" because node moves up the tree until it reaches the correct level, as in a sieve.
    sift-down: move a node down in the tree, similar to sift-up; used to restore heap condition after deletion or replacement.

    */
}

trait Set<T> {
    fn union(s: Self, t: Self);
    fn intersection(s: Self, t: Self);
    fn difference(s: Self, t: Self);
    fn subset(s: Self, t: Self);
}

enum SetSize {
    Finite(usize),
    Infinite,
}

trait StaticSet<T>: Set<T> {
    fn is_element_of(&self, elem: T) -> bool;
    fn is_empty(&self) -> bool;
    fn size(&self) -> SetSize;
    //fn iter(&self) ->
    //fn enumerate(&self) ->
    //fn build(item: Vec<T>) -> Self;
    //fn create_from() -> Self; impl FromIter
}

trait DynamicSet<T>: Set<T> {
    fn create() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn add(&mut self, elem: T);
    fn remove(&mut self, elem: T);
    fn capacity(&self) -> usize;
}

trait MultiSetKind {}

trait MultiSet<T, K>: Set<T>
where
    K: MultiSetKind,
{
}

/*
use std::ops::Add;
use num::Zero;

fn head<T>(arr: &[T]) -> (Option<&T>, &[T]) {
    (arr.get(0), &arr[1..])
}

fn recur_sum<T: Add + Zero + Copy>(arr: &[T]) -> T {
    match &arr[..] {
        [x,xs@..] => *x + recur_sum(xs),
        []  => T::zero(),
    }
}

fn recur_mul(x: i32, y: u32) -> i32 {
    if y == 0 {
        0
    } else {
        x + recur_mul(x,y-1)
    }
}


fn is_member<T: Eq>(arr: &[T], val: &T) -> bool {
    match &arr[..] {
        [x,xs@..] => x == val || is_member(xs, val),
        []  => false,
    }
}
*/
