//! Shared specifications for collections backed by partially initialized storage.
//!
//! The representation used here is deliberately independent of inline versus
//! allocated storage.  A storage implementation exposes a sequence of
//! `Option<T>` slots (the view of `MaybeUninit<T>`) and a collection chooses one
//! canonical prefix length.  Raw-pointer and allocator adapters should preserve
//! this relation; they must not invent a second logical cursor.
//!
//! The executable helpers are the ownership boundary for one slot.  Their
//! bodies are proved from the standard-library `MaybeUninit` contracts.  They
//! neither allocate nor use raw pointers, so collection proofs can keep pointer
//! liveness/allocation provenance in a separate, small component.

use crate::prelude::*;
use core::mem::MaybeUninit;

/// Every slot below `len` is initialized and `len` is within the allocation.
///
/// Slots after `len` are intentionally unconstrained: shrinking a collection
/// may leave bytes behind, but ownership of those values has ended.
#[logic(open)]
pub fn initialized_prefix<T>(slots: Seq<MaybeUninit<T>>, len: Int) -> bool {
    pearlite! {
        0 <= len && len <= slots.len()
            && forall<i: Int> 0 <= i && i < len ==> slots[i]@ != None
    }
}

/// Exact ownership shape for a collection: precisely the prefix is owned.
///
/// This stronger predicate is appropriate for operations that grow into the
/// next slot.  `initialized_prefix` remains useful for foreign buffers whose
/// spare bytes are outside the collection's ownership model.
#[logic(open)]
pub fn owned_prefix<T>(slots: Seq<MaybeUninit<T>>, len: Int) -> bool {
    pearlite! {
        initialized_prefix(slots, len)
            && forall<i: Int> len <= i && i < slots.len() ==> slots[i]@ == None
    }
}

/// Functional contents represented by an initialized prefix.
#[logic(open)]
#[requires(initialized_prefix(slots, len))]
#[variant(len)]
pub fn prefix_values<T>(slots: Seq<MaybeUninit<T>>, len: Int) -> Seq<T> {
    pearlite! {
        if len == 0 {
            Seq::empty()
        } else {
            prefix_values(slots, len - 1).push_back(slots[len - 1]@.unwrap_logic())
        }
    }
}

/// Initialize one previously uninitialized slot, transferring `value` into it.
#[check(ghost)]
#[requires(slot@ == None)]
#[ensures((^slot)@ == Some(value))]
pub fn write_slot<T>(slot: &mut MaybeUninit<T>, value: T) {
    let _ = slot.write(value);
}

/// Move the initialized value out of a slot and make the slot uninitialized.
#[check(ghost)]
#[requires(slot@ != None)]
#[ensures(slot@ == Some(result))]
#[ensures((^slot)@ == None)]
pub unsafe fn take_slot<T>(slot: &mut MaybeUninit<T>) -> T {
    let old = core::mem::replace(slot, MaybeUninit::uninit());
    unsafe { old.assume_init() }
}

/// Drop the initialized value in a slot and relinquish its ownership.
#[check(ghost)]
#[requires(slot@ != None)]
#[ensures((^slot)@ == None)]
pub unsafe fn drop_slot<T>(slot: &mut MaybeUninit<T>) {
    unsafe { slot.assume_init_drop() }
}

/// Append to an inline storage prefix.
///
/// This representative caller keeps sequence-update reasoning out of each
/// collection implementation. Allocated storage can expose the same contract
/// once its raw-pointer permission is converted to a mutable slot slice.
#[check(ghost)]
#[trusted]
#[requires(owned_prefix(slots@, len@))]
#[requires(len@ < N@)]
#[ensures(owned_prefix((^slots)@, len@ + 1))]
#[ensures(prefix_values((^slots)@, len@ + 1) == prefix_values(slots@, len@).push_back(value))]
pub fn push_array_slot<T, const N: usize>(
    slots: &mut [MaybeUninit<T>; N],
    len: usize,
    value: T,
) {
    // TRUSTED: Creusot does not yet discharge the final extensional equality
    // between an array update and the recursive `prefix_values` model.
    // Removal condition: prove the array-update/snoc lemma in this module;
    // callers and this functional contract must remain unchanged.
    write_slot(&mut slots[len], value);
}

/// Remove the last owned element from an inline storage prefix.
#[check(ghost)]
#[trusted]
#[requires(owned_prefix(slots@, len@))]
#[requires(0 < len@)]
#[ensures(owned_prefix((^slots)@, len@ - 1))]
#[ensures(prefix_values(slots@, len@) == prefix_values((^slots)@, len@ - 1).push_back(result))]
pub unsafe fn pop_array_slot<T, const N: usize>(
    slots: &mut [MaybeUninit<T>; N],
    len: usize,
) -> T {
    // TRUSTED: symmetric sequence-extensionality gap for removing the last
    // slot. Removal condition: prove the array-update/unsnoc lemma here while
    // retaining this ownership and functional contract.
    unsafe { take_slot(&mut slots[len - 1]) }
}
