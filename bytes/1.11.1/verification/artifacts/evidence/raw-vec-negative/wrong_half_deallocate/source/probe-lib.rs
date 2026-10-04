//! Focused negative checks for the B2 `resume_vec` contract.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;

use alloc::vec::Vec;
use creusot_std::prelude::*;

#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;

use raw_vec::{
    PhysicalRegion, RawAllocation, Recovery, borrow_mut, deallocate_vec, resume_vec,
};

/// All B2 conditions hold except complete allocation coverage. The prefix of
/// length one is Known and region `[0, 1)` is only half of this allocation.
#[cfg(feature = "wrong_half_region")]
#[requires(raw.invariant())]
#[requires(2usize@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0int)]
#[requires(capabilities.inner_logic().1.hi() == 1int)]
#[requires(capabilities.inner_logic().1.hi() < raw.capacity())]
#[requires(forall<index: Int> 0 <= index && index < 1 ==>
    exists<value: u8> capabilities.inner_logic().1.slot(index) == Some(Some(value)))]
pub(crate) unsafe fn wrong_half_region_recovery(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Vec<u8> {
    // SAFETY: This intentionally asks B2 to accept an incomplete region.
    unsafe { resume_vec(raw, 1, capabilities) }
}

/// All B2 conditions hold except initialized-prefix knowledge. The allocation
/// is fully covered, but slot zero is explicitly Unknown.
#[cfg(feature = "wrong_unknown_prefix")]
#[requires(raw.invariant())]
#[requires(1usize@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0int)]
#[requires(capabilities.inner_logic().1.hi() == raw.capacity())]
#[requires(capabilities.inner_logic().1.slot(0int) == Some(None))]
pub(crate) unsafe fn wrong_unknown_prefix_resume(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Vec<u8> {
    // SAFETY: This intentionally asks B2 to resume an Unknown byte as initialized.
    unsafe { resume_vec(raw, 1, capabilities) }
}

/// All B2 capacity, coverage, and Known-prefix conditions hold. The sealed
/// native descriptor belongs to a different namespace from these full caps.
#[cfg(feature = "wrong_namespace")]
#[requires(raw.invariant())]
#[requires(1usize@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() != raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0int)]
#[requires(capabilities.inner_logic().1.hi() == raw.capacity())]
#[requires(forall<index: Int> 0 <= index && index < 1 ==>
    exists<value: u8> capabilities.inner_logic().1.slot(index) == Some(Some(value)))]
pub(crate) unsafe fn wrong_namespace_resume(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Vec<u8> {
    // SAFETY: This intentionally combines a descriptor with unrelated caps.
    unsafe { resume_vec(raw, 1, capabilities) }
}

/// A mutable borrow cannot expose an Unknown byte as initialized.
#[cfg(feature = "wrong_unknown_borrow")]
#[requires(raw.invariant())]
#[requires(region.inner_logic().invariant())]
#[requires(raw.namespace() == region.inner_logic().namespace())]
#[requires(raw.capacity() == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() == 0int)]
#[requires(1int <= region.inner_logic().hi())]
#[requires(region.inner_logic().slot(0int) == Some(None))]
pub(crate) unsafe fn wrong_unknown_borrow(
    raw: RawAllocation,
    mut region: Ghost<PhysicalRegion>,
) {
    let _bytes = unsafe { borrow_mut(&raw, 0, 1, ghost! { &mut *region }) };
}

/// A nonempty borrow starting at the region's exclusive upper endpoint must
/// be rejected. The requested slot is outside the ledger domain as well.
#[cfg(feature = "wrong_out_of_region_borrow")]
#[requires(raw.invariant())]
#[requires(region.inner_logic().invariant())]
#[requires(raw.namespace() == region.inner_logic().namespace())]
#[requires(raw.capacity() == region.inner_logic().capacity())]
#[requires(2usize@ <= raw.capacity())]
#[requires(region.inner_logic().lo() == 0int)]
#[requires(region.inner_logic().hi() == 1int)]
#[requires(region.inner_logic().slot(1int) == None)]
pub(crate) unsafe fn wrong_out_of_region_borrow(
    raw: RawAllocation,
    mut region: Ghost<PhysicalRegion>,
) {
    let _bytes = unsafe { borrow_mut(&raw, 1, 1, ghost! { &mut *region }) };
}

/// Mutate one borrowed byte, then incorrectly assert that the sealed region
/// still records its pre-borrow value.
#[cfg(feature = "wrong_stale_value_after_mutation")]
#[requires(raw.invariant())]
#[requires(region.inner_logic().invariant())]
#[requires(raw.namespace() == region.inner_logic().namespace())]
#[requires(raw.capacity() == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() == 0int)]
#[requires(1int <= region.inner_logic().hi())]
#[requires(region.inner_logic().slot(0int) == Some(Some(0x11u8)))]
pub(crate) unsafe fn wrong_stale_value_after_mutation(
    raw: RawAllocation,
    mut region: Ghost<PhysicalRegion>,
) {
    {
        let bytes = unsafe { borrow_mut(&raw, 0, 1, ghost! { &mut *region }) };
        bytes[0] = 0x22;
    }
    proof_assert!(region.inner_logic().slot(0int) == Some(Some(0x11u8)));
}

/// B3 cleanup also requires the entire allocation region, even though it
/// does not require any slot to be Known.
#[cfg(feature = "wrong_half_deallocate")]
#[requires(raw.invariant())]
#[requires(2usize@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == raw.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0int)]
#[requires(capabilities.inner_logic().1.hi() == 1int)]
#[requires(capabilities.inner_logic().1.hi() < raw.capacity())]
pub(crate) unsafe fn wrong_half_deallocate(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) {
    // SAFETY: This intentionally asks B3 to free an allocation with partial coverage.
    unsafe { deallocate_vec(raw, capabilities) }
}

/// B3 cleanup rejects Recovery minted in a different namespace even when the
/// region itself has the right namespace, full coverage, and invariant.
#[cfg(feature = "wrong_namespace_deallocate")]
#[requires(raw.invariant())]
#[requires(1usize@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() != raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == raw.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0int)]
#[requires(capabilities.inner_logic().1.hi() == raw.capacity())]
pub(crate) unsafe fn wrong_namespace_deallocate(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) {
    // SAFETY: This intentionally combines Recovery with an unrelated descriptor.
    unsafe { deallocate_vec(raw, capabilities) }
}
