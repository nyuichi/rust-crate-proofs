//! Comparison translation probes.
#![allow(unexpected_cfgs)]

#[cfg(all(feature = "actual-runtime", creusot))]
mod actual_runtime {
    use bytes::{Bytes, BytesMut};
    use core::cmp::Ordering;

    /// Exercise the proof-only `Bytes` view equality adapter on actual handles.
    pub fn bytes_equal(left: &Bytes, right: &Bytes) -> bool {
        left.__creusot_eq_bytes(right)
    }

    /// Exercise the proof-only `Bytes` view ordering adapter on actual handles.
    pub fn bytes_order(left: &Bytes, right: &Bytes) -> Ordering {
        left.__creusot_cmp_bytes(right)
    }

    /// Exercise byte-slice equality over an actual `Bytes` receiver.
    pub fn bytes_slice_equal(left: &Bytes, right: &[u8]) -> bool {
        left.__creusot_eq_slice(right)
    }

    /// Exercise byte-slice ordering over an actual `Bytes` receiver.
    pub fn bytes_slice_order(left: &Bytes, right: &[u8]) -> Ordering {
        left.__creusot_cmp_slice(right)
    }

    /// Exercise UTF-8-byte equality over an actual `Bytes` receiver.
    pub fn bytes_text_equal(left: &Bytes, right: &str) -> bool {
        left.__creusot_eq_str(right)
    }

    /// Exercise UTF-8-byte ordering over an actual `Bytes` receiver.
    pub fn bytes_text_order(left: &Bytes, right: &str) -> Ordering {
        left.__creusot_cmp_str(right)
    }

    /// Exercise `BytesMut` view equality and ordering.
    pub fn bytes_mut_equal(left: &BytesMut, right: &BytesMut) -> bool {
        left.__creusot_eq_bytes_mut(right)
    }

    /// Exercise `BytesMut` view ordering.
    pub fn bytes_mut_order(left: &BytesMut, right: &BytesMut) -> Ordering {
        left.__creusot_cmp_bytes_mut(right)
    }

    /// Exercise byte-slice equality over an actual `BytesMut` receiver.
    pub fn bytes_mut_slice_equal(left: &BytesMut, right: &[u8]) -> bool {
        left.__creusot_eq_slice(right)
    }

    /// Exercise byte-slice ordering over an actual `BytesMut` receiver.
    pub fn bytes_mut_slice_order(left: &BytesMut, right: &[u8]) -> Ordering {
        left.__creusot_cmp_slice(right)
    }

    /// Exercise UTF-8-byte equality over an actual `BytesMut` receiver.
    pub fn bytes_mut_text_equal(left: &BytesMut, right: &str) -> bool {
        left.__creusot_eq_str(right)
    }

    /// Exercise UTF-8-byte ordering over an actual `BytesMut` receiver.
    pub fn bytes_mut_text_order(left: &BytesMut, right: &str) -> Ordering {
        left.__creusot_cmp_str(right)
    }

    /// Exercise equality across actual `Bytes` and `BytesMut` handles.
    pub fn bytes_mut_bytes_equal(left: &BytesMut, right: &Bytes) -> bool {
        left.__creusot_eq_bytes(right)
    }

    /// Exercise equality across actual `BytesMut` and `Bytes` handles.
    pub fn bytes_bytes_mut_equal(left: &Bytes, right: &BytesMut) -> bool {
        left.__creusot_eq_bytes_mut(right)
    }
}

/// A small compiler diagnostic for the generic reference-impl pattern from
/// `Bytes`. This feature has no dependency on the `bytes` runtime crate and is
/// not runtime verification coverage.
#[cfg(all(feature = "generic-pattern-diagnostic", creusot))]
mod generic_pattern_diagnostic {
    use core::cmp::{Ordering, PartialEq, PartialOrd};
    use creusot_std::prelude::*;

    pub struct Tiny(pub u8);

    impl PartialEq for Tiny {
        fn eq(&self, other: &Tiny) -> bool {
            self.0 == other.0
        }
    }

    impl PartialOrd for Tiny {
        fn partial_cmp(&self, other: &Tiny) -> Option<Ordering> {
            self.0.partial_cmp(&other.0)
        }
    }

    impl<'a, T: ?Sized> PartialEq<&'a T> for Tiny
    where
        Tiny: PartialEq<T>,
    {
        fn eq(&self, other: &&'a T) -> bool {
            *self == **other
        }
    }

    impl<'a, T: ?Sized> PartialOrd<&'a T> for Tiny
    where
        Tiny: PartialOrd<T>,
    {
        fn partial_cmp(&self, other: &&'a T) -> Option<Ordering> {
            self.partial_cmp(&**other)
        }
    }

    pub fn compare_generic_reference(left: &Tiny, right: &&Tiny) -> bool {
        left == right
    }

    pub fn order_generic_reference(left: &Tiny, right: &&Tiny) -> Option<Ordering> {
        left.partial_cmp(right)
    }
}
