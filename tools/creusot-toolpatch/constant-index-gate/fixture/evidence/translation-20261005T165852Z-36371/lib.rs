#![allow(unexpected_cfgs)]

extern crate creusot_std;
use creusot_std::prelude::*;

/// Reads every lane through a four-byte literal pattern from an unconstrained array.
#[ensures(result == (bytes@[0]@ == 71 && bytes@[1]@ == 69 && bytes@[2]@ == 84 && bytes@[3]@ == 32))]
pub fn get_pattern(bytes: [u8; 4]) -> bool {
    matches!(bytes, [b'G', b'E', b'T', b' '])
}

/// Negative control: the source returns the pattern predicate, so this clause is false.
#[ensures(result == false)]
pub fn reject_false_contract(bytes: [u8; 4]) -> bool {
    matches!(bytes, [b'G', b'E', b'T', b' '])
}
