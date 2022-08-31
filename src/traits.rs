pub mod adt;
pub mod fp;
pub mod hash_table;
pub mod iter;

// this is just playing with some traits
use core::iter::IntoIterator;
use iter::*;

trait Collection<T>:
    Iterable<Item = T> + FromIterator<T> + IntoIterator<Item = T> + IterableMut<Item = T>
{
}
