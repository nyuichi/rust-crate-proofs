use super::BytesMut;
use creusot_std::prelude::*;

/// Convert a frame stated in view-relative slot coordinates to the same
/// frame in allocation coordinates. The snapshots carry only values; the
/// caller retains the actual authority while the write takes place.
#[check(ghost)]
#[requires((*before).buffer_cursor_offset() == (*after).buffer_cursor_offset())]
#[requires(forall<i:Int>
    (*before).len@ + count@ <= i && i < (*before).cap@ ==>
        (*after).buffer_slot(i) == (*before).buffer_slot(i))]
#[ensures(forall<i:Int>
    (*after).buffer_slot(i) == (*after).buffer_allocation_slot(
        (*before).buffer_cursor_offset() + i))]
#[ensures(forall<i:Int>
    (*before).buffer_slot(i) == (*before).buffer_allocation_slot(
        (*before).buffer_cursor_offset() + i))]
#[ensures(forall<i:Int>
    (*before).len@ + count@ <= i && i < (*before).cap@ ==>
        (*after).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i) ==
        (*before).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i))]
pub(crate) fn buffer_view_slot_frame_to_allocation_frame(
    before: Snapshot<BytesMut>, after: Snapshot<BytesMut>, count: usize,
) {
    proof_assert!(forall<i:Int>
        (*before).buffer_slot(i) == (*before).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i));
    proof_assert!(forall<i:Int>
        (*after).buffer_slot(i) == (*after).buffer_allocation_slot(
            (*before).buffer_cursor_offset() + i));
    proof_assert!(forall<i:Int>
        (*before).len@ + count@ <= i && i < (*before).cap@ ==>
            (*after).buffer_allocation_slot(
                (*before).buffer_cursor_offset() + i) ==
            (*before).buffer_allocation_slot(
                (*before).buffer_cursor_offset() + i));
}
