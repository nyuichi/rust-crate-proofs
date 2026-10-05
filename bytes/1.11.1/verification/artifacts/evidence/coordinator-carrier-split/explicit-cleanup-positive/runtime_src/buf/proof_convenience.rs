//! Proof-only constructors for `Buf` and `BufMut` convenience adapters.
//!
//! These functions live outside the traits so Creusot does not have to
//! normalize recursive `Buf`/`BufMut` bounds in convenience-method signatures.

use super::{Buf, BufMut, Chain, Take};

/// Constructs a `Take` through its actual constructor.
///
/// The constructor itself specifies preservation of the stored fields. This
/// wrapper carries no adapter byte or advance contract.
#[inline]
pub(crate) fn take<T: Buf>(inner: T, limit: usize) -> Take<T> {
    super::take::new(inner, limit)
}

/// Constructs a `Chain` through its actual constructor.
///
/// The constructor itself specifies preservation of the stored fields. This
/// wrapper carries no concatenated-content or advance contract.
#[inline]
pub(crate) fn chain<T: Buf, U: Buf>(first: T, second: U) -> Chain<T, U> {
    Chain::new(first, second)
}

#[cfg(feature = "std")]
/// Constructs a reader through its actual constructor.
///
/// The constructor itself specifies preservation of its stored field. This
/// wrapper carries no `Read` transfer contract.
#[inline]
pub(crate) fn reader<T: Buf>(inner: T) -> super::Reader<T> {
    super::reader::new(inner)
}

#[cfg(feature = "std")]
/// Constructs a writer through its actual constructor.
///
/// The constructor itself specifies preservation of its stored field. This
/// wrapper carries no `Write` transfer contract.
#[inline]
pub(crate) fn writer<T: BufMut>(inner: T) -> super::Writer<T> {
    super::writer::new(inner)
}

/// Constructs a mutable `Chain` through its actual constructor.
///
/// The constructor itself specifies preservation of the stored fields. This
/// wrapper carries no write-behavior contract.
#[inline]
pub(crate) fn chain_mut<T: BufMut, U: BufMut>(first: T, second: U) -> Chain<T, U> {
    Chain::new(first, second)
}
