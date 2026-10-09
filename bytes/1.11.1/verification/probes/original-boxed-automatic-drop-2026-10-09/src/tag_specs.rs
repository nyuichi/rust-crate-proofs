//! Generic assumed exposed-provenance low-bit tag roundtrip and equal-pointer distance.
//! Default native ptr_map exposes an address then reconstructs through exposed
//! provenance; the contract binds its roundtrip to the original exact pointer.
//! Eq-pointer offset_from is zero without inventing allocation ownership.
//! Std analogue: raw pointer exposed-provenance/offset_from interpretation; removal path is
//! supported exact-pointer contracts. Numerical address equality is insufficient.
use creusot_std::prelude::*;
use crate::raw_vec::BoundPtr;
#[logic(opaque)] pub fn tagged_data(pointer:*mut u8)->*mut () {dead}
#[trusted]
#[requires(base.inner_logic().raw_pointer().addr_logic() & 1usize == 0usize)]
#[requires(tagged == tagged_data(base.inner_logic().raw_pointer()))]
#[ensures(result == base.inner_logic().raw_pointer())]
pub fn clear_low_bit(tagged:*mut (),base:Ghost<&BoundPtr>)->*mut u8 {
    #[cfg(creusot)] {unreachable!("generic exact-provenance tag roundtrip")}
    #[cfg(not(creusot))] { (tagged as usize & !1) as *mut u8 }
}
#[trusted]
#[requires(pointer == base as *const u8)]
#[ensures(result == 0isize)]
pub unsafe fn equal_pointer_distance(pointer:*const u8,base:*mut u8)->isize {
    unsafe { pointer.offset_from(base) }
}

// Mathematical lemmas are proved in the shipped bitvector mode, not trusted.
#[check(ghost)]
#[bitwise_proof]
#[ensures(address & 1usize == 0usize || address & 1usize == 1usize)]
pub fn classify_low_bit(address:usize) {}
#[check(ghost)]
#[bitwise_proof]
#[ensures((address | 1usize) & 1usize == 1usize)]
pub fn tagged_low_bit(address:usize) {}
