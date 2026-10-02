#![no_std]
extern crate creusot_std;

use creusot_std::prelude::{bitwise_proof, ensures, requires};

#[bitwise_proof]
#[requires(k@ >= 0 && k@ < 128)]
#[ensures(result@ == n@.div_euclid(k@.pow2()))]
#[ensures(result == n >> k)]
pub fn shift_by_u128_euclid(n: u128, k: u128) -> u128 {
    n >> k
}
