#![allow(dead_code)]

extern crate creusot_std;
#[allow(unused_imports)]
use creusot_std::prelude::ensures;

#[path = "../../../../src/verification/model.rs"]
mod verification_model;

#[allow(unused_macros)]
#[macro_use]
#[path = "../../../../src/macros.rs"]
mod macros;

#[path = "../../../../src/byteclass.rs"]
mod byteclass;

/// Minimal caller that composes the four source helper contracts.
#[ensures(result.0 == crate::verification_model::is_tchar(b))]
#[ensures(result.1 == crate::verification_model::is_tchar(b))]
#[ensures(result.2 == crate::verification_model::is_uri_byte(b))]
#[ensures(result.3 == crate::verification_model::is_header_value_byte(b))]
fn classify_byte(b: u8) -> (bool, bool, bool, bool) {
    (
        byteclass::is_method_token(b),
        byteclass::is_header_name_token(b),
        byteclass::is_uri_token(b),
        byteclass::is_header_value_token(b),
    )
}
