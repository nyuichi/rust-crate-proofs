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
#[cfg(all(creusot, bytes_original_unique_gate))]
use crate::ownership_proof::raw_vec::{BoundPtr, PhysicalRegion};
use alloc::vec::Vec;
use core::{mem::ManuallyDrop, ptr::NonNull};

/// A private, drop-free Vec carrier for the Shared copy-detach path. The
/// native Vec remains inside `ManuallyDrop` until the caller has released the
/// old Shared owner; the proof-only fields are value snapshots, not pointer or
/// allocation authority.
#[allow(missing_debug_implementations)]
pub(crate) struct GuardedByteVec {
    native: ManuallyDrop<Vec<u8>>,
    #[cfg(creusot)]
    contents: Snapshot<Seq<u8>>,
    #[cfg(creusot)]
    capacity: Snapshot<Int>,
}

#[cfg(creusot)]
impl GuardedByteVec {
    #[logic(open(self))]
    pub(crate) fn contents(&self) -> Seq<u8> {
        pearlite! { *self.contents }
    }

    #[logic(open(self))]
    pub(crate) fn capacity(&self) -> Int {
        pearlite! { *self.capacity }
    }
}

/// STD-GUARDED-VEC-01: the pinned Std model does not cover the generic
/// consuming `ManuallyDrop<Vec<u8>>` carrier used to keep a Vec owner guarded
/// across a separate ownership-release operation. This boundary records only
/// the normal-return Vec sequence and capacity produced by
/// `Vec::with_capacity`; it creates no pointer, allocation, Bytes, Context, or
/// ownership fact.
#[cfg_attr(creusot, trusted)]
#[cfg_attr(creusot, ensures(result.contents() == Seq::empty()))]
#[cfg_attr(creusot, ensures(result.capacity() >= requested@))]
pub(crate) fn guarded_byte_vec_with_capacity(requested: usize) -> GuardedByteVec {
    let native = Vec::with_capacity(requested);
    #[cfg(creusot)]
    let contents = snapshot!(native@);
    #[cfg(creusot)]
    let capacity = snapshot!(creusot_std::std::vec::capacity_model(native));
    GuardedByteVec {
        native: ManuallyDrop::new(native),
        #[cfg(creusot)]
        contents,
        #[cfg(creusot)]
        capacity,
    }
}

/// STD-GUARDED-VEC-02: expose only the ordinary Vec append effect when the
/// requested bytes fit in the carrier's existing capacity. The native body is
/// one `Vec::extend_from_slice` call. No allocation identity or byte pointer
/// is specified.
#[cfg_attr(creusot, trusted)]
#[cfg_attr(creusot, requires(guard.contents().len() + source@.len() <= guard.capacity()))]
#[cfg_attr(creusot, ensures((^guard).contents() == guard.contents().concat(source@)))]
#[cfg_attr(creusot, ensures((^guard).capacity() == guard.capacity()))]
pub(crate) fn guarded_byte_vec_extend_from_slice(
    guard: &mut GuardedByteVec,
    source: &[u8],
) {
    #[cfg(creusot)]
    let old_contents = guard.contents;
    guard.native.extend_from_slice(source);
    #[cfg(creusot)]
    {
        guard.contents = snapshot!((*old_contents).concat(source@));
    }
}

/// STD-GUARDED-VEC-03: consume the guarded wrapper and return its one native
/// Vec owner. The postcondition relates only the resulting Vec value and
/// capacity to the snapshots carried by the wrapper; it does not preserve or
/// duplicate the wrapper after the transfer.
#[cfg_attr(creusot, trusted)]
#[cfg_attr(creusot, ensures(result@ == guard.contents()))]
#[cfg_attr(creusot, ensures(
    creusot_std::std::vec::capacity_model(result) == guard.capacity()
))]
pub(crate) fn guarded_byte_vec_into_inner(guard: GuardedByteVec) -> Vec<u8> {
    ManuallyDrop::into_inner(guard.native)
}

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

/// Apply the standard pointer-distance operation when the two operands are
/// the same non-null byte pointer.
///
/// The proof contract covers only this equal-pointer case and concludes a
/// zero distance. Native callers may use the same operation with other
/// same-allocation operands; this helper provides no general-distance
/// theorem, liveness fact, or dereference permission.
///
/// # Safety
/// Both pointers must refer to the same allocated object. The selected proof
/// path additionally requires exact pointer equality, supplied by its B1
/// allocation and header-frame facts.
#[cfg_attr(creusot, trusted)]
#[cfg_attr(creusot, requires(pointer == base))]
#[cfg_attr(creusot, requires(pointer.addr_logic()@ != 0))]
#[cfg_attr(creusot, ensures(result == 0isize))]
#[inline]
pub(crate) unsafe fn offset_from_equal_u8(pointer: *const u8, base: *const u8) -> isize {
    // SAFETY: the caller guarantees both operands belong to the same live
    // allocation; in the selected proof path they are the same non-null word.
    unsafe { pointer.offset_from(base) }
}

// STD-PTRSUB-01: The pinned pointer contracts do not describe the unsigned
// pointer-subtraction effect. Keep the original native `sub` operation in one
// all-build adapter used by both the checked B1 retreat and the native reclaim
// path. In proof builds, the actual cursor/base descriptors and borrowed live
// region tie the raw pointer arithmetic to one allocation; they grant no
// dereference or ownership permission.
//
// # Safety
// For nonzero `count`, `pointer` must be derived from the same live allocation
// as `base` and `count` must remain within it. A zero count is valid for an
// empty allocation. Proof builds additionally require the actual B1 cursor,
// offset-zero base, and live PhysicalRegion for that allocation.
#[cfg_attr(all(creusot, bytes_original_unique_gate), trusted)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    cursor.invariant() && cursor@ != None && base.invariant() && base@ != None &&
    region.inner_logic().invariant()
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match (cursor@, base@) {
    (Some((namespace, capacity, offset)), Some((base_namespace, base_capacity, base_offset))) =>
        pointer == cursor.raw_pointer() &&
        namespace == base_namespace && capacity == base_capacity && base_offset == 0 &&
        pointer == base.raw_pointer().offset_logic(offset) &&
        count@ <= offset &&
        region.inner_logic().namespace() == namespace &&
        region.inner_logic().capacity() == capacity &&
        region.inner_logic().lo() <= offset - count@ &&
        offset <= region.inner_logic().hi(),
    _ => false,
}))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= isize::MAX@))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.addr_logic()@ == cursor.current_address() - count@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result == pointer.offset_logic(0 - count@)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(match (cursor@, base@) {
    (Some((_, _, offset)), Some((_, _, _))) =>
        result == base.raw_pointer().offset_logic(offset - count@),
    _ => false,
}))]
#[inline]
pub(crate) unsafe fn sub_within_region(
    pointer: *mut u8,
    count: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))] cursor: &BoundPtr,
    #[cfg(all(creusot, bytes_original_unique_gate))] base: &BoundPtr,
    #[cfg(all(creusot, bytes_original_unique_gate))] region: Ghost<&PhysicalRegion>,
) -> *mut u8 {
    unsafe { pointer.sub(count) }
}

// STD-PTRADD-01: the pinned pointer model does not provide the exact effect
// for raw-pointer `add`. Retain the native operation and relate only its
// result to the matching B1 base descriptor and borrowed live region. The
// descriptor/region pair is supplied by the caller; this helper creates no
// ownership or dereference permission.
//
// # Safety
// For nonzero counts, the pointer must be the offset-zero base of the same
// live allocation witnessed by the actual B1 region, and the result must be
// in that allocation or one-past it. A zero count may use an empty/dangling
// Vec pointer, as permitted by the standard pointer contract.
#[cfg_attr(all(creusot, bytes_original_unique_gate), trusted)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    base.invariant() && base@ != None && region.inner_logic().invariant()
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match base@ {
    Some((namespace, capacity, offset)) =>
        offset == 0 && pointer@ == base.raw_pointer() &&
        region.inner_logic().namespace() == namespace &&
        region.inner_logic().capacity() == capacity &&
        region.inner_logic().resource_id() == namespace &&
        region.inner_logic().lo() <= offset &&
        offset + count@ <= region.inner_logic().hi() &&
        base.current_address() + count@ <= usize::MAX@,
    None => false,
}))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(count@ <= isize::MAX@))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result == pointer@.offset_logic(count@)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.addr_logic()@ == base.current_address() + count@
))]
#[inline]
pub(crate) unsafe fn add_within_region(
    pointer: NonNull<u8>,
    count: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))] base: &BoundPtr,
    #[cfg(all(creusot, bytes_original_unique_gate))] region: Ghost<&PhysicalRegion>,
) -> *mut u8 {
    unsafe { pointer.as_ptr().add(count) }
}

// STD-VEC-RESERVE-01: the pinned Std model specifies `Vec::reserve` on a
// directly borrowed Vec, but its contract only preserves the Vec sequence; it
// does not provide the capacity effects required here or model the generic
// consuming transfer through `ManuallyDrop`. Keep the original Vec owner
// guarded across the reserve call (including its unwind path), and expose only
// the standard Vec value/capacity effect on normal return: the Vec value is
// preserved, capacity does not decrease, and the requested `len + additional`
// capacity is met.
// This says nothing about Bytes metadata, allocation ledgers, slot maps, or
// ownership protocols.
#[cfg_attr(creusot, trusted)]
#[cfg_attr(creusot, ensures(result@ == input@))]
#[cfg_attr(creusot, ensures(
    creusot_std::std::vec::capacity_model(result) >=
        creusot_std::std::vec::capacity_model(input)
))]
#[cfg_attr(creusot, ensures(
    creusot_std::std::vec::capacity_model(result) >= input@.len() + additional@
))]
#[inline]
pub(crate) fn reserve_vec_owned(input: Vec<u8>, additional: usize) -> Vec<u8> {
    let mut guarded = ManuallyDrop::new(input);
    guarded.reserve(additional);
    ManuallyDrop::into_inner(guarded)
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
