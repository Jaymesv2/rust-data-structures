#![cfg_attr(not(test), no_std)]
#![feature(
    test,
    variant_count,
    iter_intersperse,
    generic_associated_types,
    generators,
    allocator_api,
    box_into_inner,
    let_chains,
    const_option
)]

//#![warn(unsafe_code)]

#[cfg(test)]
extern crate test;

extern crate alloc;

pub mod prelude;

pub mod hash_table;
pub mod linked_lists;
pub mod queue;
pub mod traits;

pub use hash_table::SCHashTable;
pub use linked_lists::SinglyLinkedList;
pub use queue::ArrayQueue;
