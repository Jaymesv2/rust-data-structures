pub trait Iterable {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item> + 'a where Self: 'a;
    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}

pub trait IterableMut {
    type Item;
    type IterMut<'a>: Iterator<Item = &'a mut Self::Item> + 'a where Self: 'a;
    fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;
}

pub trait Drainable {
    type Item;
    type Drain<'a>: Iterator<Item = Self::Item> + 'a where Self: 'a;
    fn drain<'a>(&'a mut self) -> Self::Drain<'a>;
}