#![cfg_attr(not(test), no_std)]
#![feature(allocator_api, test, box_into_inner)]

extern crate alloc;
#[cfg(test)]
extern crate test;

pub mod singly_linked_list;
