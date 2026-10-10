//! Specifications for the `std` crate
mod array;
mod borrow;
pub use borrow::BorrowModel;
mod boxed;
pub(crate) mod hash_word;
#[cfg(feature = "bytes-model")]
pub mod bytes;
pub mod cell;
pub mod char;
pub mod clone;
pub mod cmp;
pub mod convert;
pub mod cow;
pub mod default;
pub mod fmt;
#[cfg(all(feature = "bytes-model", creusot))]
mod hash;
pub mod hint;
pub mod intrinsics;
pub mod iter;
pub mod mem;
pub mod num;
pub mod ops;
pub mod option;
#[cfg(creusot)]
pub mod partial_ord;
pub mod panicking;
pub mod partial_eq;
pub mod ptr;
pub mod range;
pub mod rc;
pub mod result;
#[cfg(creusot)]
#[doc(hidden)]
pub mod seq_ord;
#[cfg(creusot)]
mod seq_ord_impl;
pub mod slice;
pub mod string;
#[cfg(creusot)]
#[doc(hidden)]
pub mod str_index;
pub mod time;
mod tuples;
pub mod unsafe_collection;
pub mod vec;

// Every std-dependent part of the Creusot Standard Library must be disabled when
// compiling with [no_std].

#[cfg(feature = "std")]
pub mod collections {
    pub mod hash_map;
    pub mod hash_set;
}

#[cfg(feature = "std")]
pub mod deque;

#[cfg(feature = "std")]
pub mod io;

#[cfg(feature = "std")]
pub mod sync;

#[cfg(feature = "std")]
pub mod thread;
