use bytes::{Bytes, BytesMut};

use std::convert::TryFrom;
use std::error::Error;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::{cmp, fmt, str};

#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::prelude::{
    DeepModel, Seq, View, ensures, ghost, invariant, logic, pearlite, proof_assert, requires,
    variant,
};
#[cfg(creusot)]
use creusot_std::logic::OrdLogic as _;
#[cfg(creusot)]
use creusot_std::std::partial_eq::PartialEqModel;
#[cfg(creusot)]
use creusot_std::std::partial_ord::PartialOrdModel;

#[cfg(not(http_header_value_leaf))]
use crate::header::name::HeaderName;

#[path = "value_validation.rs"]
mod value_validation;
use value_validation::{is_valid, is_visible_ascii};

/// Represents an HTTP header field value.
///
/// In practice, HTTP header field values are usually valid ASCII. However, the
/// HTTP spec allows for a header value to contain opaque bytes as well. In this
/// case, the header field value is not able to be represented as a string.
///
/// To handle this, the `HeaderValue` is usable as a type and can be compared
/// with strings and implements `Debug`. A `to_str` fn is provided that returns
/// an `Err` if the header value contains non visible ascii characters.
pub struct HeaderValue {
    inner: Bytes,
    is_sensitive: bool,
}

#[cfg(creusot)]
impl View for HeaderValue {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        crate::bytes_model::bytes_seq(self.inner)
    }
}

#[cfg(creusot)]
impl DeepModel for HeaderValue {
    type DeepModelTy = Seq<u8>;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

#[cfg(creusot)]
impl HeaderValue {
    /// Logical projection for the sensitivity bit, which is intentionally
    /// excluded from the content model used by equality and ordering.
    #[logic]
    pub fn sensitive_model(&self) -> bool {
        self.is_sensitive
    }
}

/// Whether every byte is permitted in a header value.
#[cfg(creusot)]
#[logic(open)]
pub fn valid_value_bytes(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i> 0 <= i && i < bytes.len()
            ==> bytes[i]@ >= 32 && bytes[i]@ != 127 || bytes[i]@ == 9
    }
}

/// Whether every byte can be represented by `HeaderValue::to_str`.
#[cfg(creusot)]
#[logic(open)]
pub fn visible_value_bytes(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i> 0 <= i && i < bytes.len()
            ==> bytes[i]@ >= 32 && bytes[i]@ < 127 || bytes[i]@ == 9
    }
}

/// Check that a byte slice satisfies the production header value classifier.
#[cfg_attr(creusot, ensures(result == valid_value_bytes(src@)))]
pub(crate) fn are_valid_value_bytes(src: &[u8]) -> bool {
    let mut i = 0;
    let mut valid = true;

    #[cfg_attr(creusot, invariant(i@ <= src@.len()))]
    #[cfg_attr(creusot, invariant(valid == valid_value_bytes(src@.subsequence(0, i@))))]
    #[cfg_attr(creusot, variant(src@.len() - i@))]
    while i < src.len() {
        valid &= is_valid(src[i]);
        i += 1;
    }

    valid
}

/// Check that a byte slice can be represented by `to_str`.
#[cfg_attr(creusot, ensures(result == visible_value_bytes(src@)))]
pub(crate) fn are_visible_value_bytes(src: &[u8]) -> bool {
    let mut i = 0;
    let mut visible = true;

    #[cfg_attr(creusot, invariant(i@ <= src@.len()))]
    #[cfg_attr(creusot, invariant(visible == visible_value_bytes(src@.subsequence(0, i@))))]
    #[cfg_attr(creusot, variant(src@.len() - i@))]
    while i < src.len() {
        visible &= is_visible_ascii(src[i]);
        i += 1;
    }

    visible
}

/// Return a one-character lowercase hexadecimal digit.
#[cfg(not(http_header_value_leaf))]
#[cfg_attr(creusot, requires(digit@ < 16))]
#[cfg_attr(creusot, ensures(result@.to_bytes().len() == 1))]
#[cfg_attr(creusot, ensures(
    result@.to_bytes().map(|byte: u8| byte@)
        == Seq::singleton(if digit@ < 10 { 48 + digit@ } else { 87 + digit@ })
))]
fn hex_digit(digit: u8) -> &'static str {
    let bytes: &'static [u8] = match digit {
        0 => &[48],
        1 => &[49],
        2 => &[50],
        3 => &[51],
        4 => &[52],
        5 => &[53],
        6 => &[54],
        7 => &[55],
        8 => &[56],
        9 => &[57],
        10 => &[97],
        11 => &[98],
        12 => &[99],
        13 => &[100],
        14 => &[101],
        15 => &[102],
        _ => unreachable!(),
    };

    #[cfg(creusot)]
    proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
        creusot_std::std::string::valid_utf8(bytes@)
    };

    // The match above selects one ASCII byte, so this is a valid one-byte str.
    unsafe { str::from_utf8_unchecked(bytes) }
}

/// A possible error when converting a `HeaderValue` from a string or byte
/// slice.
pub struct InvalidHeaderValue {
    _priv: (),
}

/// A possible error when converting a `HeaderValue` to a string representation.
///
/// Header field values may contain opaque bytes, in which case it is not
/// possible to represent the value as a string.
#[cfg_attr(not(http_header_value_leaf), derive(Debug))]
pub struct ToStrError {
    _priv: (),
}

impl HeaderValue {
    /// Convert a static string to a `HeaderValue`.
    ///
    /// This function will not perform any copying, however the string is
    /// checked to ensure that no invalid characters are present. Only visible
    /// ASCII characters (32-127) are permitted.
    ///
    /// # Panics
    ///
    /// This function panics if the argument contains invalid header value
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val, "hello");
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(visible_value_bytes(src@.to_bytes())))]
    #[cfg_attr(creusot, ensures(result@ == src@.to_bytes()))]
    #[cfg_attr(creusot, ensures(!result.sensitive_model()))]
    pub const fn from_static(src: &'static str) -> HeaderValue {
        let bytes = src.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if !is_visible_ascii(bytes[i]) {
                panic!("HeaderValue::from_static with invalid bytes")
            }
            i += 1;
        }

        HeaderValue {
            inner: Bytes::from_static(bytes),
            is_sensitive: false,
        }
    }

    /// Attempt to convert a string to a `HeaderValue`.
    ///
    /// If the argument contains invalid header value characters, an error is
    /// returned. Only visible ASCII characters (32-127) are permitted. Use
    /// `from_bytes` to create a `HeaderValue` that includes opaque octets
    /// (128-255).
    ///
    /// This function is intended to be replaced in the future by a `TryFrom`
    /// implementation once the trait is stabilized in std.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_str("hello").unwrap();
    /// assert_eq!(val, "hello");
    /// ```
    ///
    /// An invalid value
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_str("\n");
    /// assert!(val.is_err());
    /// ```
    #[inline]
    #[allow(clippy::should_implement_trait)]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == src@.to_bytes()
            && valid_value_bytes(src@.to_bytes())
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(src@.to_bytes()),
    }))]
    pub fn from_str(src: &str) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_bytes(src.as_bytes())
    }

    /// Converts a HeaderName into a HeaderValue
    ///
    /// Since every valid HeaderName is a valid HeaderValue this is done infallibly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::{HeaderValue, HeaderName};
    /// # use http::header::ACCEPT;
    /// let val = HeaderValue::from_name(ACCEPT);
    /// assert_eq!(val, HeaderValue::from_bytes(b"accept").unwrap());
    /// ```
    #[inline]
    #[cfg(not(http_header_value_leaf))]
    #[cfg_attr(creusot, ensures(result@ == name@ && !result.sensitive_model()))]
    pub fn from_name(name: HeaderName) -> HeaderValue {
        name.into()
    }

    /// Attempt to convert a byte slice to a `HeaderValue`.
    ///
    /// If the argument contains invalid header value bytes, an error is
    /// returned. Only byte values between 32 and 255 (inclusive) are permitted,
    /// excluding byte 127 (DEL).
    ///
    /// This function is intended to be replaced in the future by a `TryFrom`
    /// implementation once the trait is stabilized in std.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_bytes(b"hello\xfa").unwrap();
    /// assert_eq!(val, &b"hello\xfa"[..]);
    /// ```
    ///
    /// An invalid value
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_bytes(b"\n");
    /// assert!(val.is_err());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == src@ && valid_value_bytes(src@)
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(src@),
    }))]
    pub fn from_bytes(src: &[u8]) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::try_from_bytes(src)
    }

    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == src@ && valid_value_bytes(src@)
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(src@),
    }))]
    fn try_from_bytes(src: &[u8]) -> Result<HeaderValue, InvalidHeaderValue> {
        if !are_valid_value_bytes(src) {
            return Err(InvalidHeaderValue { _priv: () });
        }

        Ok(HeaderValue {
            inner: Bytes::copy_from_slice(src),
            is_sensitive: false,
        })
    }

    /// Attempt to convert a `Bytes` buffer to a `HeaderValue`.
    ///
    /// This will try to prevent a copy if the type passed is the type used
    /// internally, and will copy the data if it is not.
    // The proof leaf for concrete HeaderValue APIs deliberately excludes the
    // runtime Any/downcast path; all other value methods remain available.
    #[cfg(all(not(http_header_value_leaf), not(http_header_value_complete)))]
    pub fn from_maybe_shared<T>(src: T) -> Result<HeaderValue, InvalidHeaderValue>
    where
        T: AsRef<[u8]> + 'static,
    {
        if_downcast_into!(T, Bytes, src, {
            return HeaderValue::from_shared(src);
        });

        HeaderValue::from_bytes(src.as_ref())
    }

    /// Convert a `Bytes` directly into a `HeaderValue` without validating.
    ///
    /// This function does NOT validate that illegal bytes are not contained
    /// within the buffer.
    ///
    /// ## Panics
    /// In a debug build this will panic if `src` is not valid UTF-8.
    ///
    /// ## Safety
    /// `src` must contain valid UTF-8. In a release build it is undefined
    /// behaviour to call this with `src` that is not valid UTF-8.
    #[cfg(all(not(http_header_value_leaf), not(http_header_value_complete)))]
    pub unsafe fn from_maybe_shared_unchecked<T>(src: T) -> HeaderValue
    where
        T: AsRef<[u8]> + 'static,
    {
        if cfg!(debug_assertions) {
            match HeaderValue::from_maybe_shared(src) {
                Ok(val) => val,
                Err(_err) => {
                    panic!("HeaderValue::from_maybe_shared_unchecked() with invalid bytes");
                }
            }
        } else {
            if_downcast_into!(T, Bytes, src, {
                return HeaderValue {
                    inner: src,
                    is_sensitive: false,
                };
            });

            let src = Bytes::copy_from_slice(src.as_ref());
            HeaderValue {
                inner: src,
                is_sensitive: false,
            }
        }
    }

    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == crate::bytes_model::bytes_seq(src)
            && valid_value_bytes(crate::bytes_model::bytes_seq(src))
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(crate::bytes_model::bytes_seq(src)),
    }))]
    fn from_shared(src: Bytes) -> Result<HeaderValue, InvalidHeaderValue> {
        if !are_valid_value_bytes(src.as_ref()) {
            return Err(InvalidHeaderValue { _priv: () });
        }
        Ok(HeaderValue {
            inner: src,
            is_sensitive: false,
        })
    }

    /// Yields a `&str` slice if the `HeaderValue` only contains visible ASCII
    /// chars.
    ///
    /// This function will perform a scan of the header value, checking all the
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.to_str().unwrap(), "hello");
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => visible_value_bytes(self@) && value@.to_bytes() == self@,
        Err(_) => !visible_value_bytes(self@),
    }))]
    pub fn to_str(&self) -> Result<&str, ToStrError> {
        let bytes = self.as_ref();

        if !are_visible_value_bytes(bytes) {
            return Err(ToStrError { _priv: () });
        }

        #[cfg(creusot)]
        proof_assert! {
            crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
            creusot_std::std::string::valid_utf8(bytes@)
        };

        unsafe { Ok(str::from_utf8_unchecked(bytes)) }
    }

    /// Returns the length of `self`.
    ///
    /// This length is in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.len(), 5);
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == self@.len()))]
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Returns true if the `HeaderValue` has a length of zero bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("");
    /// assert!(val.is_empty());
    ///
    /// let val = HeaderValue::from_static("hello");
    /// assert!(!val.is_empty());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self@.len() == 0)))]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts a `HeaderValue` to a byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.as_bytes(), b"hello");
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == self@))]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Mark that the header value represents sensitive information.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let mut val = HeaderValue::from_static("my secret");
    ///
    /// val.set_sensitive(true);
    /// assert!(val.is_sensitive());
    ///
    /// val.set_sensitive(false);
    /// assert!(!val.is_sensitive());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures((^self)@ == self@))]
    #[cfg_attr(creusot, ensures((^self).sensitive_model() == val))]
    pub fn set_sensitive(&mut self, val: bool) {
        self.is_sensitive = val;
    }

    /// Returns `true` if the value represents sensitive data.
    ///
    /// Sensitive data could represent passwords or other data that should not
    /// be stored on disk or in memory. By marking header values as sensitive,
    /// components using this crate can be instructed to treat them with special
    /// care for security reasons. For example, caches can avoid storing
    /// sensitive values, and HPACK encoders used by HTTP/2.0 implementations
    /// can choose not to compress them.
    ///
    /// Additionally, sensitive values will be masked by the `Debug`
    /// implementation of `HeaderValue`.
    ///
    /// Note that sensitivity is not factored into equality or ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let mut val = HeaderValue::from_static("my secret");
    ///
    /// val.set_sensitive(true);
    /// assert!(val.is_sensitive());
    ///
    /// val.set_sensitive(false);
    /// assert!(!val.is_sensitive());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.sensitive_model()))]
    pub fn is_sensitive(&self) -> bool {
        self.is_sensitive
    }
}

impl Clone for HeaderValue {
    #[cfg_attr(creusot, ensures(result@ == self@))]
    #[cfg_attr(creusot, ensures(result.sensitive_model() == self.sensitive_model()))]
    fn clone(&self) -> Self {
        HeaderValue {
            inner: self.inner.clone(),
            is_sensitive: self.is_sensitive,
        }
    }
}

impl AsRef<[u8]> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

#[cfg(not(http_header_value_leaf))]
impl fmt::Debug for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_sensitive {
            f.write_str("Sensitive")
        } else {
            f.write_str("\"")?;
            let mut from = 0;
            let bytes = self.as_bytes();
            for (i, &b) in bytes.iter().enumerate() {
                if !is_visible_ascii(b) || b == b'"' {
                    if from != i {
                        f.write_str(unsafe { str::from_utf8_unchecked(&bytes[from..i]) })?;
                    }
                    if b == b'"' {
                        f.write_str("\\\"")?;
                    } else {
                        f.write_str("\\x")?;
                        let high = b / 16;
                        if high != 0 {
                            f.write_str(hex_digit(high))?;
                        }
                        f.write_str(hex_digit(b % 16))?;
                    }
                    from = i + 1;
                }
            }

            f.write_str(unsafe { str::from_utf8_unchecked(&bytes[from..]) })?;
            f.write_str("\"")
        }
    }
}

#[cfg(not(http_header_value_leaf))]
impl From<HeaderName> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == h@ && !result.sensitive_model()))]
    fn from(h: HeaderName) -> HeaderValue {
        HeaderValue {
            inner: h.into_bytes(),
            is_sensitive: false,
        }
    }
}

macro_rules! from_integers {
    ($($name:ident: $t:ident => $max_len:expr),*) => {$(
        impl From<$t> for HeaderValue {
            #[cfg_attr(creusot, ensures(
                result@.map(|byte: u8| byte@) == ::itoa::integer_decimal_values(num)
            ))]
            #[cfg_attr(creusot, ensures(!result.sensitive_model()))]
            fn from(num: $t) -> HeaderValue {
                let mut buf = BytesMut::with_capacity($max_len);
                let _ = buf.write_str(::itoa::Buffer::new().format(num));
                HeaderValue {
                    inner: buf.freeze(),
                    is_sensitive: false,
                }
            }
        }

        #[test]
        fn $name() {
            let n: $t = 55;
            let val = HeaderValue::from(n);
            assert_eq!(val, &n.to_string());

            let n = ::std::$t::MAX;
            let val = HeaderValue::from(n);
            assert_eq!(val, &n.to_string());
        }
    )*};
}

#[cfg(not(http_header_value_leaf))]
from_integers! {
    // integer type => maximum decimal length

    // u8 purposely left off... HeaderValue::from(b'3') could be confusing
    from_u16: u16 => 5,
    from_i16: i16 => 6,
    from_u32: u32 => 10,
    from_i32: i32 => 11,
    from_u64: u64 => 20,
    from_i64: i64 => 20
}

#[cfg(all(not(http_header_value_leaf), target_pointer_width = "16"))]
from_integers! {
    from_usize: usize => 5,
    from_isize: isize => 6
}

#[cfg(all(not(http_header_value_leaf), target_pointer_width = "32"))]
from_integers! {
    from_usize: usize => 10,
    from_isize: isize => 11
}

#[cfg(all(not(http_header_value_leaf), target_pointer_width = "64"))]
from_integers! {
    from_usize: usize => 20,
    from_isize: isize => 20
}

// The focused verification crate omits HeaderMap, while the full crate keeps
// this conversion integration test enabled.
#[cfg(all(test, not(http_header_value_complete)))]
mod from_header_name_tests {
    use super::*;
    use crate::header::map::HeaderMap;
    use crate::header::name;

    #[test]
    fn it_can_insert_header_name_as_header_value() {
        let mut map = HeaderMap::new();
        map.insert(name::UPGRADE, name::SEC_WEBSOCKET_PROTOCOL.into());
        map.insert(
            name::ACCEPT,
            name::HeaderName::from_bytes(b"hello-world").unwrap().into(),
        );

        assert_eq!(
            map.get(name::UPGRADE).unwrap(),
            HeaderValue::from_bytes(b"sec-websocket-protocol").unwrap()
        );

        assert_eq!(
            map.get(name::ACCEPT).unwrap(),
            HeaderValue::from_bytes(b"hello-world").unwrap()
        );
    }
}

impl FromStr for HeaderValue {
    type Err = InvalidHeaderValue;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == s@.to_bytes()
            && valid_value_bytes(s@.to_bytes())
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(s@.to_bytes()),
    }))]
    fn from_str(s: &str) -> Result<HeaderValue, Self::Err> {
        HeaderValue::from_str(s)
    }
}

impl From<&HeaderValue> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == t@))]
    #[cfg_attr(creusot, ensures(result.sensitive_model() == t.sensitive_model()))]
    fn from(t: &HeaderValue) -> Self {
        t.clone()
    }
}

impl TryFrom<&str> for HeaderValue {
    type Error = InvalidHeaderValue;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == t@.to_bytes()
            && valid_value_bytes(t@.to_bytes())
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(t@.to_bytes()),
    }))]
    fn try_from(t: &str) -> Result<Self, Self::Error> {
        HeaderValue::from_str(t)
    }
}

impl TryFrom<&String> for HeaderValue {
    type Error = InvalidHeaderValue;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == s@.to_bytes()
            && valid_value_bytes(s@.to_bytes())
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(s@.to_bytes()),
    }))]
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl TryFrom<&[u8]> for HeaderValue {
    type Error = InvalidHeaderValue;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == t@ && valid_value_bytes(t@)
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(t@),
    }))]
    fn try_from(t: &[u8]) -> Result<Self, Self::Error> {
        HeaderValue::from_bytes(t)
    }
}

impl TryFrom<String> for HeaderValue {
    type Error = InvalidHeaderValue;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == t@.to_bytes()
            && valid_value_bytes(t@.to_bytes())
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(t@.to_bytes()),
    }))]
    fn try_from(t: String) -> Result<Self, Self::Error> {
        HeaderValue::from_shared(t.into())
    }
}

impl TryFrom<Vec<u8>> for HeaderValue {
    type Error = InvalidHeaderValue;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => value@ == vec@ && valid_value_bytes(vec@)
            && !value.sensitive_model(),
        Err(_) => !valid_value_bytes(vec@),
    }))]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        HeaderValue::from_shared(vec.into())
    }
}

#[cfg(test)]
mod try_from_header_name_tests {
    use super::*;
    use crate::header::name;

    #[test]
    fn it_converts_using_try_from() {
        assert_eq!(
            HeaderValue::try_from(name::UPGRADE).unwrap(),
            HeaderValue::from_bytes(b"upgrade").unwrap()
        );
    }
}

#[cfg(not(http_header_value_leaf))]
impl fmt::Debug for InvalidHeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidHeaderValue")
            // skip _priv noise
            .finish()
    }
}

impl fmt::Display for InvalidHeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to parse header value")
    }
}

#[cfg(not(http_header_value_leaf))]
impl Error for InvalidHeaderValue {}

impl fmt::Display for ToStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to convert header to a str")
    }
}

#[cfg(not(http_header_value_leaf))]
impl Error for ToStrError {}

// ===== PartialEq / PartialOrd =====

#[cfg(not(http_header_value_leaf))]
impl Hash for HeaderValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl PartialEq for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self@ == other@)))]
    fn eq(&self, other: &HeaderValue) -> bool {
        self.inner == other.inner
    }
}

impl Eq for HeaderValue {}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(self@.cmp_log(other@))))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(not(http_header_value_leaf))]
impl Ord for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self@.cmp_log(other@)))]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<str> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &str) -> bool {
        let other_bytes = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                self.deep_model(),
                other_bytes@,
                other_bytes.deep_model(),
            )
        };
        self.inner == other_bytes
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<[u8]> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &[u8]) -> bool {
        self.inner == other
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<str> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        let left = &*self.inner;
        let right = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                self.deep_model(),
                left.deep_model(),
                right@,
                right.deep_model(),
            )
        };
        left.partial_cmp(right)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<[u8]> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        let left = &*self.inner;
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_left_transport_preserved(
                self.deep_model(),
                left.deep_model(),
                other.deep_model(),
            )
        };
        left.partial_cmp(other)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<HeaderValue> for str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<HeaderValue> for [u8] {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<HeaderValue> for str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = &*other.inner;
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                left@,
                left.deep_model(),
                other.deep_model(),
                right.deep_model(),
            )
        };
        left.partial_cmp(right)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<HeaderValue> for [u8] {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        let right = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_reverse_transport_preserved(
                right@,
                right.deep_model(),
                self.deep_model(),
            )
        };
        self.partial_cmp(right)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<String> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &String) -> bool {
        let other_bytes = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                self.deep_model(),
                other_bytes@,
                other_bytes.deep_model(),
            )
        };
        self.inner == other_bytes
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<String> for HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        let left = &*self.inner;
        let right = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                self.deep_model(),
                left.deep_model(),
                right@,
                right.deep_model(),
            )
        };
        left.partial_cmp(right)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<HeaderValue> for String {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<HeaderValue> for String {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                left@,
                left.deep_model(),
                right@,
                right.deep_model(),
            )
        };
        left.partial_cmp(right)
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialEq<HeaderValue> for &HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &HeaderValue) -> bool {
        **self == *other
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<HeaderValue> for &HeaderValue {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

macro_rules! impl_partial_eq_ref {
    ($($model_bounds:tt)*) => {
        impl<T: ?Sized> PartialEq<&T> for HeaderValue
        where
            HeaderValue: PartialEq<T>,
            $($model_bounds)*
        {
            #[inline]
            #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
            fn eq(&self, other: &&T) -> bool {
                *self == **other
            }
        }
    };
}

// Creusot needs a deep model for the RHS to state the heterogeneous equality
// contract. The ordinary Rust build retains the original unrestricted
// forwarding implementation; both variants expand to the same method body.
#[cfg(not(creusot))]
#[cfg(not(http_header_value_leaf))]
impl_partial_eq_ref!();

#[cfg(creusot)]
#[cfg(not(http_header_value_leaf))]
impl_partial_eq_ref!(T: DeepModel, Seq<u8>: PartialEqModel<T::DeepModelTy>,);

macro_rules! impl_partial_ord_ref {
    ($($model_bounds:tt)*) => {
        impl<T: ?Sized> PartialOrd<&T> for HeaderValue
        where
            HeaderValue: PartialOrd<T>,
            $($model_bounds)*
        {
            #[inline]
            #[cfg_attr(creusot, ensures(result == Some(
                self.deep_model().partial_cmp_model(other.deep_model())
            )))]
            fn partial_cmp(&self, other: &&T) -> Option<cmp::Ordering> {
                self.partial_cmp(*other)
            }
        }
    };
}

#[cfg(not(creusot))]
#[cfg(not(http_header_value_leaf))]
impl_partial_ord_ref!();

#[cfg(creusot)]
#[cfg(not(http_header_value_leaf))]
impl_partial_ord_ref!(
    T: DeepModel,
    Seq<u8>: PartialEqModel<T::DeepModelTy>,
    Seq<u8>: PartialOrdModel<T::DeepModelTy>,
);

#[cfg(not(http_header_value_leaf))]
impl PartialEq<HeaderValue> for &str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.deep_model().eq_model(other.deep_model())))]
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

#[cfg(not(http_header_value_leaf))]
impl PartialOrd<HeaderValue> for &str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = other.as_bytes();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                left@,
                left.deep_model(),
                right@,
                right.deep_model(),
            )
        };
        left.partial_cmp(right)
    }
}

#[test]
fn test_try_from() {
    HeaderValue::try_from(vec![127]).unwrap_err();
}

#[test]
fn test_debug() {
    let cases = &[
        ("hello", "\"hello\""),
        ("hello \"world\"", "\"hello \\\"world\\\"\""),
        ("\u{7FFF}hello", "\"\\xe7\\xbf\\xbfhello\""),
    ];

    for &(value, expected) in cases {
        let val = HeaderValue::from_bytes(value.as_bytes()).unwrap();
        let actual = format!("{:?}", val);
        assert_eq!(expected, actual);
    }

    let mut sensitive = HeaderValue::from_static("password");
    sensitive.set_sensitive(true);
    assert_eq!("Sensitive", format!("{:?}", sensitive));

    let escaped = HeaderValue::from_bytes(&[b'\t', 128, 255]).unwrap();
    let expected = "\"\t\\x80\\xff\"";
    assert_eq!(format!("{:?}", escaped), expected);
    assert_eq!(format!("{:>24?}", escaped), expected);
    assert_eq!(format!("{:.1?}", escaped), expected);
    assert_eq!(format!("{:#?}", escaped), expected);

    for byte in 0..=u8::MAX {
        assert_eq!(
            format!("{:?}", LegacyEscapedByte(byte)),
            format!("{:?}", ManualEscapedByte(byte))
        );
        assert_eq!(
            format!("{:>24?}", LegacyEscapedByte(byte)),
            format!("{:>24?}", ManualEscapedByte(byte))
        );
        assert_eq!(
            format!("{:.1?}", LegacyEscapedByte(byte)),
            format!("{:.1?}", ManualEscapedByte(byte))
        );
        assert_eq!(
            format!("{:#?}", LegacyEscapedByte(byte)),
            format!("{:#?}", ManualEscapedByte(byte))
        );
    }
}

#[cfg(test)]
struct LegacyEscapedByte(u8);

#[cfg(test)]
impl fmt::Debug for LegacyEscapedByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\x{:x}", self.0)
    }
}

#[cfg(test)]
struct ManualEscapedByte(u8);

#[cfg(test)]
impl fmt::Debug for ManualEscapedByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\\x")?;
        let high = self.0 / 16;
        if high != 0 {
            f.write_str(hex_digit(high))?;
        }
        f.write_str(hex_digit(self.0 % 16))
    }
}
