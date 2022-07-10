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
    const_option,
    generic_const_exprs,
    negative_impls,
    int_roundings,
    ptr_metadata,
    array_from_fn,
    slice_flatten,
    strict_provenance,
    type_alias_impl_trait
)]

#[cfg(test)]
extern crate std;

//#![warn(unsafe_code)]

#[cfg(test)]
extern crate test;

extern crate alloc;

pub mod prelude;

pub mod hash_table;
pub mod linked_lists;
pub mod queue;
pub mod traits;
pub mod bitstring;
mod util;

pub use crate::hash_table::SCHashTable;
pub use linked_lists::SinglyLinkedList;
pub use queue::ArrayQueue;

