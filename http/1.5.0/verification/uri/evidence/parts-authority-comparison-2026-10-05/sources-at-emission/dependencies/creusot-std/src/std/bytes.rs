//! Conditional content models for the pinned `bytes` 1.11.1 dependency.
//!
//! Enabling the `bytes-model` feature adds exact visible-byte observers and
//! external contracts for operations used by HTTP. These contracts are valid
//! only when the corresponding `bytes` implementation has been verified. They
//! describe contents, not reference counts, allocation validity, or ownership
//! transfer; those remain responsibilities of the dependency proof.

#[cfg(creusot)]
use crate::logic::OrdLogic;
#[cfg(creusot)]
use crate::std::partial_eq::PartialEqModel;
#[cfg(creusot)]
use crate::std::partial_ord::PartialOrdModel;
use crate::prelude::*;
use bytes::Bytes;
#[cfg(creusot)]
use bytes::BytesMut;

/// The initialized bytes visible through an immutable `Bytes` value.
#[logic(opaque)]
pub fn bytes_seq(_value: Bytes) -> Seq<u8> {
    dead
}

/// The initialized prefix visible through a mutable `BytesMut` value.
#[logic(opaque)]
pub fn bytes_mut_seq(_value: BytesMut) -> Seq<u8> {
    dead
}

/// Allocated capacity visible through a mutable `BytesMut` value.
#[logic(opaque)]
pub fn bytes_mut_capacity(_value: BytesMut) -> Int {
    dead
}

impl View for Bytes {
    type ViewTy = Seq<u8>;

    #[logic(open)]
    fn view(self) -> Self::ViewTy {
        bytes_seq(self)
    }
}

impl DeepModel for Bytes {
    type DeepModelTy = Seq<u8>;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        bytes_seq(self)
    }
}

extern_spec! {
    impl Bytes {
        #[ensures(bytes_seq(result) == Seq::empty())]
        fn new() -> Bytes;

        #[ensures(bytes_seq(result) == bytes@)]
        fn from_static(bytes: &'static [u8]) -> Bytes;

        #[ensures(bytes_seq(result) == data@)]
        fn copy_from_slice(data: &[u8]) -> Bytes;

        #[ensures(result@ == bytes_seq(*self).len())]
        fn len(&self) -> usize;

        #[ensures(result == (bytes_seq(*self).len() == 0))]
        fn is_empty(&self) -> bool;

        #[requires(at@ <= bytes_seq(*self).len())]
        #[ensures(bytes_seq(result) == bytes_seq(*self).subsequence(0, at@))]
        #[ensures(bytes_seq(^self) == bytes_seq(*self).subsequence(at@, bytes_seq(*self).len()))]
        fn split_to(&mut self, at: usize) -> Bytes;

        #[requires(at@ <= bytes_seq(*self).len())]
        #[ensures(bytes_seq(result) == bytes_seq(*self).subsequence(at@, bytes_seq(*self).len()))]
        #[ensures(bytes_seq(^self) == bytes_seq(*self).subsequence(0, at@))]
        fn split_off(&mut self, at: usize) -> Bytes;

        #[ensures(bytes_seq(^self) == bytes_seq(*self).subsequence(0, if len@ < bytes_seq(*self).len() { len@ } else { bytes_seq(*self).len() }))]
        fn truncate(&mut self, len: usize);

        #[ensures(bytes_seq(^self) == Seq::empty())]
        fn clear(&mut self);
    }

    impl AsRef<[u8]> for Bytes {
        #[check(ghost)]
        #[ensures(result@ == bytes_seq(*self))]
        fn as_ref(&self) -> &[u8];
    }

    impl core::ops::Deref for Bytes {
        #[check(ghost)]
        #[ensures(result@ == bytes_seq(*self))]
        fn deref(&self) -> &[u8];
    }

    impl Clone for Bytes {
        #[ensures(bytes_seq(result) == bytes_seq(*self))]
        fn clone(&self) -> Bytes;
    }

    impl From<Vec<u8>> for Bytes {
        #[ensures(bytes_seq(result) == bytes@)]
        fn from(bytes: Vec<u8>) -> Bytes;
    }

    impl From<String> for Bytes {
        #[ensures(bytes_seq(result) == bytes@.to_bytes())]
        fn from(bytes: String) -> Bytes;
    }

    impl From<&'static [u8]> for Bytes {
        #[ensures(bytes_seq(result) == bytes@)]
        fn from(bytes: &'static [u8]) -> Bytes;
    }

    impl From<&'static str> for Bytes {
        #[ensures(bytes_seq(result) == bytes@.to_bytes())]
        fn from(bytes: &'static str) -> Bytes;
    }

    impl PartialEq for Bytes {
        #[ensures(result == (bytes_seq(*self) == bytes_seq(*other)))]
        fn eq(&self, other: &Bytes) -> bool;
    }

    // These heterogeneous comparisons match bytes 1.11.1's implementations,
    // which compare `self.as_slice()` with the supplied byte slice. The
    // slice's deep model is `Seq<Int>`, so the std model relates each byte by
    // its numeric view rather than treating the sequence element types as
    // interchangeable.
    impl PartialEq<[u8]> for Bytes {
        #[ensures(result == bytes_seq(*self).eq_model(other.deep_model()))]
        fn eq(&self, other: &[u8]) -> bool;
    }

    impl PartialOrd<[u8]> for Bytes {
        #[ensures(result == Some(
            bytes_seq(*self).partial_cmp_model(other.deep_model())
        ))]
        fn partial_cmp(&self, other: &[u8]) -> Option<core::cmp::Ordering>;
    }

    impl PartialOrd for Bytes {
        #[ensures(result == Some(bytes_seq(*self).cmp_log(bytes_seq(*other))))]
        fn partial_cmp(&self, other: &Bytes) -> Option<core::cmp::Ordering>;
    }

    impl Ord for Bytes {
        #[ensures(result == bytes_seq(*self).cmp_log(bytes_seq(*other)))]
        fn cmp(&self, other: &Bytes) -> core::cmp::Ordering;
    }

    impl BytesMut {
        #[ensures(bytes_mut_seq(result) == Seq::empty())]
        fn new() -> BytesMut;

        #[requires(capacity@ <= isize::MAX@)]
        #[ensures(bytes_mut_seq(result) == Seq::empty())]
        #[ensures(bytes_mut_capacity(result) >= capacity@)]
        fn with_capacity(capacity: usize) -> BytesMut;

        #[ensures(result@ == bytes_mut_seq(*self).len())]
        fn len(&self) -> usize;

        #[ensures(result == (bytes_mut_seq(*self).len() == 0))]
        fn is_empty(&self) -> bool;

        #[ensures(result@ == bytes_mut_capacity(*self))]
        fn capacity(&self) -> usize;

        #[requires(additional@ <= isize::MAX@ - bytes_mut_seq(*self).len())]
        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self))]
        #[ensures(bytes_mut_capacity(^self) >= bytes_mut_seq(*self).len() + additional@)]
        fn reserve(&mut self, additional: usize);

        #[requires(bytes_mut_seq(*self).len() + src@.len() <= isize::MAX@)]
        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self).concat(src@))]
        fn extend_from_slice(&mut self, src: &[u8]);

        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self).subsequence(0, if len@ < bytes_mut_seq(*self).len() { len@ } else { bytes_mut_seq(*self).len() }))]
        fn truncate(&mut self, len: usize);

        #[ensures(bytes_mut_seq(^self) == Seq::empty())]
        fn clear(&mut self);

        #[ensures(bytes_seq(result) == bytes_mut_seq(self))]
        fn freeze(self) -> Bytes;
    }

    impl AsRef<[u8]> for BytesMut {
        #[check(ghost)]
        #[ensures(result@ == bytes_mut_seq(*self))]
        fn as_ref(&self) -> &[u8];
    }

    impl core::ops::Deref for BytesMut {
        #[check(ghost)]
        #[ensures(result@ == bytes_mut_seq(*self))]
        fn deref(&self) -> &[u8];
    }

    impl Clone for BytesMut {
        #[ensures(bytes_mut_seq(result) == bytes_mut_seq(*self))]
        fn clone(&self) -> BytesMut;
    }

    impl bytes::BufMut for BytesMut {
        #[requires(bytes_mut_seq(*self).len() + src@.len() <= isize::MAX@)]
        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self).concat(src@))]
        fn put_slice(&mut self, src: &[u8]);
    }

    impl core::fmt::Write for BytesMut {
        #[requires(bytes_mut_seq(*self).len() + value@.to_bytes().len() <= isize::MAX@)]
        #[ensures(result == Ok(()))]
        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self).concat(value@.to_bytes()))]
        fn write_str(&mut self, value: &str) -> core::fmt::Result;
    }

    impl From<Bytes> for BytesMut {
        #[ensures(bytes_mut_seq(result) == bytes_seq(bytes))]
        fn from(bytes: Bytes) -> BytesMut;
    }

    impl BytesMut {
        #[requires(at@ <= bytes_mut_seq(*self).len())]
        #[ensures(bytes_mut_seq(result) == bytes_mut_seq(*self).subsequence(0, at@))]
        #[ensures(bytes_mut_seq(^self) == bytes_mut_seq(*self).subsequence(at@, bytes_mut_seq(*self).len()))]
        fn split_to(&mut self, at: usize) -> BytesMut;

        #[requires(at@ <= bytes_mut_capacity(*self))]
        #[ensures(bytes_mut_seq(result) == if at@ <= bytes_mut_seq(*self).len() {
            bytes_mut_seq(*self).subsequence(at@, bytes_mut_seq(*self).len())
        } else {
            Seq::empty()
        })]
        #[ensures(bytes_mut_seq(^self) == if at@ <= bytes_mut_seq(*self).len() {
            bytes_mut_seq(*self).subsequence(0, at@)
        } else {
            bytes_mut_seq(*self)
        })]
        fn split_off(&mut self, at: usize) -> BytesMut;
    }

    impl<'a> From<&'a [u8]> for BytesMut {
        #[ensures(bytes_mut_seq(result) == bytes@)]
        fn from(bytes: &'a [u8]) -> BytesMut;
    }

    impl<'a> From<&'a str> for BytesMut {
        #[ensures(bytes_mut_seq(result) == bytes@.to_bytes())]
        fn from(bytes: &'a str) -> BytesMut;
    }
}

// `Bytes::hash` delegates to the `[u8]` Hash implementation. Under the
// invariant-preserving `Hasher` callback protocol in `std::hash`, it preserves
// the state invariant. This is conditional on the completed bytes 1.11.1
// dependency premise; it does not model digest values or Hash/Eq consistency.
extern_spec! {
    impl core::hash::Hash for Bytes {
        #[requires(inv(*state))]
        #[ensures(inv(^state))]
        fn hash<H: core::hash::Hasher>(&self, state: &mut H);
    }
}
