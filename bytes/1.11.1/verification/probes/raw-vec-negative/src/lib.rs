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

use raw_vec::{PhysicalRegion, RawAllocation, Recovery, resume_vec};

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
