//! Requested-capacity construction joined directly to the existing B1 Vec
//! detachment boundary.
//!
//! Creusot's Vec model contains the initialized sequence but does not retain
//! allocator capacity. Consequently, a separately returned `Vec` and sampled
//! capacity cannot later be proved to describe the same allocation. This
//! helper constructs and immediately detaches the same Vec, so the requested
//! capacity fact is stated over the exact allocation and B1 capabilities that
//! callers receive.

use alloc::vec::Vec;
use creusot_std::prelude::*;

use super::{
    bound_ptr::detach_bound_vec,
    raw_vec::{BoundPtr, PhysicalRegion, Recovery},
};

/// Construct an empty Vec and detach its exact allocation through B1.
///
/// # Trusted physical boundary
///
/// Creusot's standard-library contract does not state that
/// `Vec::with_capacity(requested)` returns a Vec whose reported capacity is at
/// least `requested`. This bridge supplies that documented allocator fact for
/// the capacity returned by `detach_bound_vec` on the very same Vec. The
/// remaining postconditions repeat that helper's B1 pointer, recovery, and
/// full-region guarantees specialized to an empty input: every allocation
/// slot is `Unknown` (`Some(None)`). No BytesMut handle, shared protocol,
/// reference-count fact, or additional permission beyond B1 is assumed here.
#[trusted]
#[ensures(result.0@ == Some((
    result.3.inner_logic().0.namespace(),
    result.2@,
    0int,
)))]
#[ensures(result.0.invariant())]
#[ensures(result.1@ == 0)]
#[ensures(result.2@ >= requested@)]
#[ensures(result.3.inner_logic().0.invariant())]
#[ensures(result.3.inner_logic().1.invariant())]
#[ensures(result.3.inner_logic().0.capacity() == result.2@)]
#[ensures(result.3.inner_logic().1.capacity() == result.2@)]
#[ensures(result.3.inner_logic().0.namespace() == result.3.inner_logic().1.namespace())]
#[ensures(result.3.inner_logic().0.namespace() == result.3.inner_logic().1.resource_id())]
#[ensures(result.3.inner_logic().1.lo() == 0)]
#[ensures(result.3.inner_logic().1.hi() == result.2@)]
#[ensures(forall<index: Int> 0 <= index && index < result.2@ ==>
    result.3.inner_logic().1.slot(index) == Some(None))]
pub(crate) fn with_capacity_bound(
    requested: usize,
) -> (BoundPtr, usize, usize, Ghost<(Recovery, PhysicalRegion)>) {
    let vector = Vec::with_capacity(requested);
    detach_bound_vec(vector)
}

/// Native probe wrapper that releases the detached allocation after reading
/// its actual length and capacity.
#[cfg(not(creusot))]
pub fn native_allocate_and_release(requested: usize) -> (usize, usize) {
    let (bound, len, capacity, capabilities) = with_capacity_bound(requested);
    // SAFETY: `with_capacity_bound` returns the matching B1 recovery and full
    // region for this exact allocation.
    unsafe { super::raw_vec::deallocate_bound_vec(bound, capacity, capabilities) };
    (len, capacity)
}
