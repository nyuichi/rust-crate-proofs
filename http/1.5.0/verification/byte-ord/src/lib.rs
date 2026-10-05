#![allow(dead_code, unexpected_cfgs)]

#[cfg(creusot)]
pub use creusot_std::prelude as prelude;

#[cfg(creusot)]
#[path = "../../../../../creusot-libs/creusot-std/src/std/seq_ord.rs"]
mod seq_ord;
