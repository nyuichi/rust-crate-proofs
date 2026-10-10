//! Re-export the single conditional `bytes` 1.11.1 model supplied by
//! `creusot-std`'s optional `bytes-model` feature.

#[cfg(creusot)]
#[allow(unused_imports)]
pub(crate) use creusot_std::std::bytes::{bytes_mut_capacity, bytes_mut_seq, bytes_seq};
