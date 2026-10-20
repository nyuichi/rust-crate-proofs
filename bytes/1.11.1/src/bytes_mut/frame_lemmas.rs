use super::BytesMut;
use creusot_std::prelude::*;

/// Equal valid byte views have the same observable slots, even when a reserve
/// operation has moved the visible cursor within the allocation.
#[check(ghost)]
#[requires((*before).buffer_view_valid() && (*after).buffer_view_valid())]
#[requires((*before).len@ == (*after).len@)]
#[requires((*before).buffer_bytes() == (*after).buffer_bytes())]
#[ensures(forall<i:Int> 0 <= i && i < (*before).len@ ==>
    (*after).buffer_slot(i) == (*before).buffer_slot(i))]
pub(crate) fn equal_buffer_bytes_preserve_view_slots(
    before: Snapshot<BytesMut>, after: Snapshot<BytesMut>,
) {
    proof_assert!(forall<i:Int> 0 <= i && i < (*before).len@ ==>
        (*after).buffer_slot(i) == (*before).buffer_slot(i));
}

/// Decompose the current view slots into its visible bytes and a just-written
/// suffix. This keeps sequence algebra separate from the actual write and its
/// affine authority.
#[check(ghost)]
#[requires(forall<i:Int> 0 <= i && i < (*suffix).len() ==>
    (*state).buffer_slot((*state).len@ + i) == Some(Some((*suffix)[i])))]
#[ensures(Seq::create((*state).len@ + (*suffix).len(), |i:Int|
    (*state).buffer_slot(i).unwrap_logic().unwrap_logic()) ==
        (*state).buffer_bytes().concat(*suffix))]
pub(crate) fn buffer_bytes_append_from_slots(
    state: Snapshot<BytesMut>, suffix: Snapshot<Seq<u8>>,
) {
    proof_assert!(Seq::create((*state).len@ + (*suffix).len(), |i:Int|
        (*state).buffer_slot(i).unwrap_logic().unwrap_logic()) ==
            (*state).buffer_bytes().concat(*suffix));
}


/// Convert a frame stated in view-relative slot coordinates to the same
/// frame in allocation coordinates. The snapshots carry only values; the
/// caller retains the actual authority while the write takes place.
#[check(ghost)]
#[requires((*before).buffer_cursor_offset() == (*after).buffer_cursor_offset())]
#[requires(forall<i:Int>
    (*before).len@ + count@ <= i && i < (*before).cap@ ==>
        (*after).buffer_slot(i) == (*before).buffer_slot(i))]
#[requires(forall<i:Int>
    (*before).buffer_cursor_offset() + (*before).cap@ <= i &&
    i < (*before).buffer_allocation_capacity() ==>
        (*after).buffer_allocation_slot(i) == (*before).buffer_allocation_slot(i))]
#[ensures(forall<i:Int>
    (*after).buffer_slot(i) == (*after).buffer_allocation_slot(
        (*before).buffer_cursor_offset() + i))]
#[ensures(forall<i:Int>
    (*before).buffer_slot(i) == (*before).buffer_allocation_slot(
        (*before).buffer_cursor_offset() + i))]
#[ensures(forall<i:Int>
    (*before).buffer_allocation_slot(i) == (*before).buffer_slot(
        i - (*before).buffer_cursor_offset()))]
#[ensures(forall<i:Int>
    (*after).buffer_allocation_slot(i) == (*after).buffer_slot(
        i - (*after).buffer_cursor_offset()))]
#[ensures(forall<i:Int>
    (*before).len@ + count@ <= i && i < (*before).cap@ ==>
        (*after).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i) ==
        (*before).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i))]
#[ensures(forall<i:Int>
    (*before).buffer_cursor_offset() + (*before).len@ + count@ <= i &&
    i < (*before).buffer_allocation_capacity() ==>
        (*after).buffer_allocation_slot(i) == (*before).buffer_allocation_slot(i))]
pub(crate) fn buffer_view_slot_frame_to_allocation_frame(
    before: Snapshot<BytesMut>, after: Snapshot<BytesMut>, count: usize,
) {
    proof_assert!(forall<i:Int>
        (*before).buffer_slot(i) == (*before).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i));
    proof_assert!(forall<i:Int>
        (*before).buffer_allocation_slot(i) == (*before).buffer_slot(
            i - (*before).buffer_cursor_offset()));
    proof_assert!(forall<i:Int>
        (*after).buffer_allocation_slot(i) == (*after).buffer_slot(
            i - (*after).buffer_cursor_offset()));
    proof_assert!(forall<i:Int>
        (*after).buffer_slot(i) == (*after).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i));
    proof_assert!(forall<i:Int>
        (*before).len@ + count@ <= i && i < (*before).cap@ ==>
            (*after).buffer_allocation_slot(
                (*before).buffer_cursor_offset() + i) ==
            (*before).buffer_allocation_slot(
                (*before).buffer_cursor_offset() + i));
    proof_assert!(forall<i:Int>
        (*before).buffer_cursor_offset() + (*before).len@ + count@ <= i &&
        i < (*before).buffer_allocation_capacity() &&
        i < (*before).buffer_cursor_offset() + (*before).cap@ ==>
            (*after).buffer_allocation_slot(i) == (*before).buffer_allocation_slot(i));
    proof_assert!(forall<i:Int>
        (*before).buffer_cursor_offset() + (*before).len@ + count@ <= i &&
        i < (*before).buffer_allocation_capacity() &&
        (*before).buffer_cursor_offset() + (*before).cap@ <= i ==>
            (*after).buffer_allocation_slot(i) == (*before).buffer_allocation_slot(i));
    proof_assert!(forall<i:Int>
        (*before).buffer_cursor_offset() + (*before).len@ + count@ <= i &&
        i < (*before).buffer_allocation_capacity() ==>
            (*after).buffer_allocation_slot(i) == (*before).buffer_allocation_slot(i));
}
