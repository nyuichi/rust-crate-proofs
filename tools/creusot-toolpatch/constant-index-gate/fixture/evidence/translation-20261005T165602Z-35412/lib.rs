#![allow(unexpected_cfgs)]

extern crate creusot_std;
use creusot_std::prelude::*;

/// Reads every lane through a four-byte literal pattern from an unconstrained array.
#[ensures(result == (bytes@[0]@ == b'G'@ && bytes@[1]@ == b'E'@ && bytes@[2]@ == b'T'@ && bytes@[3]@ == b' '@))]
pub fn get_pattern(bytes: [u8; 4]) -> bool {
    matches!(bytes, [b'G', b'E', b'T', b' '])
}

/// Negative control: the source returns the pattern predicate, so this clause is false.
#[ensures(result == false)]
pub fn reject_false_contract(bytes: [u8; 4]) -> bool {
    matches!(bytes, [b'G', b'E', b'T', b' '])
}
