//! Partitioning one pointer view's physical interval.
//!
//! This composes the sealed pointer-offset arithmetic with the affine
//! `PhysicalRegion` split. It establishes spatial correspondence only; a
//! caller still has to prove that the native record's pointer/capacity fields
//! match the supplied view and that any Shared lifetime/control ticket stays
//! live across the transition.

use creusot_std::prelude::*;

use super::raw_vec::{BoundPtr, PhysicalRegion};

/// Split a view and its physical region at a relative offset.
///
/// Both endpoint cuts are supported. In particular, a zero-length result
/// still carries a `PhysicalRegion` token; higher-level Shared protocols must
/// retain the matching control/lifetime ticket independently of this spatial
/// capability.
#[requires(view.invariant() && view@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(view@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(view@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(view@.unwrap_logic().2 == region.inner_logic().lo())]
#[requires(region.inner_logic().hi() == view@.unwrap_logic().2 + extent@)]
#[requires(cut@ <= extent@)]
#[ensures(result.0.0.invariant() && result.0.0@ == view@)]
#[ensures(result.0.0.raw_pointer() == view.raw_pointer())]
#[ensures(result.0.0.current_address() == view.current_address())]
#[ensures(result.0.1.inner_logic().invariant())]
#[ensures(result.1.0.invariant() && result.1.1.inner_logic().invariant())]
#[ensures(result.1.0@ == Some((
    view@.unwrap_logic().0,
    view@.unwrap_logic().1,
    view@.unwrap_logic().2 + cut@,
)))]
#[ensures(result.1.0.current_address() == view.current_address() + cut@)]
#[ensures(result.0.1.inner_logic().namespace() == region.inner_logic().namespace() &&
    result.1.1.inner_logic().namespace() == region.inner_logic().namespace())]
#[ensures(result.0.1.inner_logic().capacity() == region.inner_logic().capacity() &&
    result.1.1.inner_logic().capacity() == region.inner_logic().capacity())]
#[ensures(result.0.1.inner_logic().resource_id() == region.inner_logic().resource_id() &&
    result.1.1.inner_logic().resource_id() == region.inner_logic().resource_id())]
#[ensures(result.0.1.inner_logic().lo() == view@.unwrap_logic().2)]
#[ensures(result.0.1.inner_logic().hi() == view@.unwrap_logic().2 + cut@)]
#[ensures(result.1.1.inner_logic().lo() == view@.unwrap_logic().2 + cut@)]
#[ensures(result.1.1.inner_logic().hi() == region.inner_logic().hi())]
#[ensures(forall<index: Int>
    result.0.1.inner_logic().slot(index) ==
        if view@.unwrap_logic().2 <= index && index < view@.unwrap_logic().2 + cut@ {
            region.inner_logic().slot(index)
        } else { None })]
#[ensures(forall<index: Int>
    result.1.1.inner_logic().slot(index) ==
        if view@.unwrap_logic().2 + cut@ <= index && index < region.inner_logic().hi() {
            region.inner_logic().slot(index)
        } else { None })]
pub(crate) fn split_view_region(
    view: BoundPtr,
    extent: usize,
    region: Ghost<PhysicalRegion>,
    cut: usize,
) -> ((BoundPtr, Ghost<PhysicalRegion>), (BoundPtr, Ghost<PhysicalRegion>)) {
    proof_assert!(region.inner_logic().hi() <= region.inner_logic().capacity());
    proof_assert!(view@.unwrap_logic().2 + cut@ <= region.inner_logic().hi());
    proof_assert!(view@.unwrap_logic().2 + cut@ <= region.inner_logic().capacity());
    let shifted = view.advance_within(cut);
    let split_at: Snapshot<Int> = snapshot!(region.inner_logic().lo() + cut@);
    let fragments = ghost! {
        region.into_inner().split_at(split_at.into_ghost().into_inner())
    };
    let (left, right) = fragments.split();
    proof_assert!(shifted.invariant());
    proof_assert!(shifted@ == Some((
        view@.unwrap_logic().0,
        view@.unwrap_logic().1,
        *split_at,
    )));
    proof_assert!(shifted.current_address() == view.current_address() + cut@);
    ((view, left), (shifted, right))
}
