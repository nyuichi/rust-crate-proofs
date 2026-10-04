//! Compiler-model diagnostic: raw pointer `==` versus address-only equality.
#![allow(unexpected_cfgs)]

#[cfg(creusot)]
use creusot_std::prelude::*;

/// MIR pointer equality is translated to Why3 equality of the pointer values.
/// This positive control captures the resulting identity implication; it does
/// not establish that the model's identity matches Rust's address-only Eq.
#[ensures(result ==> left == right)]
pub fn native_eq_implies_logical_identity(left: *const u8, right: *const u8) -> bool {
    left == right
}

/// `addr_eq`'s std spec promises only equality of logical numeric addresses.
#[ensures(result ==> left.addr_logic() == right.addr_logic())]
pub fn addr_eq_implies_numeric_address(left: *const u8, right: *const u8) -> bool {
    core::ptr::addr_eq(left, right)
}

/// Negative control: equal numeric addresses must not imply full pointer
/// identity when the pointer model can distinguish runtime metadata/provenance.
#[cfg(feature = "wrong_addr_eq_identity")]
#[ensures(result ==> left == right)]
pub fn wrong_addr_eq_implies_logical_identity(left: *const u8, right: *const u8) -> bool {
    core::ptr::addr_eq(left, right)
}
