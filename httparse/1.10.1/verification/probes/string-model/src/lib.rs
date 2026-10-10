#![cfg_attr(not(feature = "std"), no_std)]

use creusot_std::prelude::ensures;

include!("../../../../src/error.rs");
include!("../../../../src/message.rs");

#[cfg(creusot)]
#[ensures(result@ == ""@)]
pub fn empty_string_literal() -> &'static str {
    ""
}

#[cfg(creusot)]
#[ensures(result@ == "A\0"@)]
pub fn ascii_and_nul_literal() -> &'static str {
    "A\0"
}

#[cfg(creusot)]
#[ensures(result@ == r"raw"@)]
pub fn raw_string_literal() -> &'static str {
    r"raw"
}

#[cfg(creusot)]
#[ensures(result@ == "é"@)]
#[ensures(result@.to_bytes() == seq![0xC3u8, 0xA9u8])]
pub fn two_byte_scalar() -> &'static str {
    "é"
}

#[cfg(creusot)]
#[ensures(result@ == "€"@)]
#[ensures(result@.to_bytes() == seq![0xE2u8, 0x82u8, 0xACu8])]
pub fn three_byte_scalar() -> &'static str {
    "€"
}

#[cfg(creusot)]
#[ensures(result@ == "\u{D7FF}\u{E000}\u{10FFFF}"@)]
#[ensures(result@.to_bytes() == seq![
    0xEDu8, 0x9Fu8, 0xBFu8,
    0xEEu8, 0x80u8, 0x80u8,
    0xF4u8, 0x8Fu8, 0xBFu8, 0xBFu8,
])]
pub fn scalar_boundaries() -> &'static str {
    "\u{D7FF}\u{E000}\u{10FFFF}"
}

#[cfg(creusot)]
#[ensures(result@ == ""@)]
pub fn actual_empty_header_name() -> &'static str {
    EMPTY_HEADER.name
}

#[cfg(creusot)]
#[ensures(result@ == "invalid header name"@)]
pub fn actual_error_description() -> &'static str {
    Error::HeaderName.description_str()
}

#[cfg(all(creusot, feature = "negative-probes"))]
#[ensures(result@ == "not the returned literal"@)]
pub fn deliberately_false_string_claim() -> &'static str {
    "returned literal"
}

#[cfg(all(creusot, feature = "negative-probes"))]
#[ensures(result@ == seq![0x41u8])]
pub fn deliberately_false_utf8_claim() -> &'static [u8] {
    "B".as_bytes()
}
