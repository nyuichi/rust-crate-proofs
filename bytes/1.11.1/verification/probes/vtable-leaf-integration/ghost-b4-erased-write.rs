//! Diagnostic ONLY: an intentionally strengthened trusted B4 purity contract.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
use alloc::vec::Vec;
use creusot_std::prelude::*;
#[path = "../../../../src/ownership_proof/owned_region.rs"] mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"] mod raw_vec;
#[path = "../../../../src/provenance_specs.rs"] mod provenance_specs;
#[requires(input@.len() == 1)]
#[ensures(result == 2u8)]
pub fn erased_write(input: Vec<u8>) -> u8 {
    let (raw, _len, capabilities) = raw_vec::detach_vec(input);
    let (bound, capacity) = raw.into_bound_ptr_at_zero();
    let (recovery, mut region) = capabilities.split();
    ghost! {
        let bytes = unsafe { raw_vec::borrow_bound_mut(&bound, 1, ghost! { &mut *region }) };
        bytes[0] = 2;
    };
    let observed = unsafe { raw_vec::borrow_bound(&bound, 1, ghost! { &*region }) }[0];
    let capabilities = ghost! { (recovery.into_inner(), region.into_inner()) };
    unsafe { raw_vec::deallocate_bound_vec(bound, capacity, capabilities) };
    observed
}
#[cfg(test)]
mod tests {
    #[test]
    fn native_write_is_erased() {
        assert_eq!(super::erased_write(vec![1]), 1);
    }
}
