//! Sealed B1/B2 bridge between a `Vec<u8>` and a physical slot ledger.
//!
//! The only constructors for [`RawAllocation`], [`Recovery`], and
//! [`PhysicalRegion`] are the detach/resume path below. Resource and region
//! values cross ordinary Rust call boundaries only through `Ghost`. The raw
//! pointer and capacity are metadata; they do not grant access by themselves.
//!
//! The trusted B1 interpretation binds the sealed namespace to the exact
//! global-allocator base pointer, capacity, layout, and allocation consumed
//! from the input `Vec`. A Known slot denotes an initialized byte whose value
//! matches the Vec model. Recovery excludes reclamation while present, and a
//! nonempty region excludes reclamation for its interval; an empty region
//! alone proves no allocation liveness. B2 consumes full Recovery and region
//! coverage before rebuilding the standard Vec.

use alloc::vec::Vec;
use core::mem::ManuallyDrop;

use creusot_std::{
    ghost::{NotObjective, resource::Resource},
    logic::{FMap, Id, Int, ra::{RA, excl::Excl}},
    prelude::*,
};

use super::owned_region::{KernelRA, OwnedRegion, SlotMap, map_compose_eq};

/// Logical identity shared by the recovery token and every region from B1.
/// It is not an address and carries no physical access authority.
#[derive(core::clone::Clone, Copy)]
struct AllocationDesc {
    capacity: Int,
    namespace: Id,
}

/// Model the entire allocation domain, preserving the Vec's initialized
/// prefix and recording spare capacity as Unknown slots.
#[logic]
#[requires(0 <= capacity)]
#[ensures(forall<index: Int> result.contains(index) ==
    (0 <= index && index < capacity))]
#[ensures(forall<index: Int> 0 <= index && index < capacity ==>
    result.get(index) == Some(Excl(
        if index < contents.len() { Some(contents[index]) } else { None }
    )))]
#[variant(capacity)]
fn slots_from_vec(contents: Seq<u8>, capacity: Int) -> SlotMap {
    if capacity == 0 {
        FMap::empty()
    } else {
        let index = capacity - 1;
        slots_from_vec(contents, index)
            .insert(index, Excl(if index < contents.len() {
                Some(contents[index])
            } else {
                None
            }))
    }
}

/// Native pointer metadata minted only by [`detach_vec`].
///
/// This wrapper has no destructor and is intentionally not cloneable or
/// constructible outside this module. The `Ghost<Id>` is erased at runtime;
/// it binds proof tokens to the metadata in verified callers.
#[allow(missing_debug_implementations)]
pub(crate) struct RawAllocation {
    base: *mut u8,
    capacity: usize,
    namespace: Ghost<Id>,
    _not_objective: NotObjective,
}

impl View for RawAllocation {
    type ViewTy = (Int, Id);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { (self.capacity@, self.namespace.inner_logic()) }
    }
}

impl Invariant for RawAllocation {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { 0 <= self@.0 && self@.0 <= isize::MAX@ }
    }
}

impl RawAllocation {
    /// Allocation capacity, as logical bytes.
    #[logic(open)]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self@.0 }
    }

    /// Sealed namespace minted with the resource identity by B1.
    #[logic(open)]
    pub(crate) fn namespace(self) -> Id {
        pearlite! { self@.1 }
    }
}

/// The unique allocation-recovery token. This value is only ever returned
/// inside `Ghost` and can only be minted by `detach_vec`.
#[allow(missing_debug_implementations)]
pub(crate) struct Recovery {
    resource: Resource<KernelRA>,
    descriptor: AllocationDesc,
    _not_objective: NotObjective,
}

impl View for Recovery {
    type ViewTy = (KernelRA, Int, Id, Id);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            (self.resource@, self.descriptor.capacity,
             self.descriptor.namespace, self.resource.id())
        }
    }
}

impl Recovery {
    #[logic(open)]
    pub(crate) fn namespace(self) -> Id {
        pearlite! { self@.2 }
    }

    #[logic(open)]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self@.1 }
    }

    #[logic(open)]
    pub(crate) fn invariant(self) -> bool {
        pearlite! {
            self@.0.0 == Some(Excl(())) &&
            self@.0.1.is_empty() &&
            self@.2 == self@.3
        }
    }
}

/// A physical interval token. Its ledger alone is not sufficient to create
/// this type; B1 also binds it to the raw allocation namespace.
#[allow(missing_debug_implementations)]
pub(crate) struct PhysicalRegion {
    ledger: OwnedRegion,
    descriptor: AllocationDesc,
    _not_objective: NotObjective,
}

impl View for PhysicalRegion {
    type ViewTy = (OwnedRegion, Int, Id);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            (self.ledger, self.descriptor.capacity, self.descriptor.namespace)
        }
    }
}

impl Invariant for PhysicalRegion {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self@.0.invariant() &&
            0 <= self@.0@.0 &&
            self@.0@.0 <= self@.0@.1 &&
            self@.0@.1 <= self@.1 &&
            self@.0@.2 == self@.2
        }
    }
}

impl PhysicalRegion {
    #[logic(open)]
    pub(crate) fn lo(self) -> Int {
        pearlite! { self@.0@.0 }
    }

    #[logic(open)]
    pub(crate) fn hi(self) -> Int {
        pearlite! { self@.0@.1 }
    }

    #[logic(open)]
    pub(crate) fn namespace(self) -> Id {
        pearlite! { self@.2 }
    }

    #[logic(open)]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self@.1 }
    }

    #[logic(open)]
    pub(crate) fn resource_id(self) -> Id {
        pearlite! { self@.0@.2 }
    }

    #[logic(open)]
    pub(crate) fn slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self@.0@.3.1.get(index) {
                None => None,
                Some(Excl(value)) => Some(value),
            }
        }
    }

    /// Split only the ledger. The allocation binding and non-objective marker
    /// are preserved unchanged in both returned fragments.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(self.lo() <= at && at <= self.hi())]
    #[ensures(result.0.invariant() && result.1.invariant())]
    #[ensures(result.0.namespace() == self.namespace())]
    #[ensures(result.1.namespace() == self.namespace())]
    #[ensures(result.0.capacity() == self.capacity())]
    #[ensures(result.1.capacity() == self.capacity())]
    #[ensures(result.0.lo() == self.lo() && result.0.hi() == at)]
    #[ensures(result.1.lo() == at && result.1.hi() == self.hi())]
    #[ensures(result.0.resource_id() == self.resource_id())]
    #[ensures(result.1.resource_id() == self.resource_id())]
    #[ensures(forall<index: Int>
        result.0.slot(index) ==
            if self.lo() <= index && index < at { self.slot(index) } else { None })]
    #[ensures(forall<index: Int>
        result.1.slot(index) ==
            if at <= index && index < self.hi() { self.slot(index) } else { None })]
    pub(crate) fn split_at(self, at: Int) -> (Self, Self) {
        let (left, right) = self.ledger.split_at(at);
        (
            Self { ledger: left, descriptor: self.descriptor, _not_objective: NotObjective {} },
            Self { ledger: right, descriptor: self.descriptor, _not_objective: NotObjective {} },
        )
    }

    /// Join contiguous fragments from the same B1 namespace. No slot changes
    /// are permitted by this operation.
    #[check(ghost)]
    #[requires(self.invariant() && other.invariant())]
    #[requires(self.namespace() == other.namespace())]
    #[requires(self.capacity() == other.capacity())]
    #[requires(self.resource_id() == other.resource_id())]
    #[requires(self.hi() == other.lo())]
    #[ensures(result.invariant())]
    #[ensures(result.namespace() == self.namespace())]
    #[ensures(result.capacity() == self.capacity())]
    #[ensures(result.resource_id() == self.resource_id())]
    #[ensures(result.lo() == self.lo() && result.hi() == other.hi())]
    #[ensures(forall<index: Int>
        result.slot(index) ==
            if self.lo() <= index && index < self.hi() {
                self.slot(index)
            } else if other.lo() <= index && index < other.hi() {
                other.slot(index)
            } else {
                None
            })]
    pub(crate) fn join(self, other: Self) -> Self {
        Self {
            ledger: self.ledger.join(other.ledger),
            descriptor: self.descriptor,
            _not_objective: NotObjective {},
        }
    }
}

/// Detach a Vec allocation and mint its unique recovery and full-range tokens.
///
/// # Trusted interpretation (B1)
///
/// The native body consumes the `Vec` destructor without changing its
/// allocation, and returns exactly its base pointer and capacity. The ghost
/// resource is split into a unique Recovery marker and the full `[0, capacity)`
/// region. The initialized prefix retains the Vec model; spare slots become
/// Unknown. This boundary does not grant access by itself; mutable access is a
/// separate B4 bridge.
#[trusted]
#[ensures(result.0.invariant())]
#[ensures(result.1@ == input@.len())]
#[ensures(result.0.capacity() == result.2.inner_logic().0.capacity())]
#[ensures(result.0.capacity() == result.2.inner_logic().1.capacity())]
#[ensures(result.0.capacity() >= input@.len())]
#[ensures(result.2.inner_logic().0.invariant())]
#[ensures(result.2.inner_logic().1.invariant())]
#[ensures(result.2.inner_logic().0.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().1.namespace() == result.0.namespace())]
#[ensures(result.2.inner_logic().0.namespace() == result.2.inner_logic().1.resource_id())]
#[ensures(result.2.inner_logic().1.lo() == 0)]
#[ensures(result.2.inner_logic().1.hi() == result.0.capacity())]
#[ensures(forall<index: Int> 0 <= index && index < result.0.capacity() ==>
    result.2.inner_logic().1.slot(index) ==
        if index < input@.len() { Some(Some(input@[index])) } else { Some(None) })]
pub(crate) fn detach_vec(
    input: Vec<u8>,
) -> (RawAllocation, usize, Ghost<(Recovery, PhysicalRegion)>) {
    let input_model = snapshot!(input@);
    let mut input = ManuallyDrop::new(input);
    let len = input.len();
    let capacity = input.capacity();
    let base = input.as_mut_ptr();

    // The logical constructor defines the exact model map; B1 is the sole
    // physical interpretation connecting it to the allocation.
    let slots_value: Snapshot<SlotMap> = snapshot! {
        slots_from_vec(*input_model, capacity@)
    };
    let empty_slots: Snapshot<SlotMap> = snapshot!(FMap::empty());
    let region_value: Snapshot<KernelRA> = snapshot!((None, *slots_value));
    let recovery_value: Snapshot<KernelRA> = snapshot!((Some(Excl(())), FMap::empty()));
    let allocation_value: Snapshot<KernelRA> = snapshot!((Some(Excl(())), *slots_value));
    ghost! {
        map_compose_eq(empty_slots, slots_value, slots_value);
        proof_assert!((*recovery_value).op(*region_value) == Some(*allocation_value));
        proof_assert!(KernelRA::incl_eq_op(*recovery_value, *region_value, *allocation_value));
    };

    let binding: Ghost<(Id, (Recovery, PhysicalRegion))> = ghost! {
        let allocation = Resource::alloc(allocation_value).into_inner();
        let (recovery_resource, region_resource) =
            allocation.split(recovery_value, region_value);
        let namespace = region_resource.id_ghost();
        let capacity_int: Int = *Int::new(capacity as i128);
        let descriptor = AllocationDesc { capacity: capacity_int, namespace };
        let recovery = Recovery {
            resource: recovery_resource,
            descriptor,
            _not_objective: NotObjective {},
        };
        let ledger = OwnedRegion::from_model_ledger(0int, capacity_int, region_resource);
        let region = PhysicalRegion {
            ledger,
            descriptor,
            _not_objective: NotObjective {},
        };
        (namespace, (recovery, region))
    };
    let (namespace, capabilities) = binding.split();

    (
        RawAllocation { base, capacity, namespace, _not_objective: NotObjective {} },
        len,
        capabilities,
    )
}

/// Rebuild the original Vec from a complete physical region and its Recovery.
///
/// # Safety
///
/// The tokens must be capabilities derived by permitted split/join/access
/// operations from B1 for `raw`; the region must cover the entire allocation;
/// `len` must be within capacity; and every byte in the requested prefix must
/// be Known. This function is B2: the
/// contract connects the verified ledger with the native `Vec::from_raw_parts`
/// reconstruction.
#[trusted]
#[requires(raw.invariant())]
#[requires(len@ <= raw.capacity())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0)]
#[requires(capabilities.inner_logic().1.hi() == raw.capacity())]
#[requires(forall<index: Int> 0 <= index && index < len@ ==>
    exists<value: u8> capabilities.inner_logic().1.slot(index) == Some(Some(value)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<index: Int> 0 <= index && index < len@ ==>
    capabilities.inner_logic().1.slot(index) == Some(Some(result@[index])))]
pub(crate) unsafe fn resume_vec(
    raw: RawAllocation,
    len: usize,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Vec<u8> {
    let RawAllocation { base, capacity, namespace: _, _not_objective: _ } = raw;
    let _ = capabilities;
    // SAFETY: The caller promises exact B1 capabilities and a Known prefix;
    // the B1 allocation descriptor supplies the original base/capacity.
    unsafe { Vec::from_raw_parts(base, len, capacity) }
}

/// Explicitly free a B1 allocation after recovering its complete authority.
///
/// This is B3, a trusted native boundary. It rebuilds a zero-length Vec with
/// the original base and capacity, then runs that Vec's destructor. The
/// original global allocator and `u8` layout are retained by the sealed B1
/// descriptor. No slot needs to be Known: length zero means the reconstructed
/// Vec drops no elements, while its destructor frees the allocation.
///
/// # Safety
///
/// `raw`, `Recovery`, and the full `PhysicalRegion` must be the matching,
/// unconsumed B1 capabilities for the same allocation. No physical borrow may
/// remain live.
#[trusted]
#[requires(raw.invariant())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(capabilities.inner_logic().0.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().1.capacity() == raw.capacity())]
#[requires(capabilities.inner_logic().0.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.namespace() == raw.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == raw.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0)]
#[requires(capabilities.inner_logic().1.hi() == raw.capacity())]
pub(crate) unsafe fn deallocate_vec(
    raw: RawAllocation,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) {
    let RawAllocation { base, capacity, namespace: _, _not_objective: _ } = raw;
    let _ = capabilities;
    // SAFETY: B1 supplied the exact base/capacity/layout and B3 requires the
    // unique recovery marker plus complete physical-region authority.
    let allocation = unsafe { Vec::from_raw_parts(base, 0, capacity) };
    core::mem::drop(allocation);
}

/// Borrow a nonempty initialized interval from the B1 allocation.
///
/// The pointer arithmetic and slice construction are inline at B4's trusted
/// boundary. The region is borrowed for the returned slice's full lifetime;
/// its prophetic ledger records the final values after that borrow ends.
///
/// # Safety
///
/// `raw` and `region` must be the matching B1 binding, and no overlapping
/// physical borrow may be live. The caller must end the returned slice borrow
/// before splitting, joining, resuming, or otherwise using the region.
#[trusted]
#[requires(raw.invariant())]
#[requires(region.inner_logic().invariant())]
#[requires(raw.namespace() == region.inner_logic().namespace())]
#[requires(raw.capacity() == region.inner_logic().capacity())]
#[requires(0 < len@)]
#[requires(region.inner_logic().lo() <= start@)]
#[requires(start@ + len@ <= region.inner_logic().hi())]
#[requires(forall<index: Int> start@ <= index && index < start@ + len@ ==>
    exists<value: u8> region.inner_logic().slot(index) == Some(Some(value)))]
#[ensures(result@.len() == len@)]
#[ensures((^result)@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    region.inner_logic().slot(start@ + offset) == Some(Some(result@[offset])))]
#[ensures((^region.inner_logic()).invariant())]
#[ensures((^region.inner_logic()).namespace() == region.inner_logic().namespace())]
#[ensures((^region.inner_logic()).capacity() == region.inner_logic().capacity())]
#[ensures((^region.inner_logic()).lo() == region.inner_logic().lo())]
#[ensures((^region.inner_logic()).hi() == region.inner_logic().hi())]
#[ensures((^region.inner_logic()).resource_id() == region.inner_logic().resource_id())]
#[ensures(forall<index: Int> start@ <= index && index < start@ + len@ ==>
    (^region.inner_logic()).slot(index) == Some(Some((^result)@[index - start@])))]
#[ensures(forall<index: Int>
    !(start@ <= index && index < start@ + len@) ==>
        (^region.inner_logic()).slot(index) == region.inner_logic().slot(index))]
pub(crate) unsafe fn borrow_mut<'a>(
    raw: &'a RawAllocation,
    start: usize,
    len: usize,
    region: Ghost<&'a mut PhysicalRegion>,
) -> &'a mut [u8] {
    let pointer = unsafe { raw.base.add(start) };
    let _ = region;
    // SAFETY: B1 sealed the original allocation metadata; the contract proves
    // this nonempty interval is inside the uniquely borrowed initialized region.
    unsafe { core::slice::from_raw_parts_mut(pointer, len) }
}
