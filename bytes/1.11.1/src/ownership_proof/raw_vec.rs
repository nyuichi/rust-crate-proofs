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

#[cfg(not(creusot))]
#[path = "../allocation_ops.rs"]
mod allocation_ops;

use alloc::vec::Vec;
use core::{mem::ManuallyDrop, ptr::NonNull};

use creusot_std::{
    ghost::{NotObjective, resource::Resource},
    logic::{FMap, Id, Int, ra::{RA, UnitRA, excl::Excl}},
    prelude::*,
};

use super::owned_region::{KernelRA, OwnedRegion, SlotMap, map_compose_eq, map_op_get};

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

/// Preserve every old slot while extending the allocation with Unknown slots.
#[logic]
#[requires(0 <= old_capacity && old_capacity <= capacity)]
#[ensures(forall<index: Int> index < old_capacity ==>
    result.get(index) == old.get(index))]
#[ensures(forall<index: Int> old_capacity <= index && index < capacity ==>
    result.get(index) == Some(Excl(None)))]
#[variant(capacity - old_capacity)]
fn slots_grown(old: SlotMap, old_capacity: Int, capacity: Int) -> SlotMap {
    if capacity == old_capacity {
        old
    } else {
        slots_grown(old, old_capacity, capacity - 1).insert(capacity - 1, Excl(None))
    }
}

/// B5: transfer full byte-allocation authority through native realloc/alloc.
/// Only the allocator representation/preservation is trusted, not any bytes
/// growth policy, region splitting, refcount or handle invariant.
#[trusted]
#[requires(base.invariant())]
#[requires(capabilities.inner_logic().0.invariant() && capabilities.inner_logic().1.invariant())]
#[requires(base@ == Some((capabilities.inner_logic().0.namespace(), old_capacity@, 0int)))]
#[requires(capabilities.inner_logic().0.capacity() == old_capacity@ && capabilities.inner_logic().1.capacity() == old_capacity@)]
#[requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0 && capabilities.inner_logic().1.hi() == old_capacity@)]
#[requires(old_capacity < capacity && capacity@ <= isize::MAX@)]
#[ensures(result.0.invariant())]
#[ensures(result.1.inner_logic().0.invariant() && result.1.inner_logic().1.invariant())]
#[ensures(result.0@ == Some((result.1.inner_logic().0.namespace(), capacity@, 0int)))]
#[ensures(result.1.inner_logic().0.capacity() == capacity@ && result.1.inner_logic().1.capacity() == capacity@)]
#[ensures(result.1.inner_logic().1.namespace() == result.1.inner_logic().0.namespace())]
#[ensures(result.1.inner_logic().1.resource_id() == result.1.inner_logic().0.namespace())]
#[ensures(result.1.inner_logic().1.lo() == 0 && result.1.inner_logic().1.hi() == capacity@)]
#[ensures(forall<index: Int> 0 <= index && index < old_capacity@ ==>
    result.1.inner_logic().1.slot(index) == capabilities.inner_logic().1.slot(index))]
#[ensures(forall<index: Int> old_capacity@ <= index && index < capacity@ ==>
    result.1.inner_logic().1.slot(index) == Some(None))]
pub(crate) unsafe fn reallocate_bound(base: BoundPtr, old_capacity: usize, capacity: usize,
    capabilities: Ghost<(Recovery, PhysicalRegion)>) -> (BoundPtr, Ghost<(Recovery, PhysicalRegion)>) {
    #[cfg(not(creusot))]
    let pointer = unsafe { allocation_ops::reallocate_u8(base.pointer.as_ptr(), old_capacity, capacity) };
    #[cfg(creusot)]
    let pointer = base.pointer;
    let slots = snapshot! { slots_grown(capabilities.inner_logic().1.ledger.slots(), old_capacity@, capacity@) };
    let binding = ghost! {
        let _old = capabilities.into_inner();
        let whole = snapshot!((Some(Excl(())), *slots));
        let recovery_value = snapshot!((Some(Excl(())), FMap::empty()));
        let region_value = snapshot!((None, *slots));
        let allocation = Resource::alloc(whole).into_inner();
        let (recovery_resource, region_resource) = allocation.split(recovery_value, region_value);
        let namespace = region_resource.id_ghost();
        let capacity_int = *Int::new(capacity as i128);
        let descriptor = AllocationDesc { capacity: capacity_int, namespace };
        let recovery = Recovery { resource: recovery_resource, descriptor, _not_objective: NotObjective {} };
        let ledger = OwnedRegion::from_model_ledger(0int, capacity_int, region_resource);
        let region = PhysicalRegion { ledger, descriptor, _not_objective: NotObjective {} };
        (namespace, (recovery, region))
    };
    let (namespace, capabilities) = binding.split();
    let raw = RawAllocation { base: pointer, capacity, namespace, _not_objective: NotObjective {} };
    (raw.into_bound_ptr_at_zero().0, capabilities)
}

/// Native pointer metadata minted only by [`detach_vec`].
///
/// This wrapper has no destructor and is intentionally not cloneable or
/// constructible outside this module. The `Ghost<Id>` is erased at runtime;
/// it binds proof tokens to the metadata in verified callers.
#[allow(missing_debug_implementations)]
pub(crate) struct RawAllocation {
    base: NonNull<u8>,
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
        pearlite! {
            0 <= self@.0 && self@.0 <= isize::MAX@ &&
            self.base_address() + self@.0 <= usize::MAX@
        }
    }
}

impl RawAllocation {
    /// Numeric base address only; this does not expose pointer permission.
    #[logic(open(self))]
    pub fn base_address(self) -> Int {
        pearlite! { self.base.addr_logic()@ }
    }

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

    /// Borrow this B1 descriptor to create offset-zero pointer metadata for
    /// read boundaries while retaining the raw descriptor for a later B2
    /// resume or B3 deallocation.
    ///
    /// The returned `BoundPtr` carries the same allocation namespace and
    /// capacity binding. It is still metadata only: the Recovery and
    /// PhysicalRegion capabilities remain necessary for access or cleanup.
    #[requires(self.invariant())]
    #[ensures(result.0.invariant())]
    #[ensures(result.0@ == Some((self.namespace(), self.capacity(), 0int)))]
    #[ensures(result.0.current_address() == self.base_address())]
    #[ensures(result.1@ == self.capacity())]
    pub(crate) fn bound_ptr_at_zero(&self) -> (BoundPtr, usize) {
        #[cfg(creusot)]
        let binding = ghost! {
            let namespace = self.namespace.into_inner();
            let allocation_capacity: Int = *Int::new(self.capacity as i128);
            let absolute_offset: Int = *Int::new(0i128);
            Some(BoundPtrBinding {
                namespace,
                allocation_capacity,
                absolute_offset,
            })
        };

        #[cfg(creusot)]
        let bound = BoundPtr {
            pointer: self.base,
            binding,
        };
        #[cfg(not(creusot))]
        let bound = BoundPtr {
            pointer: self.base,
        };

        (bound, self.capacity)
    }
}

/// The proof-only descriptor paired with a native pointer word. Copying this
/// descriptor does not copy or create any recovery/region capability.
#[cfg(creusot)]
#[derive(core::clone::Clone, Copy)]
struct BoundPtrBinding {
    namespace: Id,
    allocation_capacity: Int,
    absolute_offset: Int,
}

/// A pointer-sized allocation descriptor. The `Some` binding is derived only
/// from B1's `RawAllocation`, by either its consuming conversion or borrowed
/// offset-zero accessor; unbound metadata carries no physical authority. The
/// ghost field disappears from native builds.
#[allow(missing_debug_implementations)]
#[cfg_attr(not(creusot), repr(transparent))]
#[derive(core::clone::Clone, Copy)]
pub(crate) struct BoundPtr {
    pointer: NonNull<u8>,
    #[cfg(creusot)]
    binding: Ghost<Option<BoundPtrBinding>>,
}

#[cfg(creusot)]
impl View for BoundPtr {
    type ViewTy = Option<(Id, Int, Int)>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            match self.binding.inner_logic() {
                None => None,
                Some(binding) => Some((
                    binding.namespace,
                    binding.allocation_capacity,
                    binding.absolute_offset,
                )),
            }
        }
    }
}

#[cfg(creusot)]
impl Invariant for BoundPtr {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            match self@ {
                None => true,
                Some((_, allocation_capacity, absolute_offset)) =>
                    0 <= absolute_offset && absolute_offset <= allocation_capacity &&
                    allocation_capacity <= isize::MAX@ &&
                    absolute_offset < self.current_address() &&
                    self.current_address() + allocation_capacity - absolute_offset
                        <= usize::MAX@
            }
        }
    }
}

impl BoundPtr {
    /// Numeric address only; callers still need separate ownership and access
    /// permissions before dereferencing the pointer.
    #[logic(open(self))]
    pub fn current_address(self) -> Int {
        pearlite! { self.pointer.addr_logic()@ }
    }

    /// Recover the raw pointer carried by this sealed descriptor.
    ///
    /// This relates descriptor metadata to its native pointer word but grants
    /// no permission to dereference it.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub(crate) fn raw_pointer(self) -> *mut u8 {
        pearlite! { self.pointer@ }
    }

    /// Wrap pointer metadata without a B1 allocation binding or access token.
    #[ensures(result@ == None)]
    #[ensures(result.invariant())]
    pub(crate) fn unbound(pointer: NonNull<u8>) -> Self {
        #[cfg(creusot)]
        {
            Self { pointer, binding: ghost!(None) }
        }
        #[cfg(not(creusot))]
        {
            Self { pointer }
        }
    }

    /// Return pointer metadata only. Dereferencing the result still requires
    /// an independent unsafe permission; this method grants none.
    #[ensures(!result.is_null_logic())]
    #[cfg_attr(creusot, ensures(result == self.raw_pointer()))]
    pub(crate) fn as_ptr(&self) -> *mut u8 {
        self.pointer.as_ptr()
    }

    /// Return the stored non-null pointer metadata without changing its
    /// binding. This does not grant any memory permission.
    #[ensures(result.invariant())]
    pub(crate) fn as_non_null(&self) -> NonNull<u8> {
        self.pointer
    }

    /// B6: recover numeric offset from two sealed descriptors of one allocation.
    ///
    /// The native representation stores only a pointer. Its ghost binding is
    /// minted by B1 and preserved by the body-proved offset operations, so a
    /// matching allocation namespace/capacity and offset-zero base determine
    /// this difference. The current abstract invariant omits that relational
    /// address fact; this narrow physical metadata bridge supplies it.
    ///
    /// No pointer is dereferenced or reconstructed, no allocation liveness or
    /// read/write authority is granted, and equal numeric addresses alone do
    /// not satisfy the same-namespace requirement. Empty and one-past views
    /// are supported without the provenance requirements of `offset_from`.
    #[trusted]
    #[requires(self.invariant() && self@ != None)]
    #[requires(base.invariant() && base@ != None)]
    #[requires(self@.unwrap_logic().0 == base@.unwrap_logic().0)]
    #[requires(self@.unwrap_logic().1 == base@.unwrap_logic().1)]
    #[requires(base@.unwrap_logic().2 == 0)]
    #[ensures(result@ == self@.unwrap_logic().2)]
    pub(crate) fn offset_from_bound_base(&self, base: Self) -> usize {
        self.pointer.as_ptr().addr() - base.pointer.as_ptr().addr()
    }

    /// Move metadata back within the same sealed allocation. This retains the
    /// original pointer provenance and grants no liveness or byte permission.
    #[requires(self.invariant() && self@ != None)]
    #[requires(count@ <= self@.unwrap_logic().2)]
    #[ensures(result.invariant())]
    #[ensures(result@ == Some((self@.unwrap_logic().0, self@.unwrap_logic().1,
        self@.unwrap_logic().2 - count@)))]
    #[ensures(result.current_address() == self.current_address() - count@)]
    pub(crate) fn retreat_within(&self, count: usize) -> Self {
        let offset = 0usize.wrapping_sub(count);
        let pointer = crate::provenance_specs::wrapping_offset(self.pointer.as_ptr(), offset);
        // SAFETY: the sealed offset never reaches below its non-null base.
        let pointer = unsafe { NonNull::new_unchecked(pointer) };
        #[cfg(creusot)]
        let binding = ghost! {
            let count_int: Int = *Int::new(count as i128);
            match self.binding.into_inner() {
                None => None,
                Some(binding) => Some(BoundPtrBinding {
                    namespace: binding.namespace,
                    allocation_capacity: binding.allocation_capacity,
                    absolute_offset: binding.absolute_offset - count_int,
                }),
            }
        };
        #[cfg(creusot)]
        { Self { pointer, binding } }
        #[cfg(not(creusot))]
        { Self { pointer } }
    }

    /// Advance this bound descriptor within its original allocation.
    ///
    /// The pointer word is updated from its existing provenance with
    /// provenance-preserving wrapping arithmetic. The sealed allocation
    /// namespace and capacity remain fixed; only the absolute offset changes.
    /// This is metadata arithmetic only and creates no memory permission.
    #[requires(self.invariant())]
    #[requires(self@ != None)]
    #[requires(match self@ {
        Some((_, allocation_capacity, absolute_offset)) =>
            absolute_offset + count@ <= allocation_capacity,
        None => false,
    })]
    #[ensures(result.invariant())]
    #[ensures(match (self@, result@) {
        (Some((namespace, allocation_capacity, absolute_offset)),
         Some((result_namespace, result_capacity, result_offset))) =>
            namespace == result_namespace &&
            allocation_capacity == result_capacity &&
            result_offset == absolute_offset + count@,
        _ => false,
    })]
    #[ensures(result.current_address() == self.current_address() + count@)]
    pub(crate) fn advance_within(&self, count: usize) -> Self {
        let pointer = crate::provenance_specs::wrapping_offset(self.pointer.as_ptr(), count);
        // SAFETY: the B1 range invariant and the bound-offset precondition
        // keep this pointer within the original non-null allocation, including
        // its one-past position.
        let pointer = unsafe { NonNull::new_unchecked(pointer) };

        #[cfg(creusot)]
        let binding = ghost! {
            let count_int: Int = *Int::new(count as i128);
            match self.binding.into_inner() {
                None => None,
                Some(binding) => Some(BoundPtrBinding {
                    namespace: binding.namespace,
                    allocation_capacity: binding.allocation_capacity,
                    absolute_offset: binding.absolute_offset + count_int,
                }),
            }
        };

        #[cfg(creusot)]
        { Self { pointer, binding } }
        #[cfg(not(creusot))]
        { Self { pointer } }
    }
}

// The Creusot build models `Ghost` as nonzero-sized, so it omits
// `repr(transparent)`. In the native build the Ghost field is absent, and
// these compile-time checks enforce the one-word pointer layout.
#[cfg(not(creusot))]
const _: [(); core::mem::size_of::<NonNull<u8>>()] =
    [(); core::mem::size_of::<BoundPtr>()];
#[cfg(not(creusot))]
const _: [(); core::mem::align_of::<NonNull<u8>>()] =
    [(); core::mem::align_of::<BoundPtr>()];

impl RawAllocation {
    /// Consume B1's raw descriptor into an offset-zero bound pointer.
    ///
    /// This consuming form complements `bound_ptr_at_zero(&self)`, which keeps
    /// the raw descriptor available for B2 resume. The native pointer word
    /// remains metadata; callers still need Recovery and complete
    /// PhysicalRegion capabilities for any deallocation or access.
    #[requires(self.invariant())]
    #[ensures(result.0.invariant())]
    #[ensures(result.0@ == Some((self.namespace(), self.capacity(), 0int)))]
    #[ensures(result.0.current_address() == self.base_address())]
    #[ensures(result.1@ == self.capacity())]
    pub(crate) fn into_bound_ptr_at_zero(self) -> (BoundPtr, usize) {
        let RawAllocation {
            base,
            capacity,
            namespace,
            _not_objective: _,
        } = self;
        #[cfg(creusot)]
        let binding = ghost! {
            let namespace = namespace.into_inner();
            let allocation_capacity: Int = *Int::new(capacity as i128);
            let absolute_offset: Int = *Int::new(0i128);
            Some(BoundPtrBinding { namespace, allocation_capacity, absolute_offset })
        };
        #[cfg(not(creusot))]
        let _ = namespace;

        #[cfg(creusot)]
        let bound = BoundPtr { pointer: base, binding };
        #[cfg(not(creusot))]
        let bound = BoundPtr { pointer: base };
        (bound, capacity)
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

/// Sealed accumulator for retired physical-region resources from one B1
/// allocation. Its map domain may be any subset of `[0, capacity)`; unlike a
/// `PhysicalRegion`, it need not be contiguous. The unit resource used to seed
/// a pool carries no byte liveness or recovery authority.
#[allow(missing_debug_implementations)]
pub(crate) struct PhysicalPool {
    descriptor: AllocationDesc,
    resource: Resource<KernelRA>,
    _not_objective: NotObjective,
}

impl View for PhysicalPool {
    type ViewTy = (Int, Id, KernelRA, Id);

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            (self.descriptor.capacity, self.descriptor.namespace,
             self.resource@, self.resource.id())
        }
    }
}

impl Invariant for PhysicalPool {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            0 <= self@.0 &&
            self@.2.0 == None &&
            self@.3 == self@.1 &&
            forall<index: Int> self@.2.1.contains(index) ==>
                0 <= index && index < self@.0
        }
    }
}

impl PhysicalPool {
    #[logic(open)]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self@.0 }
    }

    #[logic(open)]
    pub(crate) fn namespace(self) -> Id {
        pearlite! { self@.1 }
    }

    #[logic(open)]
    pub(crate) fn resource_id(self) -> Id {
        pearlite! { self@.3 }
    }

    #[logic(open)]
    pub(crate) fn contains(self, index: Int) -> bool {
        pearlite! { self@.2.1.contains(index) }
    }

    #[logic(open)]
    pub(crate) fn slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self@.2.1.get(index) {
                None => None,
                Some(Excl(value)) => Some(value),
            }
        }
    }

    /// Expose the product resource's mathematical unit as the empty-map
    /// resource value. The proof uses only the existing RA neutral law and
    /// finite-map extensional composition.
    #[check(ghost)]
    #[ensures(KernelRA::unit() == (None, FMap::empty()))]
    fn kernel_unit_is_empty() {
        let unit: Snapshot<KernelRA> = snapshot!(KernelRA::unit());
        let empty_map: Snapshot<SlotMap> = snapshot!(FMap::empty());
        let unit_map: Snapshot<SlotMap> = snapshot!((*unit).1);
        let empty_value: Snapshot<KernelRA> = snapshot!((None, FMap::empty()));
        ghost! {
            proof_assert!(forall<index: Int>
                (*empty_map).get(index).op((*unit_map).get(index)) ==
                    Some((*unit_map).get(index))
            );
            map_compose_eq(empty_map, unit_map, unit_map);
            proof_assert!((*empty_value).op(*unit) == Some(*unit));
            proof_assert!((*empty_value).op(*unit) == Some(*empty_value));
        };
    }

    /// Create an empty retired-resource pool tied to an existing sealed
    /// physical region. This copies only the logical descriptor and mints the
    /// RA unit; it does not extract or duplicate the region's Resource.
    #[check(ghost)]
    #[requires(region.invariant())]
    #[ensures(result.invariant())]
    #[ensures(result.capacity() == region.capacity())]
    #[ensures(result.namespace() == region.namespace())]
    #[ensures(result.resource_id() == region.resource_id())]
    #[ensures(forall<index: Int> !result.contains(index))]
    pub(crate) fn empty_from(region: &PhysicalRegion) -> Self {
        ghost! { Self::kernel_unit_is_empty(); };
        let descriptor = region.descriptor;
        let resource: Resource<KernelRA> = Resource::new_unit(descriptor.namespace);
        Self {
            descriptor,
            resource,
            _not_objective: NotObjective {},
        }
    }

    /// Retire one affine physical region into this allocation's pool.
    ///
    /// The consumed region's Resource is joined into the pool Resource. The
    /// result owns the exact union of both domains and preserves every slot,
    /// including Unknown spare bytes.
    #[check(ghost)]
    #[requires(self.invariant() && region.invariant())]
    #[requires(self.namespace() == region.namespace())]
    #[requires(self.capacity() == region.capacity())]
    #[requires(self.resource_id() == region.resource_id())]
    #[requires(forall<index: Int> self.contains(index) ==>
        !(region.lo() <= index && index < region.hi()))]
    #[ensures(result.invariant())]
    #[ensures(result.namespace() == self.namespace())]
    #[ensures(result.capacity() == self.capacity())]
    #[ensures(result.resource_id() == self.resource_id())]
    #[ensures(forall<index: Int> result.contains(index) ==
        (self.contains(index) || region.lo() <= index && index < region.hi()))]
    #[ensures(forall<index: Int> result.slot(index) ==
        if self.contains(index) { self.slot(index) }
        else if region.lo() <= index && index < region.hi() { region.slot(index) }
        else { None })]
    pub(crate) fn retire(self, region: PhysicalRegion) -> Self {
        let PhysicalRegion { ledger, descriptor: _, _not_objective: _ } = region;
        let (region_lo, region_hi, region_resource) = ledger.into_parts();
        let mut pool_resource = self.resource;
        pool_resource.valid_op_lemma(&region_resource);
        let pool_map: Snapshot<SlotMap> = snapshot!(pool_resource@.1);
        let region_map: Snapshot<SlotMap> = snapshot!(region_resource@.1);
        let resource = pool_resource.join(region_resource);
        let joined_map: Snapshot<SlotMap> = snapshot!(resource@.1);
        ghost! {
            proof_assert!((*pool_map).op(*region_map) == Some(*joined_map));
            map_op_get(pool_map, region_map, joined_map);
        };
        let result = Self {
            descriptor: self.descriptor,
            resource,
            _not_objective: NotObjective {},
        };
        let _ = (region_lo, region_hi);
        result
    }

    /// Recover a full-range physical region once every allocation slot has
    /// returned to this pool. Unknown values are preserved unchanged; no
    /// recovery marker or byte-access capability is minted here.
    #[check(ghost)]
    #[requires(self.invariant())]
    #[requires(lo == 0)]
    #[requires(forall<index: Int> self.contains(index) ==
        (0 <= index && index < self.capacity()))]
    #[ensures(result.invariant())]
    #[ensures(result.capacity() == self.capacity())]
    #[ensures(result.namespace() == self.namespace())]
    #[ensures(result.resource_id() == self.resource_id())]
    #[ensures(result.lo() == lo && result.hi() == self.capacity())]
    #[ensures(forall<index: Int> result.slot(index) == self.slot(index))]
    pub(crate) fn finish(self, lo: Int) -> PhysicalRegion {
        let descriptor = self.descriptor;
        let capacity = descriptor.capacity;
        let ledger = OwnedRegion::from_model_ledger(lo, capacity, self.resource);
        PhysicalRegion {
            ledger,
            descriptor,
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
/// separate B4 bridge. Its invariant also records that the allocation's base
/// address plus byte capacity fits in the `usize` address range, which permits
/// exact non-wrapping arithmetic for later bound metadata offsets. This adds
/// no live-range or dereference permission.
#[trusted]
#[ensures(result.0.invariant())]
#[ensures(result.1@ == input@.len())]
#[cfg_attr(creusot, ensures(
    result.0.capacity() == creusot_std::std::vec::capacity_model(input)
))]
#[cfg_attr(creusot, ensures(
    result.0.base_address() == creusot_std::std::vec::base_model(input)
))]
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
    // SAFETY: `Vec::as_mut_ptr` is non-null and aligned even when capacity is
    // zero. This is part of B1's existing trusted Vec-to-allocation boundary.
    let base = unsafe { NonNull::new_unchecked(base) };

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
#[cfg_attr(creusot, ensures(
    creusot_std::std::vec::capacity_model(result) == raw.capacity()
))]
#[cfg_attr(creusot, ensures(
    creusot_std::std::vec::base_model(result) == raw.base_address()
))]
pub(crate) unsafe fn resume_vec(
    raw: RawAllocation,
    len: usize,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Vec<u8> {
    let RawAllocation { base, capacity, namespace: _, _not_objective: _ } = raw;
    let _ = capabilities;
    // SAFETY: The caller promises exact B1 capabilities and a Known prefix;
    // the B1 allocation descriptor supplies the original base/capacity.
    unsafe { Vec::from_raw_parts(base.as_ptr(), len, capacity) }
}

/// Explicitly free a B1 allocation after recovering its complete authority.
///
/// This is B3, a trusted native boundary. It directly deallocates the byte
/// allocation through the same native helper as ordinary SharedBuffer cleanup. The
/// original global allocator and `u8` layout are retained by the sealed B1
/// descriptor. No slot needs to be Known: byte deallocation
/// does not inspect initialized or spare slots.
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
    #[cfg(not(creusot))]
    unsafe { allocation_ops::deallocate_u8(base.as_ptr(), capacity); }
}

/// Deallocate through a sealed offset-zero [`BoundPtr`] descriptor.
///
/// Unlike `deallocate_vec`, this B3 variant gets the native base pointer from
/// the pointer-sized descriptor. An unbound descriptor (`None`) cannot satisfy
/// the precondition. The descriptor itself is copyable metadata only; the
/// consumed Recovery and full PhysicalRegion carry the unique authority.
///
/// # Safety
///
/// The descriptor must be the offset-zero binding minted from B1 for this
/// exact allocation, and the recovery/region capabilities must be its matching
/// full, unborrowed allocation authority.
#[trusted]
#[requires(bound.invariant())]
#[requires(capabilities.inner_logic().0.invariant())]
#[requires(capabilities.inner_logic().1.invariant())]
#[requires(bound@ == Some((capabilities.inner_logic().0.namespace(), capacity@, 0int)))]
#[requires(capabilities.inner_logic().0.capacity() == capacity@)]
#[requires(capabilities.inner_logic().1.capacity() == capacity@)]
#[requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace())]
#[requires(capabilities.inner_logic().1.lo() == 0)]
#[requires(capabilities.inner_logic().1.hi() == capacity@)]
pub(crate) unsafe fn deallocate_bound_vec(
    bound: BoundPtr,
    capacity: usize,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) {
    let base = bound.as_ptr();
    let _ = capabilities;
    // SAFETY: the sealed descriptor maps this pointer to the original B1 base,
    // and the full affine capabilities are consumed by this B3 boundary.
    #[cfg(not(creusot))]
    unsafe { allocation_ops::deallocate_u8(base, capacity); }
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
    let pointer = unsafe { raw.base.as_ptr().add(start) };
    let _ = region;
    // SAFETY: B1 sealed the original allocation metadata; the contract proves
    // this nonempty interval is inside the uniquely borrowed initialized region.
    unsafe { core::slice::from_raw_parts_mut(pointer, len) }
}

/// A byte slot contains a known initialized value exactly when both layers
/// of its optional ledger entry are present.
#[logic(open)]
pub(crate) fn slot_known(slot: Option<Option<u8>>) -> bool {
    match slot { Some(Some(_)) => true, _ => false }
}

#[check(ghost)]
#[ensures(slot_known(slot) == (exists<value: u8> slot == Some(Some(value))))]
pub(crate) fn slot_known_equivalence(slot: Option<Option<u8>>) {}

/// Borrow the initialized prefix at a sealed bound pointer's current offset.
///
/// B4-bound physical boundary: the private binding denotes the original B1
/// allocation and this exact provenance-preserving offset. The pointer is
/// obtained directly from the descriptor, never supplied or matched by address.
/// A nonempty requested interval is exclusively owned by `region`, so it also
/// supplies allocation liveness. For length zero the native pointer needs only
/// to be nonnull and aligned (u8); no allocation-liveness fact is asserted.
///
/// The slice and mutable region borrow have the same lifetime. On return of
/// that borrow, only the selected slots acquire the slice's final values;
/// every other slot, including Unknown spare capacity, is preserved.
#[trusted]
#[requires(bound.invariant() && bound@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(bound@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(bound@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <= region.inner_logic().hi())]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    slot_known(region.inner_logic().slot(bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@ && (^result)@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    region.inner_logic().slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
#[ensures((^region.inner_logic()).invariant())]
#[ensures((^region.inner_logic()).namespace() == region.inner_logic().namespace())]
#[ensures((^region.inner_logic()).capacity() == region.inner_logic().capacity())]
#[ensures((^region.inner_logic()).lo() == region.inner_logic().lo())]
#[ensures((^region.inner_logic()).hi() == region.inner_logic().hi())]
#[ensures((^region.inner_logic()).resource_id() == region.inner_logic().resource_id())]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    (^region.inner_logic()).slot(bound@.unwrap_logic().2 + offset) == Some(Some((^result)@[offset])))]
#[ensures(forall<index: Int>
    !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
        (^region.inner_logic()).slot(index) == region.inner_logic().slot(index))]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    slot_known((^region.inner_logic()).slot(bound@.unwrap_logic().2 + offset)))]
pub(crate) unsafe fn borrow_bound_mut<'a>(
    bound: &'a BoundPtr,
    len: usize,
    region: Ghost<&'a mut PhysicalRegion>,
) -> &'a mut [u8] {
    let _ = region;
    // No pointer arithmetic occurs here. Even an empty/stale descriptor grants
    // no positive-byte access without the matching initialized region token.
    unsafe { core::slice::from_raw_parts_mut(bound.as_ptr(), len) }
}

/// B4-uninit: exclusive u8 storage viewed through the standard MaybeUninit model.
///
/// Known(v) corresponds to Some(v), Unknown to None. The returned mutable
/// borrow writes back those exact Option values and frames every other slot.
/// A copied sealed descriptor grants no liveness: only a nonempty owned region
/// supports positive-length access. Zero length needs only nonnull alignment.
/// The by-value descriptor avoids tying the slice to a temporary metadata copy;
/// its lifetime is tied to the exclusive mutable PhysicalRegion borrow.
#[trusted]
#[requires(bound.invariant() && bound@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(bound@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(bound@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <= region.inner_logic().hi())]
#[ensures(result@.len() == len@ && (^result)@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    region.inner_logic().slot(bound@.unwrap_logic().2 + offset) == Some(result@[offset]@))]
#[ensures((^region.inner_logic()).invariant())]
#[ensures((^region.inner_logic()).namespace() == region.inner_logic().namespace())]
#[ensures((^region.inner_logic()).capacity() == region.inner_logic().capacity())]
#[ensures((^region.inner_logic()).lo() == region.inner_logic().lo())]
#[ensures((^region.inner_logic()).hi() == region.inner_logic().hi())]
#[ensures((^region.inner_logic()).resource_id() == region.inner_logic().resource_id())]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    (^region.inner_logic()).slot(bound@.unwrap_logic().2 + offset) == Some((^result)@[offset]@))]
#[ensures(forall<index: Int>
    !(bound@.unwrap_logic().2 <= index && index < bound@.unwrap_logic().2 + len@) ==>
        (^region.inner_logic()).slot(index) == region.inner_logic().slot(index))]
pub(crate) unsafe fn borrow_bound_uninit_mut<'a>(
    bound: BoundPtr,
    len: usize,
    region: Ghost<&'a mut PhysicalRegion>,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    let _ = region;
    // The sealed descriptor supplies the pointer; the affine region supplies
    // positive-byte liveness/exclusivity. MaybeUninit<u8> has u8 layout.
    unsafe { core::slice::from_raw_parts_mut(bound.as_ptr().cast(), len) }
}

/// Body-proved spare access: B4 frames the initialized visible prefix.
/// This adds no physical-access axiom; only the final spare slice uses B4.
#[requires(base.invariant() && base@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(base@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(base@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= base@.unwrap_logic().2)]
#[requires(base@.unwrap_logic().2 + capacity@ <= region.inner_logic().hi())]
#[requires(visible <= capacity)]
#[requires(forall<index: Int> 0 <= index && index < visible@ ==>
    slot_known(region.inner_logic().slot(base@.unwrap_logic().2 + index)))]
#[ensures(result@.len() == capacity@ - visible@ && (^result)@.len() == capacity@ - visible@)]
#[ensures((^region.inner_logic()).invariant())]
#[ensures((^region.inner_logic()).namespace() == region.inner_logic().namespace())]
#[ensures((^region.inner_logic()).capacity() == region.inner_logic().capacity())]
#[ensures((^region.inner_logic()).lo() == region.inner_logic().lo())]
#[ensures((^region.inner_logic()).hi() == region.inner_logic().hi())]
#[ensures((^region.inner_logic()).resource_id() == region.inner_logic().resource_id())]
#[ensures(forall<index: Int> 0 <= index && index < visible@ ==>
    slot_known((^region.inner_logic()).slot(base@.unwrap_logic().2 + index)))]
#[ensures(forall<index: Int> visible@ <= index && index < capacity@ ==>
    region.inner_logic().slot(base@.unwrap_logic().2 + index) == Some(result@[index - visible@]@))]
#[ensures(forall<index: Int> visible@ <= index && index < capacity@ ==>
    (^region.inner_logic()).slot(base@.unwrap_logic().2 + index) == Some((^result)@[index - visible@]@))]
#[ensures(forall<index: Int>
    !(base@.unwrap_logic().2 + visible@ <= index && index < base@.unwrap_logic().2 + capacity@) ==>
    (^region.inner_logic()).slot(index) == region.inner_logic().slot(index))]
pub(crate) unsafe fn borrow_spare_preserving_prefix<'a>(
    base: BoundPtr,
    visible: usize,
    capacity: usize,
    region: Ghost<&'a mut PhysicalRegion>,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    let bound = base.advance_within(visible);
    unsafe { borrow_bound_uninit_mut(bound, capacity - visible, region) }
}

/// B4-read: shared initialized access tied to a shared borrow of the exclusive
/// region. The region cannot be mutated, retired or recovered until all such
/// borrows end. Sealed namespace/range matching is the same as B4-bound; empty
/// reads establish no allocation liveness and perform no pointer arithmetic.
/// Ghost classification permits observation only: this shared slice cannot
/// write slots or initialize storage. Mutable B4 bridges remain ordinary
/// program operations, so erased ghost writes cannot update their slot ledger.
#[trusted]
#[requires(bound.invariant() && bound@ != None)]
#[requires(region.inner_logic().invariant())]
#[requires(bound@.unwrap_logic().0 == region.inner_logic().namespace())]
#[requires(bound@.unwrap_logic().1 == region.inner_logic().capacity())]
#[requires(region.inner_logic().lo() <= bound@.unwrap_logic().2)]
#[requires(bound@.unwrap_logic().2 + len@ <= region.inner_logic().hi())]
#[requires(forall<offset: Int> 0 <= offset && offset < len@ ==>
    slot_known(region.inner_logic().slot(bound@.unwrap_logic().2 + offset)))]
#[ensures(result@.len() == len@)]
#[ensures(forall<offset: Int> 0 <= offset && offset < len@ ==>
    region.inner_logic().slot(bound@.unwrap_logic().2 + offset) == Some(Some(result@[offset])))]
#[cfg_attr(creusot, check(ghost))]
pub(crate) unsafe fn borrow_bound<'a>(
    bound: &'a BoundPtr,
    len: usize,
    region: Ghost<&'a PhysicalRegion>,
) -> &'a [u8] {
    let _ = region;
    unsafe { core::slice::from_raw_parts(bound.as_ptr(), len) }
}

/// Fixed-zero shared slice access. Non-null u8 metadata needs no allocation
/// liveness or initialized slot when the extent is exactly zero. Its ghost
/// classification is observation-only: the returned slice contains no bytes.
#[trusted]
#[requires(bound.invariant())]
#[ensures(result@.len() == 0)]
#[cfg_attr(creusot, check(ghost))]
pub(crate) unsafe fn borrow_empty_bound<'a>(bound: &'a BoundPtr) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts(bound.as_ptr(), 0) }
}

/// Fixed-zero mutable slice access, tied to a shared metadata borrow.
/// Empty slices own no bytes; the immutable descriptor grants no access authority.
#[trusted]
#[requires(bound.invariant())]
#[ensures(result@.len() == 0 && (^result)@.len() == 0)]
pub(crate) unsafe fn borrow_empty_bound_mut<'a>(bound: &'a BoundPtr) -> &'a mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(bound.as_ptr(), 0) }
}

/// The MaybeUninit<u8> zero extent has the same one-byte alignment requirement.
#[trusted]
#[requires(bound.invariant())]
#[ensures(result@.len() == 0 && (^result)@.len() == 0)]
pub(crate) unsafe fn borrow_empty_bound_uninit_mut<'a>(
    bound: &'a BoundPtr,
) -> &'a mut [core::mem::MaybeUninit<u8>] {
    unsafe { core::slice::from_raw_parts_mut(bound.as_ptr().cast(), 0) }
}
