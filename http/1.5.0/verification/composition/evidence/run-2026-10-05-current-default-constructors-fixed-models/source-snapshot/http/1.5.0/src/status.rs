//! HTTP status codes
//!
//! This module contains HTTP-status code related structs and errors. The main
//! type in this module is `StatusCode` which is not intended to be used through
//! this module but rather the `http::StatusCode` type.
//!
//! # Examples
//!
//! ```
//! use http::StatusCode;
//!
//! assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
//! assert_eq!(StatusCode::NOT_FOUND, 404);
//! assert!(StatusCode::OK.is_success());
//! ```

use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::num::NonZeroU16;
use std::str::FromStr;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, extern_spec, invariant, logic, pearlite, proof_assert, requires, DeepModel, Int,
    Invariant, OrdLogic, Seq, snapshot, variant, View,
};

/// Whether `bytes` is a three-byte decimal status code in the HTTP range.
#[logic(open)]
pub fn valid_status_bytes(bytes: creusot_std::prelude::Seq<u8>) -> bool {
    pearlite! {
        bytes.len() == 3
            && 49 <= bytes[0]@ && bytes[0]@ <= 57
            && 48 <= bytes[1]@ && bytes[1]@ <= 57
            && 48 <= bytes[2]@ && bytes[2]@ <= 57
    }
}

/// Decimal value represented by a three-byte status code, or zero for a non-three-byte input.
#[logic(open)]
pub fn status_bytes_value(bytes: creusot_std::prelude::Seq<u8>) -> Int {
    pearlite! {
        if bytes.len() == 3 {
            (bytes[0]@ - 48) * 100 + (bytes[1]@ - 48) * 10 + bytes[2]@ - 48
        } else {
            0
        }
    }
}

/// ASCII byte for one decimal digit of a status code.
#[logic(open)]
pub fn status_digit_byte(code: Int, column: Int) -> Int {
    pearlite! {
        if column == 0 {
            48 + code / 100
        } else if column == 1 {
            48 + (code / 10) % 10
        } else {
            48 + code % 10
        }
    }
}

/// Generate the packed static digit table from its decimal values.
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len() ==>
    result@[i]@ == status_digit_byte(100 + i / 3, i % 3)
))]
const fn build_code_digits() -> [u8; 2700] {
    let mut digits = [0u8; 2700];
    let mut code = 100u16;

    #[cfg_attr(creusot, invariant(100 <= code@ && code@ <= 1000))]
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < (code@ - 100) * 3 ==>
        digits@[i]@ == status_digit_byte(100 + i / 3, i % 3)
    ))]
    #[cfg_attr(creusot, variant(1000 - code@))]
    while code < 1000 {
        let row = ((code - 100) * 3) as usize;
        digits[row] = 48 + (code / 100) as u8;
        digits[row + 1] = 48 + ((code / 10) % 10) as u8;
        digits[row + 2] = 48 + (code % 10) as u8;
        code += 1;
    }

    digits
}

/// Borrow the by-value constant as a promoted static so `as_str` keeps its
/// static backing storage.
#[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len() ==>
    result@[i]@ == status_digit_byte(100 + i / 3, i % 3)
))]
fn code_digits() -> &'static [u8; 2700] {
    &CODE_DIGITS
}

#[requires(forall<i: Int> 0 <= i && i < bytes@.len() ==> bytes@[i]@ < 128)]
#[cfg_attr(creusot, ensures(result@.to_bytes() == bytes@))]
fn status_bytes_as_str(bytes: &[u8]) -> &str {
    #[cfg(creusot)]
    proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
        creusot_std::std::string::valid_utf8(bytes@)
    };
    // Safety: the ASCII witness proves this exact byte slice is valid UTF-8.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

/// An HTTP status code (`status-code` in RFC 9110 et al.).
///
/// Constants are provided for known status codes, including those in the IANA
/// [HTTP Status Code Registry](
/// https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml).
///
/// Status code values in the range 100-999 (inclusive) are supported by this
/// type. Values in the range 100-599 are semantically classified by the most
/// significant digit. See [`StatusCode::is_success`], etc. Values above 599
/// are unclassified but allowed for legacy compatibility, though their use is
/// discouraged. Applications may interpret such values as protocol errors.
///
/// # Examples
///
/// ```
/// use http::StatusCode;
///
/// assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
/// assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
/// assert!(StatusCode::OK.is_success());
/// ```
#[derive(Copy, Hash)]
pub struct StatusCode(NonZeroU16);

#[cfg(creusot)]
impl View for StatusCode {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.0.deep_model() }
    }
}

#[cfg(not(creusot))]
impl View for StatusCode {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.0.get()@ }
    }
}

impl DeepModel for StatusCode {
    type DeepModelTy = Int;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

impl Invariant for StatusCode {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { 100 <= self@ && self@ <= 999 }
    }
}

impl Clone for StatusCode {
    #[cfg_attr(creusot, ensures(result@ == self@ && result.invariant()))]
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for StatusCode {
    #[cfg_attr(creusot, ensures(result == (self@ == other@)))]
    fn eq(&self, other: &Self) -> bool {
        self.as_u16() == other.as_u16()
    }
}

impl Eq for StatusCode {}

impl PartialOrd for StatusCode {
    #[cfg_attr(creusot, ensures(
        result == Some(self@.cmp_log(other@))
    ))]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StatusCode {
    #[cfg_attr(creusot, ensures(result == self@.cmp_log(other@)))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u16().cmp(&other.as_u16())
    }
}

/// A possible error value when converting a `StatusCode` from a `u16` or `&str`.
///
/// This error indicates that the supplied input was not a valid number, was less
/// than 100, or was greater than 999.
pub struct InvalidStatusCode {
    _priv: (),
}

impl StatusCode {
    /// Converts a u16 to a status code.
    ///
    /// The function validates the correctness of the supplied u16. It must be
    /// greater or equal to 100 and less than 1000.
    ///
    /// # Example
    ///
    /// ```
    /// use http::StatusCode;
    ///
    /// let ok = StatusCode::from_u16(200).unwrap();
    /// assert_eq!(ok, StatusCode::OK);
    ///
    /// let err = StatusCode::from_u16(99);
    /// assert!(err.is_err());
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => code@ == src@ && code.invariant(),
        Err(_) => src@ < 100 || src@ > 999,
    }))]
    #[inline]
    pub const fn from_u16(src: u16) -> Result<StatusCode, InvalidStatusCode> {
        if let 100..=999 = src {
            if let Some(code) = NonZeroU16::new(src) {
                return Ok(StatusCode(code));
            }
        }
        Err(InvalidStatusCode::new())
    }

    /// Converts a `&[u8]` to a status code.
    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => valid_status_bytes(src@)
            && code@ == status_bytes_value(src@)
            && code.invariant(),
        Err(_) => !valid_status_bytes(src@),
    }))]
    pub fn from_bytes(src: &[u8]) -> Result<StatusCode, InvalidStatusCode> {
        if src.len() != 3 {
            return Err(InvalidStatusCode::new());
        }

        let a = src[0].wrapping_sub(b'0') as u16;
        let b = src[1].wrapping_sub(b'0') as u16;
        let c = src[2].wrapping_sub(b'0') as u16;

        if a == 0 || a > 9 || b > 9 || c > 9 {
            return Err(InvalidStatusCode::new());
        }

        let status = (a * 100) + (b * 10) + c;
        match NonZeroU16::new(status) {
            Some(code) => Ok(StatusCode(code)),
            None => Err(InvalidStatusCode::new()),
        }
    }

    /// Returns the `u16` corresponding to this `StatusCode`.
    ///
    /// # Note
    ///
    /// This is the same as the `From<StatusCode>` implementation, but
    /// included as an inherent method because that implementation doesn't
    /// appear in rustdocs, as well as a way to force the type instead of
    /// relying on inference.
    ///
    /// # Example
    ///
    /// ```
    /// let status = http::StatusCode::OK;
    /// assert_eq!(status.as_u16(), 200);
    /// ```
    #[cfg_attr(creusot, ensures(result@ == self@))]
    #[inline]
    pub const fn as_u16(&self) -> u16 {
        self.0.get()
    }

    /// Returns a &str representation of the `StatusCode`
    ///
    /// The return value only includes a numerical representation of the
    /// status code. The canonical reason is not included.
    ///
    /// # Example
    ///
    /// ```
    /// let status = http::StatusCode::OK;
    /// assert_eq!(status.as_str(), "200");
    /// ```
    #[cfg_attr(creusot, ensures(
        valid_status_bytes(result@.to_bytes())
            && status_bytes_value(result@.to_bytes()) == self@
    ))]
    #[inline]
    pub fn as_str(&self) -> &str {
        let offset = ((self.0.get() - 100) * 3) as usize;
        let table = code_digits();
        let bytes = &table.as_slice()[offset..offset + 3];
        #[cfg(creusot)]
        proof_assert! {
            forall<i: Int> 0 <= i && i < 3 ==>
                bytes@[i]@ == status_digit_byte(self@, i)
        };
        #[cfg(creusot)]
        proof_assert! { valid_status_bytes(bytes@) };
        #[cfg(creusot)]
        proof_assert! { status_bytes_value(bytes@) == self@ };
        status_bytes_as_str(bytes)
    }

    /// Get the standardised `reason-phrase` for this status code.
    ///
    /// This is mostly here for servers writing responses, but could potentially have application
    /// at other times.
    ///
    /// The reason phrase is defined as being exclusively for human readers. You should avoid
    /// deriving any meaning from it at all costs.
    ///
    /// Bear in mind also that in HTTP/2.0 and HTTP/3.0 the reason phrase is abolished from
    /// transmission, and so this canonical reason phrase really is the only reason phrase you’ll
    /// find.
    ///
    /// # Example
    ///
    /// ```
    /// let status = http::StatusCode::OK;
    /// assert_eq!(status.canonical_reason(), Some("OK"));
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(reason) => canonical_reason_model(self@) == Some(reason@),
        None => canonical_reason_model(self@) == None,
    }))]
    pub fn canonical_reason(&self) -> Option<&'static str> {
        canonical_reason(self.0.get())
    }

    /// Check if status is within 100-199.
    #[cfg_attr(creusot, ensures(result == (100 <= self@ && self@ < 200)))]
    #[inline]
    pub fn is_informational(&self) -> bool {
        100 <= self.0.get() && self.0.get() < 200
    }

    /// Check if status is within 200-299.
    #[cfg_attr(creusot, ensures(result == (200 <= self@ && self@ < 300)))]
    #[inline]
    pub fn is_success(&self) -> bool {
        200 <= self.0.get() && self.0.get() < 300
    }

    /// Check if status is within 300-399.
    #[cfg_attr(creusot, ensures(result == (300 <= self@ && self@ < 400)))]
    #[inline]
    pub fn is_redirection(&self) -> bool {
        300 <= self.0.get() && self.0.get() < 400
    }

    /// Check if status is within 400-499.
    #[cfg_attr(creusot, ensures(result == (400 <= self@ && self@ < 500)))]
    #[inline]
    pub fn is_client_error(&self) -> bool {
        400 <= self.0.get() && self.0.get() < 500
    }

    /// Check if status is within 500-599.
    #[cfg_attr(creusot, ensures(result == (500 <= self@ && self@ < 600)))]
    #[inline]
    pub fn is_server_error(&self) -> bool {
        500 <= self.0.get() && self.0.get() < 600
    }
}

impl fmt::Debug for StatusCode {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(
    creusot_std::std::fmt::formatter_extends(before, middle)
        && creusot_std::std::fmt::formatter_extends(middle, after)
)]
#[ensures(result)]
#[ensures(
    result == creusot_std::std::fmt::formatter_extends(before, after)
)]
fn status_formatter_extends_transitive(
    before: Seq<Int>,
    middle: Seq<Int>,
    after: Seq<Int>,
) -> bool {
    proof_assert! {
        forall<x: Seq<Int>, y: Seq<Int>>
            middle == before.concat(x) && after == middle.concat(y) ==> {
                creusot_std::std::str_index::utf8::seq_concat_assoc(before, x, y);
                proof_assert! { after == before.concat(x.concat(y)) };
                creusot_std::std::fmt::formatter_extends(before, after)
            }
    };
    true
}

/// Formats the status code, *including* the canonical reason.
///
/// # Example
///
/// ```
/// # use http::StatusCode;
/// assert_eq!(format!("{}", StatusCode::OK), "200 OK");
/// ```
impl fmt::Display for StatusCode {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = self.canonical_reason().unwrap_or("<unknown status code>");
        #[cfg(creusot)]
        let before = snapshot!(f.deep_model());

        let first = f.write_str(self.as_str());
        #[cfg(creusot)]
        let after_first = snapshot!(f.deep_model());
        first?;

        let second = f.write_str(" ");
        #[cfg(creusot)]
        let after_second = snapshot!(f.deep_model());
        #[cfg(creusot)]
        proof_assert! {
            status_formatter_extends_transitive(*before, *after_first, *after_second)
        };
        second?;

        let third = f.write_str(reason);
        #[cfg(creusot)]
        proof_assert! {
            status_formatter_extends_transitive(*before, *after_second, f.deep_model())
        };
        third
    }
}

impl Default for StatusCode {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == 200 && result.invariant()))]
    fn default() -> StatusCode {
        match StatusCode::from_u16(200) {
            Ok(status) => status,
            Err(_) => unreachable!(),
        }
    }
}

impl PartialEq<u16> for StatusCode {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self@ == other@)))]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16() == *other
    }
}

impl PartialEq<StatusCode> for u16 {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self@ == other@)))]
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other.as_u16()
    }
}

impl From<StatusCode> for u16 {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == status@))]
    fn from(status: StatusCode) -> u16 {
        status.0.get()
    }
}

impl FromStr for StatusCode {
    type Err = InvalidStatusCode;

    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => valid_status_bytes(s@.to_bytes())
            && code@ == status_bytes_value(s@.to_bytes())
            && code.invariant(),
        Err(_) => !valid_status_bytes(s@.to_bytes()),
    }))]
    fn from_str(s: &str) -> Result<StatusCode, InvalidStatusCode> {
        StatusCode::from_bytes(s.as_bytes())
    }
}

impl From<&StatusCode> for StatusCode {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == t@ && result.invariant()))]
    fn from(t: &StatusCode) -> Self {
        *t
    }
}

impl TryFrom<&[u8]> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => valid_status_bytes(t@)
            && code@ == status_bytes_value(t@)
            && code.invariant(),
        Err(_) => !valid_status_bytes(t@),
    }))]
    fn try_from(t: &[u8]) -> Result<Self, Self::Error> {
        StatusCode::from_bytes(t)
    }
}

impl TryFrom<&str> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => valid_status_bytes(t@.to_bytes())
            && code@ == status_bytes_value(t@.to_bytes())
            && code.invariant(),
        Err(_) => !valid_status_bytes(t@.to_bytes()),
    }))]
    fn try_from(t: &str) -> Result<Self, Self::Error> {
        StatusCode::from_bytes(t.as_bytes())
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(code) => code@ == t@ && code.invariant(),
        Err(_) => t@ < 100 || t@ > 999,
    }))]
    fn try_from(t: u16) -> Result<Self, Self::Error> {
        StatusCode::from_u16(t)
    }
}

macro_rules! status_codes {
    (
        $(
            $(#[$docs:meta])*
            ($num:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        $(
            #[cfg(creusot)]
            /// Verifier-only canonical reason phrase for this status code.
            pub const $konst: &str = $phrase;
        )+

        impl StatusCode {
        $(
            #[cfg(not(all(creusot, http_status_leaf)))]
            $(#[$docs])*
            pub const $konst: StatusCode = StatusCode(unsafe { NonZeroU16::new_unchecked($num) });
        )+

        }

        #[cfg(creusot)]
        #[logic(open)]
        pub fn canonical_reason_model(num: Int) -> Option<Seq<char>> {
            pearlite! {
                $( if num == $num {
                    Some($konst@)
                } else )+
                { None }
            }
        }

        #[cfg_attr(creusot, ensures(match result {
            Some(reason) => canonical_reason_model(num@) == Some(reason@),
            None => canonical_reason_model(num@) == None,
        }))]
        fn canonical_reason(num: u16) -> Option<&'static str> {
            match num {
                $(
                $num => Some($phrase),
                )+
                _ => None
            }
        }
    }
}

status_codes! {
    /// 100 Continue
    /// [[RFC9110, Section 15.2.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.2.1)]
    (100, CONTINUE, "Continue");
    /// 101 Switching Protocols
    /// [[RFC9110, Section 15.2.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.2.2)]
    (101, SWITCHING_PROTOCOLS, "Switching Protocols");
    /// 102 Processing
    /// [[RFC2518, Section 10.1](https://datatracker.ietf.org/doc/html/rfc2518#section-10.1)]
    (102, PROCESSING, "Processing");
    /// 103 Early Hints
    /// [[RFC8297, Section 2](https://datatracker.ietf.org/doc/html/rfc8297#section-2)]
    (103, EARLY_HINTS, "Early Hints");

    /// 200 OK
    /// [[RFC9110, Section 15.3.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.1)]
    (200, OK, "OK");
    /// 201 Created
    /// [[RFC9110, Section 15.3.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.2)]
    (201, CREATED, "Created");
    /// 202 Accepted
    /// [[RFC9110, Section 15.3.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.3)]
    (202, ACCEPTED, "Accepted");
    /// 203 Non-Authoritative Information
    /// [[RFC9110, Section 15.3.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.4)]
    (203, NON_AUTHORITATIVE_INFORMATION, "Non Authoritative Information");
    /// 204 No Content
    /// [[RFC9110, Section 15.3.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.5)]
    (204, NO_CONTENT, "No Content");
    /// 205 Reset Content
    /// [[RFC9110, Section 15.3.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.6)]
    (205, RESET_CONTENT, "Reset Content");
    /// 206 Partial Content
    /// [[RFC9110, Section 15.3.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.7)]
    (206, PARTIAL_CONTENT, "Partial Content");
    /// 207 Multi-Status
    /// [[RFC4918, Section 11.1](https://datatracker.ietf.org/doc/html/rfc4918#section-11.1)]
    (207, MULTI_STATUS, "Multi-Status");
    /// 208 Already Reported
    /// [[RFC5842, Section 7.1](https://datatracker.ietf.org/doc/html/rfc5842#section-7.1)]
    (208, ALREADY_REPORTED, "Already Reported");

    /// 226 IM Used
    /// [[RFC3229, Section 10.4.1](https://datatracker.ietf.org/doc/html/rfc3229#section-10.4.1)]
    (226, IM_USED, "IM Used");

    /// 300 Multiple Choices
    /// [[RFC9110, Section 15.4.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.1)]
    (300, MULTIPLE_CHOICES, "Multiple Choices");
    /// 301 Moved Permanently
    /// [[RFC9110, Section 15.4.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.2)]
    (301, MOVED_PERMANENTLY, "Moved Permanently");
    /// 302 Found
    /// [[RFC9110, Section 15.4.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.3)]
    (302, FOUND, "Found");
    /// 303 See Other
    /// [[RFC9110, Section 15.4.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.4)]
    (303, SEE_OTHER, "See Other");
    /// 304 Not Modified
    /// [[RFC9110, Section 15.4.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.5)]
    (304, NOT_MODIFIED, "Not Modified");
    /// 305 Use Proxy
    /// [[RFC9110, Section 15.4.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.6)]
    (305, USE_PROXY, "Use Proxy");
    /// 307 Temporary Redirect
    /// [[RFC9110, Section 15.4.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.7)]
    (307, TEMPORARY_REDIRECT, "Temporary Redirect");
    /// 308 Permanent Redirect
    /// [[RFC9110, Section 15.4.8](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.8)]
    (308, PERMANENT_REDIRECT, "Permanent Redirect");

    /// 400 Bad Request
    /// [[RFC9110, Section 15.5.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.1)]
    (400, BAD_REQUEST, "Bad Request");
    /// 401 Unauthorized
    /// [[RFC9110, Section 15.5.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.2)]
    (401, UNAUTHORIZED, "Unauthorized");
    /// 402 Payment Required
    /// [[RFC9110, Section 15.5.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.3)]
    (402, PAYMENT_REQUIRED, "Payment Required");
    /// 403 Forbidden
    /// [[RFC9110, Section 15.5.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.4)]
    (403, FORBIDDEN, "Forbidden");
    /// 404 Not Found
    /// [[RFC9110, Section 15.5.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.5)]
    (404, NOT_FOUND, "Not Found");
    /// 405 Method Not Allowed
    /// [[RFC9110, Section 15.5.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.6)]
    (405, METHOD_NOT_ALLOWED, "Method Not Allowed");
    /// 406 Not Acceptable
    /// [[RFC9110, Section 15.5.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.7)]
    (406, NOT_ACCEPTABLE, "Not Acceptable");
    /// 407 Proxy Authentication Required
    /// [[RFC9110, Section 15.5.8](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.8)]
    (407, PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required");
    /// 408 Request Timeout
    /// [[RFC9110, Section 15.5.9](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.9)]
    (408, REQUEST_TIMEOUT, "Request Timeout");
    /// 409 Conflict
    /// [[RFC9110, Section 15.5.10](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.10)]
    (409, CONFLICT, "Conflict");
    /// 410 Gone
    /// [[RFC9110, Section 15.5.11](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.11)]
    (410, GONE, "Gone");
    /// 411 Length Required
    /// [[RFC9110, Section 15.5.12](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.12)]
    (411, LENGTH_REQUIRED, "Length Required");
    /// 412 Precondition Failed
    /// [[RFC9110, Section 15.5.13](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.13)]
    (412, PRECONDITION_FAILED, "Precondition Failed");
    /// 413 Payload Too Large
    /// [[RFC9110, Section 15.5.14](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.14)]
    (413, PAYLOAD_TOO_LARGE, "Payload Too Large");
    /// 414 URI Too Long
    /// [[RFC9110, Section 15.5.15](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.15)]
    (414, URI_TOO_LONG, "URI Too Long");
    /// 415 Unsupported Media Type
    /// [[RFC9110, Section 15.5.16](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.16)]
    (415, UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type");
    /// 416 Range Not Satisfiable
    /// [[RFC9110, Section 15.5.17](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.17)]
    (416, RANGE_NOT_SATISFIABLE, "Range Not Satisfiable");
    /// 417 Expectation Failed
    /// [[RFC9110, Section 15.5.18](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.18)]
    (417, EXPECTATION_FAILED, "Expectation Failed");
    /// 418 I'm a teapot
    /// [curiously not registered by IANA but [RFC2324, Section 2.3.2](https://datatracker.ietf.org/doc/html/rfc2324#section-2.3.2)]
    (418, IM_A_TEAPOT, "I'm a teapot");

    /// 421 Misdirected Request
    /// [[RFC9110, Section 15.5.20](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.20)]
    (421, MISDIRECTED_REQUEST, "Misdirected Request");
    /// 422 Unprocessable Entity
    /// [[RFC9110, Section 15.5.21](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.21)]
    (422, UNPROCESSABLE_ENTITY, "Unprocessable Entity");
    /// 423 Locked
    /// [[RFC4918, Section 11.3](https://datatracker.ietf.org/doc/html/rfc4918#section-11.3)]
    (423, LOCKED, "Locked");
    /// 424 Failed Dependency
    /// [[RFC4918, Section 11.4](https://tools.ietf.org/html/rfc4918#section-11.4)]
    (424, FAILED_DEPENDENCY, "Failed Dependency");

    /// 425 Too early
    /// [[RFC8470, Section 5.2](https://httpwg.org/specs/rfc8470.html#status)]
    (425, TOO_EARLY, "Too Early");

    /// 426 Upgrade Required
    /// [[RFC9110, Section 15.5.22](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.22)]
    (426, UPGRADE_REQUIRED, "Upgrade Required");

    /// 428 Precondition Required
    /// [[RFC6585, Section 3](https://datatracker.ietf.org/doc/html/rfc6585#section-3)]
    (428, PRECONDITION_REQUIRED, "Precondition Required");
    /// 429 Too Many Requests
    /// [[RFC6585, Section 4](https://datatracker.ietf.org/doc/html/rfc6585#section-4)]
    (429, TOO_MANY_REQUESTS, "Too Many Requests");

    /// 431 Request Header Fields Too Large
    /// [[RFC6585, Section 5](https://datatracker.ietf.org/doc/html/rfc6585#section-5)]
    (431, REQUEST_HEADER_FIELDS_TOO_LARGE, "Request Header Fields Too Large");

    /// 451 Unavailable For Legal Reasons
    /// [[RFC7725, Section 3](https://tools.ietf.org/html/rfc7725#section-3)]
    (451, UNAVAILABLE_FOR_LEGAL_REASONS, "Unavailable For Legal Reasons");

    /// 500 Internal Server Error
    /// [[RFC9110, Section 15.6.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.1)]
    (500, INTERNAL_SERVER_ERROR, "Internal Server Error");
    /// 501 Not Implemented
    /// [[RFC9110, Section 15.6.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.2)]
    (501, NOT_IMPLEMENTED, "Not Implemented");
    /// 502 Bad Gateway
    /// [[RFC9110, Section 15.6.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.3)]
    (502, BAD_GATEWAY, "Bad Gateway");
    /// 503 Service Unavailable
    /// [[RFC9110, Section 15.6.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.4)]
    (503, SERVICE_UNAVAILABLE, "Service Unavailable");
    /// 504 Gateway Timeout
    /// [[RFC9110, Section 15.6.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.5)]
    (504, GATEWAY_TIMEOUT, "Gateway Timeout");
    /// 505 HTTP Version Not Supported
    /// [[RFC9110, Section 15.6.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.6)]
    (505, HTTP_VERSION_NOT_SUPPORTED, "HTTP Version Not Supported");
    /// 506 Variant Also Negotiates
    /// [[RFC2295, Section 8.1](https://datatracker.ietf.org/doc/html/rfc2295#section-8.1)]
    (506, VARIANT_ALSO_NEGOTIATES, "Variant Also Negotiates");
    /// 507 Insufficient Storage
    /// [[RFC4918, Section 11.5](https://datatracker.ietf.org/doc/html/rfc4918#section-11.5)]
    (507, INSUFFICIENT_STORAGE, "Insufficient Storage");
    /// 508 Loop Detected
    /// [[RFC5842, Section 7.2](https://datatracker.ietf.org/doc/html/rfc5842#section-7.2)]
    (508, LOOP_DETECTED, "Loop Detected");

    /// 510 Not Extended
    /// [[RFC2774, Section 7](https://datatracker.ietf.org/doc/html/rfc2774#section-7)]
    (510, NOT_EXTENDED, "Not Extended");
    /// 511 Network Authentication Required
    /// [[RFC6585, Section 6](https://datatracker.ietf.org/doc/html/rfc6585#section-6)]
    (511, NETWORK_AUTHENTICATION_REQUIRED, "Network Authentication Required");
}

impl InvalidStatusCode {
    const fn new() -> InvalidStatusCode {
        InvalidStatusCode { _priv: () }
    }
}

impl fmt::Debug for InvalidStatusCode {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidStatusCode")
            // skip _priv noise
            .finish()
    }
}

impl fmt::Display for InvalidStatusCode {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid status code")
    }
}

impl Error for InvalidStatusCode {}

const CODE_DIGITS: [u8; 2700] = build_code_digits();
