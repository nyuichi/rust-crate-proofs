#![feature(hasher_prefixfree_extras)]
pub fn prefix<H:core::hash::Hasher>(h:&mut H,len:usize) { h.write_length_prefix(len) }
