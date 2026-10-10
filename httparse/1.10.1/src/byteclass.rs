//! Scalar HTTP byte predicates backed by the original lookup tables.

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, invariant, variant, Int};

/// Determines if byte is a method token char.
///
/// > ```notrust
/// > token          = 1*tchar
/// >
/// > tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*"
/// >                / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
/// >                / DIGIT / ALPHA
/// >                ; any VCHAR, except delimiters
/// > ```
#[inline]
#[ensures(result == crate::verification_model::is_tchar(b))]
pub(crate) fn is_method_token(b: u8) -> bool {
    match b {
        // For the majority case, this can be faster than the table lookup.
        b'A'..=b'Z' => true,
        _ => TOKEN_MAP[b as usize],
    }
}

/// Lookup table for bytes accepted in a URI before UTF-8 validation.
pub(crate) const URI_MAP: [bool; 256] = byte_map!(
    crate::verification_model::is_uri_byte;
    b'!'..=0x7e | 0x80..=0xFF
);

/// Returns whether a byte is accepted in a URI before UTF-8 validation.
#[inline]
#[ensures(result == crate::verification_model::is_uri_byte(b))]
pub(crate) fn is_uri_token(b: u8) -> bool {
    URI_MAP[b as usize]
}

/// Lookup table for HTTP `tchar` bytes used in methods and header names.
pub(crate) const TOKEN_MAP: [bool; 256] = byte_map!(
    crate::verification_model::is_tchar;
    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' |
    b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' |  b'*' | b'+' |
    b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
);

/// Returns whether a byte is an HTTP `tchar` for a header name.
#[inline]
#[ensures(result == crate::verification_model::is_tchar(b))]
pub(crate) fn is_header_name_token(b: u8) -> bool {
    TOKEN_MAP[b as usize]
}

/// Lookup table for bytes accepted in an HTTP header value.
pub(crate) const HEADER_VALUE_MAP: [bool; 256] = byte_map!(
    crate::verification_model::is_header_value_byte;
    b'\t' | b' '..=0x7e | 0x80..=0xFF
);

/// Returns whether a byte is accepted in an HTTP header value.
#[inline]
#[ensures(result == crate::verification_model::is_header_value_byte(b))]
pub(crate) fn is_header_value_token(b: u8) -> bool {
    HEADER_VALUE_MAP[b as usize]
}
