#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
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
    array_try_from_fn,
    slice_flatten,
    strict_provenance,
    type_alias_impl_trait
)]

#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate test;

//#![warn(unsafe_code)]

extern crate alloc;

pub mod prelude;

pub mod bitstring;
pub mod hash_table;
pub mod linked_lists;
pub mod queue;
pub mod traits;
mod util;

pub use crate::hash_table::SCHashTable;
pub use linked_lists::SinglyLinkedList;
pub use queue::ArrayQueue;
