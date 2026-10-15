//! Pointer and address helpers used by pointer-tagged metadata paths.
//!
//! Address conversion helpers model numerical addresses only. The
//! provenance-preserving `wrapping_offset` operation uses `wrapping_add` on
//! the original pointer; for non-wrapping offsets its proof contract relates
//! the exact result to the standard logical pointer offset. This is pointer
//! metadata only and grants no dereferenceability or memory permission.
//! Metadata pointers made from null are for integer metadata fields only.

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
// This extern spec is usable only for one-byte pointees. Besides the modular
// address calculation, a non-wrapping offset is modeled as the Std pointer
// `offset_logic` operation on the exact input pointer. This records the normal
// provenance-preserving pointer-word result; it does not establish a live
// range, create `Perm`/`PtrLive`, or grant dereferenceability.
#[cfg(creusot)]
extern_spec! {
    impl<T> *mut T {
        #[requires(size_of_logic::<T>() == 1)]
        #[ensures(
            result.addr_logic()@ ==
                (self.addr_logic()@ + offset@) % (usize::MAX@ + 1)
        )]
        #[ensures(
            self.addr_logic()@ + offset@ <= usize::MAX@ ==>
                result == self.offset_logic(offset@)
        )]
        fn wrapping_add(self, offset: usize) -> *mut T;
    }
}

// STD-PTRWRAP-02: the pinned Std overlay has no contract for the signed raw
// pointer `wrapping_offset` operation. Restrict this generic fact to one-byte
// pointees and the non-wrapping address domain. It relates the exact output to
// the input pointer's logical signed offset; it grants no liveness, allocation,
// or dereference permission.
#[cfg(creusot)]
extern_spec! {
    impl<T> *mut T {
        #[requires(size_of_logic::<T>() == 1)]
        #[requires(0 <= self.addr_logic()@ + offset@)]
        #[requires(self.addr_logic()@ + offset@ <= usize::MAX@)]
        #[ensures(result.addr_logic()@ == self.addr_logic()@ + offset@)]
        #[ensures(result == self.offset_logic(offset@))]
        fn wrapping_offset(self, offset: isize) -> *mut T;
    }
}

/// Read a pointer's address without converting its provenance into an integer.
#[cfg(any(miri, creusot))]
#[cfg_attr(creusot, ensures(result == ptr.addr_logic()))]
#[inline]
#[cfg_attr(creusot, check(ghost))]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr.addr()
}

/// Address extraction for normal optimized builds, preserving the published
/// implementation and its minimum supported Rust version.
#[cfg(not(any(miri, creusot)))]
#[inline]
#[cfg_attr(creusot, check(ghost))]
pub(crate) fn pointer_addr<T>(ptr: *const T) -> usize {
    ptr as usize
}

/// Return the requested address by offsetting the original pointer with
/// provenance-preserving wrapping pointer arithmetic.
///
/// For a non-wrapping address, the specification also relates the result to
/// the standard logical pointer-offset operation on the same input pointer.
/// It does not construct or return a `Perm`, `PtrLive`, or other dereference
/// authority.
#[cfg_attr(
    creusot,
    ensures(
        result.addr_logic()@ == (ptr.addr_logic()@ + offset@) % (usize::MAX@ + 1)
    )
)]
#[cfg_attr(
    creusot,
    ensures(
        ptr.addr_logic()@ + offset@ <= usize::MAX@ ==>
            result == ptr.offset_logic(offset@)
    )
)]
#[inline]
pub(crate) fn wrapping_offset(ptr: *mut u8, offset: usize) -> *mut u8 {
    ptr.wrapping_add(offset)
}

/// Apply a signed, non-wrapping byte offset to the original pointer.
///
/// This exposes only the generic Std pointer-offset relation; callers must
/// separately establish allocation ownership and memory validity.
#[cfg_attr(creusot, requires(0 <= ptr.addr_logic()@ + offset@))]
#[cfg_attr(creusot, requires(ptr.addr_logic()@ + offset@ <= usize::MAX@))]
#[cfg_attr(creusot, ensures(result == ptr.offset_logic(offset@)))]
#[cfg_attr(creusot, ensures(result.addr_logic()@ == ptr.addr_logic()@ + offset@))]
#[inline]
pub(crate) fn signed_wrapping_offset(ptr: *mut u8, offset: isize) -> *mut u8 {
    ptr.wrapping_offset(offset)
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
