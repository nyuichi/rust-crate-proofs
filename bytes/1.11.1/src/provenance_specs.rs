//! Small pointer-address helpers used by the pointer-tagged metadata paths.
//!
//! These functions deliberately model numerical addresses only. The
//! provenance-preserving operation below uses `wrapping_add` on the original
//! allocation-derived pointer, but its contract says nothing about
//! dereferenceability or permission creation. Metadata pointers made from null
//! are for integer metadata fields only.

#[cfg(creusot)]
use creusot_std::prelude::*;
#[cfg(creusot)]
use creusot_std::std::mem::size_of_logic;

/// Compare thin pointers by address without granting logical provenance
/// identity. Normal builds retain native pointer equality and the crate's
/// minimum supported Rust version; the proof build uses the address-only spec.
#[inline]
#[cfg_attr(creusot, ensures(result == (left.addr_logic() == right.addr_logic())))]
pub(crate) fn pointer_addr_eq<T>(left: *const T, right: *const T) -> bool {
    #[cfg(creusot)]
    {
        core::ptr::addr_eq(left, right)
    }
    #[cfg(not(creusot))]
    {
        left == right
    }
}

// STD-PTRWRAP-01: Creusot 0.13 has no contract for raw-pointer wrapping_add.
// This extern spec is usable only for one-byte pointees (the u8 operation used
// by pointer tagging) and states only the numeric address calculation. It says
// nothing about pointer equality, provenance, live ranges, `Perm`, `PtrLive`,
// or dereferenceability.
#[cfg(creusot)]
extern_spec! {
    impl<T> *mut T {
        #[requires(size_of_logic::<T>() == 1)]
        #[ensures(
            result.addr_logic()@ ==
                (self.addr_logic()@ + offset@) % (usize::MAX@ + 1)
        )]
        fn wrapping_add(self, offset: usize) -> *mut T;
    }
}

/// Read a pointer's address without converting its provenance into an integer.
#[cfg(any(miri, creusot))]
#[cfg_attr(creusot, ensures(result == ptr.addr_logic()))]
#[inline]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr.addr()
}

/// Address extraction for normal optimized builds, preserving the published
/// implementation and its minimum supported Rust version.
#[cfg(not(any(miri, creusot)))]
#[inline]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr as usize
}

/// Return the requested address by offsetting the original pointer with
/// provenance-preserving wrapping pointer arithmetic.
///
/// The specification describes only the resulting numerical address. It does
/// not construct or return a `Perm`, `PtrLive`, or other dereference authority.
#[cfg_attr(
    creusot,
    ensures(
        result.addr_logic()@ == (ptr.addr_logic()@ + offset@) % (usize::MAX@ + 1)
    )
)]
#[inline]
pub(crate) fn wrapping_offset(ptr: *mut u8, offset: usize) -> *mut u8 {
    ptr.wrapping_add(offset)
}

/// Obtain a pointer with a requested numerical address by applying
/// provenance-preserving wrapping pointer arithmetic to the original pointer.
///
/// The postcondition states only the address. The runtime operation starts from
/// `ptr`, but this specification does not assert or mint a `Perm` or `PtrLive`.
#[cfg_attr(creusot, ensures(result.addr_logic() == new_addr))]
#[inline]
pub(crate) fn pointer_with_address(ptr: *mut u8, new_addr: usize) -> *mut u8 {
    let old_addr = pointer_addr(ptr);
    let diff = new_addr.wrapping_sub(old_addr);
    wrapping_offset(ptr, diff)
}

/// Build a provenance-less pointer used only to store integer metadata in a
/// pointer-typed field. No allocation permission is associated with it.
#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]
#[inline]
pub(crate) fn metadata_pointer(addr: usize) -> *mut u8 {
    core::ptr::null_mut::<u8>().wrapping_add(addr)
}
