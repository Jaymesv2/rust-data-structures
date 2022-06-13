use crate::HashTableImpl;
use std::hash::{Hash, BuildHasher};

struct HashTableTester<I>
where
    I: HashTableImpl,
{
    table: I,
}

impl<'a, I> HashTableTester<I> 
where
    I: HashTableImpl,
{
    fn new(table: I) -> Self {
        Self { 
            table,
        }
    }
}