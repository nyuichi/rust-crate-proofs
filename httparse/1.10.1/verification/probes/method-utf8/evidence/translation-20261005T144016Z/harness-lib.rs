#![cfg_attr(not(feature = "std"), no_std)]

use creusot_std::prelude::*;

// Compile the same helper source that lib.rs includes in the crate.
include!("../../../../src/parse_method_utf8.rs");

#[derive(Clone, Copy)]
pub enum MethodOutcome<'a> {
    Complete(&'a str),
    InvalidToken,
}

#[ensures(match result {
    MethodOutcome::Complete(value) => value@.to_bytes() == raw@,
    MethodOutcome::InvalidToken => !valid_utf8(raw@),
})]
pub fn map_method_outcome<'a>(raw: &'a [u8]) -> MethodOutcome<'a> {
    match method_from_bytes(raw) {
        Some(value) => MethodOutcome::Complete(value),
        None => MethodOutcome::InvalidToken,
    }
}
