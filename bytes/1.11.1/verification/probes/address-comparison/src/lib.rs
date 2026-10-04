//! Actual thin-pointer comparison helper; address facts only.
#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

#[ensures(result == (left.addr_logic() == right.addr_logic()))]
pub fn compare_addresses(left: *const u8, right: *const u8) -> bool {
    provenance_specs::pointer_addr_eq(left, right)
}

#[cfg(feature = "wrong_identity")]
#[ensures(result ==> left == right)]
pub fn rejected_pointer_identity(left: *const u8, right: *const u8) -> bool {
    provenance_specs::pointer_addr_eq(left, right)
}
