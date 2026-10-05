//! Move a unique handle's disjoint initialized view to its allocation base.
//! Only existing B4 borrows perform physical access; split/join retain all
//! allocation authority. The caller keeps its Recovery capability unchanged.

use creusot_std::prelude::*;
use super::raw_vec::{self, BoundPtr, PhysicalRegion};

#[requires(view.invariant() && view@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(view@ == Some((region.inner_logic().namespace(), region.inner_logic().capacity(), offset@)))]
#[requires(region.inner_logic().lo() == 0 && region.inner_logic().hi() == region.inner_logic().capacity())]
#[requires(len <= offset)]
#[requires(offset@ + len@ <= region.inner_logic().capacity())]
#[requires(forall<i: Int> 0 <= i && i < len@ ==> raw_vec::slot_known(region.inner_logic().slot(offset@ + i)))]
#[ensures(result.0.invariant())]
#[ensures(result.0@ == Some((view@.unwrap_logic().0, view@.unwrap_logic().1, 0int)))]
#[ensures(result.0.current_address() == view.current_address() - offset@)]
#[ensures(result.1.inner_logic().invariant())]
#[ensures(result.1.inner_logic().namespace() == region.inner_logic().namespace())]
#[ensures(result.1.inner_logic().capacity() == region.inner_logic().capacity())]
#[ensures(result.1.inner_logic().resource_id() == region.inner_logic().resource_id())]
#[ensures(result.1.inner_logic().lo() == 0 && result.1.inner_logic().hi() == region.inner_logic().capacity())]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==> result.1.inner_logic().slot(i) == region.inner_logic().slot(offset@ + i))]
#[ensures(forall<i: Int> 0 <= i && i < len@ ==> raw_vec::slot_known(result.1.inner_logic().slot(i)))]
#[ensures(forall<i: Int> len@ <= i && i < region.inner_logic().capacity() ==> result.1.inner_logic().slot(i) == region.inner_logic().slot(i))]
pub(crate) fn reclaim(
    view: BoundPtr, offset: usize, len: usize, region: Ghost<PhysicalRegion>,
) -> (BoundPtr, Ghost<PhysicalRegion>) {
    let base = view.retreat_within(offset);
    let fragments = ghost! { region.into_inner().split_at(*Int::new(offset as i128)) };
    let (mut left, right) = fragments.split();
    {
        // [0, len) is in the left fragment; [offset, offset+len) is in
        // the right one. Their physical permissions remain disjoint.
        let source = unsafe { raw_vec::borrow_bound(&view, len, right.borrow()) };
        let destination = unsafe { raw_vec::borrow_bound_uninit_mut(base, len, left.borrow_mut()) };
        crate::storage_ops::copy_to_uninit_prefix(destination, source);
    }
    let full = ghost! { left.into_inner().join(right.into_inner()) };
    (base, full)
}
