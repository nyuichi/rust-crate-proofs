#![no_std]
extern crate creusot_std;

use creusot_std::prelude::{bitwise_proof, ensures, proof_assert};

#[bitwise_proof]
#[ensures(result@ == n@.div_euclid(64.pow2()))]
pub fn high_half_as_u64(n: u128) -> u64 {
    proof_assert!(n@ >= 0);
    proof_assert!(0 < 64.pow2());
    (n >> 64i32) as u64
}
