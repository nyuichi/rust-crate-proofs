use bytes::Bytes;

#[cfg(creusot)]
use creusot_std::logic::OrdLogic as _;
#[cfg(creusot)]
use creusot_std::prelude::DeepModel as _;
#[allow(unused_imports)]
use creusot_std::prelude::{
    check, ensures, logic, pearlite, proof_assert, requires, Invariant, Seq, View,
};
#[cfg(creusot)]
use creusot_std::std::string::{utf8_error_matches, valid_utf8, Utf8ErrorExt};
use std::{ops, str};

#[derive(Hash)]
pub(crate) struct ByteStr {
    // Invariant: bytes contains valid UTF-8
    bytes: Bytes,
}

impl std::fmt::Debug for ByteStr {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            formatter.deep_model(),
            (^formatter).deep_model()
        ))
    )]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ByteStr")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl View for ByteStr {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        crate::bytes_model::bytes_seq(self.bytes)
    }
}

impl Invariant for ByteStr {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { valid_utf8(self@) }
    }
}

impl creusot_std::prelude::DeepModel for ByteStr {
    type DeepModelTy = Seq<u8>;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

impl Clone for ByteStr {
    #[ensures(result@ == self@)]
    fn clone(&self) -> Self {
        ByteStr {
            bytes: self.bytes.clone(),
        }
    }
}

impl PartialEq for ByteStr {
    #[ensures(result == (self@ == other@))]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for ByteStr {}

impl PartialOrd for ByteStr {
    #[ensures(result == Some(self@.cmp_log(other@)))]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl Ord for ByteStr {
    #[ensures(result == self@.cmp_log(other@))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl ByteStr {
    #[inline]
    #[ensures(result@ == Seq::empty())]
    pub fn new() -> ByteStr {
        #[cfg(creusot)]
        proof_assert!(Seq::<char>::empty().to_bytes() == Seq::<u8>::empty());
        #[cfg(creusot)]
        proof_assert!(valid_utf8(Seq::<u8>::empty()));
        ByteStr {
            // Invariant: the empty slice is trivially valid UTF-8.
            bytes: Bytes::new(),
        }
    }

    #[inline]
    #[ensures(result@ == val@.to_bytes())]
    pub const fn from_static(val: &'static str) -> ByteStr {
        ByteStr {
            // Invariant: val is a str so contains valid UTF-8.
            bytes: Bytes::from_static(val.as_bytes()),
        }
    }

    #[inline]
    /// ## Panics
    /// In a debug build this will panic if `bytes` is not valid UTF-8.
    ///
    /// ## Safety
    /// `bytes` must contain valid UTF-8. In a release build it is undefined
    /// behavior to call this with `bytes` that is not valid UTF-8.
    #[requires(valid_utf8(crate::bytes_model::bytes_seq(bytes)))]
    #[ensures(result@ == crate::bytes_model::bytes_seq(bytes))]
    pub unsafe fn from_utf8_unchecked(bytes: Bytes) -> ByteStr {
        if cfg!(debug_assertions) {
            assert!(
                str::from_utf8(&bytes).is_ok(),
                "ByteStr::from_utf8_unchecked() with invalid bytes"
            );
        }
        // Invariant: assumed by the safety requirements of this function.
        ByteStr { bytes }
    }

    #[ensures(match result {
        Ok(value) => value.invariant() && value@ == crate::bytes_model::bytes_seq(bytes),
        Err(error) => utf8_error_matches(
            crate::bytes_model::bytes_seq(bytes),
            error.valid_up_to_logic(),
            error.error_len_logic(),
        ),
    })]
    pub(crate) fn from_utf8(bytes: Bytes) -> Result<ByteStr, std::str::Utf8Error> {
        match str::from_utf8(&bytes) {
            Ok(_) => Ok(ByteStr { bytes }),
            Err(err) => Err(err),
        }
    }
}

impl ops::Deref for ByteStr {
    type Target = str;

    #[inline]
    #[check(ghost)]
    #[requires(self.invariant())]
    #[ensures(result@.to_bytes() == self@)]
    fn deref(&self) -> &str {
        let b: &[u8] = self.bytes.as_ref();
        // Safety: the invariant of `bytes` is that it contains valid UTF-8.
        unsafe { str::from_utf8_unchecked(b) }
    }
}

impl From<String> for ByteStr {
    #[inline]
    #[ensures(result@ == src@.to_bytes())]
    fn from(src: String) -> ByteStr {
        ByteStr {
            // Invariant: src is a String so contains valid UTF-8.
            bytes: Bytes::from(src),
        }
    }
}

impl From<&str> for ByteStr {
    #[inline]
    #[ensures(result@ == src@.to_bytes())]
    fn from(src: &str) -> ByteStr {
        ByteStr {
            // Invariant: src is a str so contains valid UTF-8.
            bytes: Bytes::copy_from_slice(src.as_bytes()),
        }
    }
}

impl From<ByteStr> for Bytes {
    #[ensures(crate::bytes_model::bytes_seq(result) == src@)]
    fn from(src: ByteStr) -> Self {
        src.bytes
    }
}
