#![cfg_attr(not(test), no_std)]
#![feature(allocator_api, test)]

#[cfg(test)]
extern crate test;

extern crate alloc;

pub mod hash_table;