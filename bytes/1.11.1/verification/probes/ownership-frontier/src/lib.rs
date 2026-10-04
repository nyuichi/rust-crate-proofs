//! Diagnostic boundary probes, not replacement BytesMut implementations.
//! Positive case reuses the actual borrowed Box-region helper. Failing cases
//! expose API/resource boundaries; none is counted as runtime caller coverage.
#![allow(unexpected_cfgs)]
use creusot_std::{ghost::perm::Perm, prelude::*};

#[path = "../../region-permissions/src/lib.rs"]
mod borrowed_region;

#[requires(input@.len() >= 2)]
#[requires(0 < split@ && split@ < input@.len())]
#[requires(left@ != right@)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[split@ - 1] == left)]
#[ensures(result@[split@] == right)]
pub fn borrowed_split_positive(input: Box<[u8]>, split: usize, left: u8, right: u8) -> Box<[u8]> {
    borrowed_region::write_disjoint_regions(input, split, left, right)
}

// A diagnostic expected type rejection: the existing API cannot return two
// independently owned permissions. It returns references borrowing `owner`.
#[cfg(feature = "owned_split")]
pub fn owned_split_boundary(
    mut owner: Ghost<Box<Perm<*const [u8]>>>,
    split: Ghost<Int>,
) -> Ghost<(Box<Perm<*const [u8]>>, Box<Perm<*const [u8]>>)> {
    ghost! { (**owner).split_at_mut(*split) }
}

// A PtrLive witness is copyable liveness, not deallocation authority. This
// negative must fail Box::from_raw's false precondition, even for a live base.
#[cfg(feature = "live_recovery")]
#[requires(live.ward() == pointer as *const u8)]
#[requires(live.len() == len)]
pub unsafe fn recover_with_only_liveness(
    pointer: *mut u8,
    len: usize,
    live: Ghost<creusot_std::std::ptr::PtrLive<'_, u8>>,
) -> Box<[u8]> {
    let raw = core::ptr::slice_from_raw_parts_mut(pointer, len);
    unsafe { Box::from_raw(raw) }
}

// Direct prefix of actual BytesMut::from_vec, before wrapping the raw pointer
// in NonNull and packing metadata. This intentionally has no invented Vec
// allocation/permission contract. See source correspondence in the design note.
#[cfg(feature = "vec_raw_prefix")]
#[ensures(result.1@ == vec@.len())]
#[ensures(result.1 <= result.2)]
pub fn from_vec_raw_prefix(vec: Vec<u8>) -> (*mut u8, usize, usize) {
    let mut vec = core::mem::ManuallyDrop::new(vec);
    let ptr = vec.as_mut_ptr();
    let len = vec.len();
    let cap = vec.capacity();
    (ptr, len, cap)
}

// Exact actual body, copied from src/bytes_mut.rs::rebuild_vec. It is reached
// by both promote_to_shared and the KIND_VEC Drop arm. No precondition below
// can conjure the absent ownership resource; arithmetic bounds are not enough.
#[cfg(feature = "rebuild_vec")]
#[requires(off <= isize::MAX as usize)]
#[requires(len <= cap)]
#[requires(cap@ + off@ <= isize::MAX@)]
pub unsafe fn rebuild_vec(ptr: *mut u8, mut len: usize, mut cap: usize, off: usize) -> Vec<u8> {
    let ptr = ptr.sub(off);
    len += off;
    cap += off;

    Vec::from_raw_parts(ptr, len, cap)
}
