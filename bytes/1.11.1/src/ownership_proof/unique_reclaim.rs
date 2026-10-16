//! Move a unique handle's disjoint initialized view to its allocation base.
//! Only existing B4 borrows perform physical access; split/join retain all
//! allocation authority. The caller keeps its Recovery capability unchanged.

use creusot_std::prelude::*;
use super::raw_vec::{self, BoundPtr, PhysicalRegion};

#[requires(view.invariant() && view@ != None)]
#[requires(base.invariant() && base@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(view@ == Some((region.inner_logic().namespace(), region.inner_logic().capacity(), offset@)))]
#[requires(base@ == Some((region.inner_logic().namespace(), region.inner_logic().capacity(), 0int)))]
#[requires(view.raw_pointer() == base.raw_pointer().offset_logic(offset@))]
#[requires(view.current_address() == base.current_address() + offset@)]
#[requires(region.inner_logic().lo() == 0 && region.inner_logic().hi() == region.inner_logic().capacity())]
#[requires(len <= offset)]
#[requires(offset@ + len@ <= region.inner_logic().capacity())]
#[requires(forall<i: Int> 0 <= i && i < len@ ==> raw_vec::slot_known(region.inner_logic().slot(offset@ + i)))]
#[ensures(result.0.invariant())]
#[ensures(result.0@ == Some((view@.unwrap_logic().0, view@.unwrap_logic().1, 0int)))]
#[ensures(result.0@ == base@)]
#[ensures(result.0.raw_pointer() == base.raw_pointer())]
#[ensures(result.0.current_address() == base.current_address())]
#[ensures(result.1.inner_logic().invariant())]
#[ensures(result.1.inner_logic().namespace() == region.inner_logic().namespace())]
#[ensures(result.1.inner_logic().capacity() == region.inner_logic().capacity())]
#[ensures(result.1.inner_logic().resource_id() == region.inner_logic().resource_id())]
#[ensures(result.1.inner_logic().lo() == 0 && result.1.inner_logic().hi() == region.inner_logic().capacity())]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==> result.1.inner_logic().slot(i) == region.inner_logic().slot(offset@ + i))]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==> raw_vec::slot_known(result.1.inner_logic().slot(i)))]
#[ensures(forall<i: Int> len@ <= i && i < region.inner_logic().capacity() ==> result.1.inner_logic().slot(i) == region.inner_logic().slot(i))]
pub(crate) fn reclaim(
    view: BoundPtr, base: BoundPtr, offset: usize, len: usize, region: Ghost<PhysicalRegion>,
) -> (BoundPtr, Ghost<PhysicalRegion>) {
    let base = view.retreat_within_region(offset, base, region.borrow());
    let fragments = ghost! { region.into_inner().split_at(*Int::new(offset as i128)) };
    let (left, right) = fragments.split();
    let (left, right) = copy_prefix(view, base, offset, len, left, right);
    let full = ghost! { left.into_inner().join(right.into_inner()) };
    (base, full)
}

/// Copy the visible prefix between the two actual region fragments produced
/// by `split_at`. This checked boundary keeps B4 borrow effects local and
/// returns the destination fragment with its source-slot relation explicit.
#[requires(view.invariant() && view@ != None)]
#[requires(base.invariant() && base@ != None)]
#[requires(left.inner_logic().invariant() && right.inner_logic().invariant())]
#[requires(view@ == Some((left.inner_logic().namespace(), left.inner_logic().capacity(), offset@)))]
#[requires(base@ == Some((left.inner_logic().namespace(), left.inner_logic().capacity(), 0int)))]
#[requires(view.raw_pointer() == base.raw_pointer().offset_logic(offset@))]
#[requires(view.current_address() == base.current_address() + offset@)]
#[requires(left.inner_logic().namespace() == right.inner_logic().namespace())]
#[requires(left.inner_logic().capacity() == right.inner_logic().capacity())]
#[requires(left.inner_logic().resource_id() == right.inner_logic().resource_id())]
#[requires(left.inner_logic().lo() == 0 && left.inner_logic().hi() == offset@)]
#[requires(right.inner_logic().lo() == offset@ && right.inner_logic().hi() == left.inner_logic().capacity())]
#[requires(len <= offset)]
#[requires(offset@ + len@ <= left.inner_logic().capacity())]
#[requires(forall<i: Int> 0 <= i && i < len@ ==>
    raw_vec::slot_known(right.inner_logic().slot(offset@ + i)))]
#[ensures(result.0.inner_logic().invariant())]
#[ensures(result.0.inner_logic().namespace() == left.inner_logic().namespace())]
#[ensures(result.0.inner_logic().capacity() == left.inner_logic().capacity())]
#[ensures(result.0.inner_logic().resource_id() == left.inner_logic().resource_id())]
#[ensures(result.0.inner_logic().lo() == left.inner_logic().lo() &&
    result.0.inner_logic().hi() == left.inner_logic().hi())]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==>
    result.0.inner_logic().slot(i) == right.inner_logic().slot(offset@ + i))]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==>
    raw_vec::slot_known(result.0.inner_logic().slot(i)))]
#[ensures(forall<i: Int> len@ <= i && i < offset@ ==>
    result.0.inner_logic().slot(i) == left.inner_logic().slot(i))]
#[ensures(result.1.inner_logic() == right.inner_logic())]
fn copy_prefix(
    view: BoundPtr,
    base: BoundPtr,
    offset: usize,
    len: usize,
    mut left: Ghost<PhysicalRegion>,
    right: Ghost<PhysicalRegion>,
) -> (Ghost<PhysicalRegion>, Ghost<PhysicalRegion>) {
    // [0, len) is in the destination fragment; [offset, offset+len) is in
    // the source fragment. Their physical permissions remain disjoint.
    let source = unsafe { raw_vec::borrow_bound(&view, len, right.borrow()) };
    let destination = unsafe { raw_vec::borrow_bound_uninit_mut(base, len, left.borrow_mut()) };
    // Both paths retain the original one-call memcpy kernel. B4 borrows carry
    // the physical authority and this generic boundary describes only bytes.
    unsafe { crate::storage_ops::copy_to_uninit_prefix_raw(destination, source) };
    (left, right)
}
