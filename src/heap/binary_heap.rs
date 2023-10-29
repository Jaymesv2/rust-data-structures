use alloc::vec::Vec;

use core::cmp::Ord;


use super::Heap;

//pub type MinBinaryHeap<T> = BinaryHeap<T, {HeapType::Min}>;
//pub type MaxBinaryHeap<T> = BinaryHeap<T, {HeapType::Max}>;

#[derive(Debug, Clone)]
pub struct BinaryHeap<T> {
    ombga: Vec<T>,
}

impl<T: Ord> BinaryHeap<T> {
    #[inline(always)]
    fn parent_idx(&self, i: usize) -> Option<usize> {
        i.checked_sub(1).map(|s| s.div_floor(2))
    }

    #[inline(always)]
    fn left_child_idx(&self,i: usize) -> Option<usize> {
        let idx = (2*i)+1;
        if idx >= self.ombga.len() {
            return None;
        }
        Some(idx)
    }

    #[inline(always)]
    fn right_child_idx(&self,i: usize) -> Option<usize> {
        let idx = (2*i)+2;
        if idx >= self.ombga.len() {
            return None;
        }
        Some(idx)
    }
    fn downheap(&mut self, mut idx: usize) {
        loop {
            let (child_idx, child) = match (self.left_child_idx(idx), self.right_child_idx(idx)) {
                (Some(left_idx), Some(right_idx)) => {
                    let left = self.ombga.get(left_idx).unwrap();                    
                    let right = self.ombga.get(right_idx).unwrap();  
                    if left < right {
                        (left_idx, left)
                    } else {
                        (right_idx, right)
                    }
                },
                (Some(left_idx), None) => (left_idx, self.ombga.get(left_idx).unwrap()),
                (None, Some(right_idx)) => (right_idx, self.ombga.get(right_idx).unwrap()),
                (None,None) => break,
            };

            let cur = self.ombga.get(idx).unwrap();
            if cur <= child {
                break;
            }
            self.ombga.swap(idx, child_idx);
            idx = child_idx;
        }
    }
    fn upheap(&mut self, mut idx: usize) {
        // bounds check
        if idx >= self.ombga.len() {
            return;
        }
        while let Some(parent_idx) = self.parent_idx(idx) {
            // these are safe since parent idx is None if there is no parent and
            // idx is the given by len and then pushed

            // memory safety, more like CRINGE
            let val = unsafe{ self.ombga.get(idx).unwrap_unchecked()};
            let parent = unsafe {self.ombga.get(parent_idx).unwrap_unchecked()};
            if parent <= val {
                break;   
            }
            self.ombga.swap(idx,parent_idx);
            idx = parent_idx;
        }

    }
}

impl<T: Ord> Heap<T> for BinaryHeap<T> {
    fn new() -> BinaryHeap<T>{
        Self {
            ombga: Vec::new(),
        }
    }
    fn extract(&mut self) -> Option<T> {
        if self.ombga.len() == 0 {
            return None;
        }
        let elem = self.ombga.swap_remove(0);
        self.downheap(0);
        Some(elem)
    }

    fn insert(&mut self, val: T) {
        let idx = self.ombga.len();
        self.ombga.push(val);
        self.upheap(idx);
    }
    fn is_empty(&self) -> bool {
        self.ombga.is_empty()
    }
    /* fn heapify(arr: Vec<T>) -> Self {
        // this can be more effecient
        arr.into_iter().fold(Self::new(), |mut h,i| {h.insert(i); h})
    }
    fn merge(&mut self, other: Self) {
        other.ombga.into_iter().for_each(|x| self.insert(x));
    } */
    /*fn meld(&self, other: &Self) -> Self {
        self.ombga.iter().chain(other.ombga.iter()).fold(Self::new(), |h, x| {h.insert(x.clone()); h})
    }
    fn pushpop(&mut self, val: T) -> T {
        let Some(root) = self.ombga.get(0) else {
            return val;
        };

        todo!()
    }*/
    fn peek(&self) -> Option<&T> {
        self.ombga.get(0)
    }
    fn len(&self) -> usize {
        self.ombga.len()
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_in_order() {
        let mut bheap: BinaryHeap<u32> = BinaryHeap::new();
        let max = 100;
        for i in (1..=max).rev() {
            bheap.insert(i);
            println!("{bheap:?}");
        }
        for i in 1..=max {
            let ex = bheap.extract().expect("failed to extract");
            println!("extracted: {ex:?}");
            assert_eq!(ex, i);
            println!("{bheap:?}");
        }
    }
    use rand::thread_rng;
    use rand::seq::SliceRandom;
    #[test]
    fn insert_rand_order() {
        let mut bheap: BinaryHeap<u32> = BinaryHeap::new();
        let max = 100;

        let mut elems: Vec<_> = (1..=max).collect();
        elems.shuffle(&mut thread_rng());

        for i in elems.into_iter() {
            bheap.insert(i);
            println!("{bheap:?}");
        }
        for i in 1..=max {
            let ex = bheap.extract().expect("failed to extract");
            println!("extracted: {ex:?}");
            assert_eq!(ex, i);
            println!("{bheap:?}");
        }
    }
}