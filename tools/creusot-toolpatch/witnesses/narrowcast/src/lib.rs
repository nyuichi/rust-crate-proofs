#![no_std]
extern crate creusot_std;
use creusot_std::prelude::{bitwise_proof, ensures};

#[bitwise_proof]
#[ensures(result@ == n@ % 64.pow2())]
pub fn trunc_u64_bw(n: u128) -> u64 {
    n as u64
}

#[ensures(result@ == n@ % 64.pow2())]
pub fn trunc_u64_int(n: u128) -> u64 {
    n as u64
}
