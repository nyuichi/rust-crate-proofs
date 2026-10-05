#![allow(dead_code)]

use core::convert::TryInto;
use creusot_std::prelude::*;

/// Actual caller shape used by `peek_n::<[u8; 8]>`: take exactly eight bytes,
/// then run the standard slice-to-array conversion.
#[ensures(match result {
    Some(array) => input@.len() >= 8
        && array@ == input@.subsequence(0, 8),
    None => input@.len() < 8,
})]
pub fn array8_prefix(input: &[u8]) -> Option<[u8; 8]> {
    let prefix = input.get(..8)?;
    prefix.try_into().ok()
}

/// The fixed-size conversion used by the method parser's four-byte check.
#[ensures(match result {
    Some(array) => input@.len() >= 4
        && array@ == input@.subsequence(0, 4),
    None => input@.len() < 4,
})]
pub fn array4_prefix(input: &[u8]) -> Option<[u8; 4]> {
    let prefix = input.get(..4)?;
    prefix.try_into().ok()
}
