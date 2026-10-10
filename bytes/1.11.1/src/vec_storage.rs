//! Opaque native storage for a Vec-backed shared allocation.
//!
//! This type keeps the original `Vec<u8>` value and allocation in place while
//! the bytes crate exposes disjoint raw-pointer views. It deliberately has no
//! `Deref`, slice conversion, or public vector access. The internal transfer
//! methods are currently native runtime operations; their B1/B2/B3 proof-side
//! authority contracts are still being developed.

use alloc::vec::Vec;
use core::{mem::ManuallyDrop, ptr::NonNull};

#[cfg(all(creusot, bytes_original_unique_gate))]
use crate::ownership_proof::raw_vec::{self, PhysicalRegion, RawAllocation, Recovery};
#[cfg(all(creusot, bytes_original_unique_gate))]
use creusot_std::prelude::*;
#[cfg(all(creusot, bytes_original_unique_gate))]
use creusot_std::logic::Id;

/// Native parts transferred from an owned Vec without leaving a live Vec
/// value beside the raw-pointer view.
///
/// The proof-only fields are minted by the generic B1 boundary from this
/// exact native tuple. With zero capacity they describe only an empty logical
/// interval and recovery identity: they grant no dereference permission and
/// do not assert that an allocator block is live.
#[allow(missing_debug_implementations)]
pub(crate) struct VecRawParts {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    raw: RawAllocation,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Invariant for VecRawParts {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.raw.invariant() &&
            self.raw.raw_pointer() == self.native_ptr_model() &&
            self.raw.base_address() == self.native_ptr_model().addr_logic()@ &&
            self.raw.capacity() == self.native_capacity_model() &&
            self.native_len_model() <= self.native_capacity_model() &&
            self.native_capacity_model() <= isize::MAX@ &&
            self.raw.base_address() + self.native_capacity_model() <= usize::MAX@ &&
            self.capabilities.inner_logic().0.invariant() &&
            self.capabilities.inner_logic().1.invariant() &&
            self.capabilities.inner_logic().0.capacity() == self.native_capacity_model() &&
            self.capabilities.inner_logic().1.capacity() == self.native_capacity_model() &&
            self.capabilities.inner_logic().0.namespace() == self.raw.namespace() &&
            self.capabilities.inner_logic().1.namespace() == self.raw.namespace() &&
            self.capabilities.inner_logic().1.resource_id() == self.raw.namespace() &&
            self.capabilities.inner_logic().1.lo() == 0 &&
            self.capabilities.inner_logic().1.hi() == self.native_capacity_model() &&
            forall<i: Int> 0 <= i && i < self.native_capacity_model() ==>
                self.capabilities.inner_logic().1.slot(i) == self.region_slot_model(i)
        }
    }
}

/// One-use, sealed witness created only while consuming a native Vec.
/// `raw_vec` can turn it into the logical B1 ledger, but no caller can forge
/// one from an arbitrary pointer/length/capacity tuple.
#[cfg(all(creusot, bytes_original_unique_gate))]
#[allow(missing_debug_implementations)]
pub(crate) struct VecPartsWitness {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    contents: Snapshot<Seq<u8>>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl VecPartsWitness {
    #[logic(open(self))]
    pub(crate) fn pointer(self) -> *mut u8 {
        pearlite! { self.ptr@ }
    }

    #[logic(open(self))]
    pub(crate) fn length(self) -> Int {
        pearlite! { self.len@ }
    }

    #[logic(open(self))]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self.cap@ }
    }

    #[logic(open(self))]
    pub(crate) fn contents(self) -> Seq<u8> {
        pearlite! { *self.contents }
    }

    #[ensures(result.0@ == self.pointer())]
    #[ensures(result.1@ == self.length())]
    #[ensures(result.2@ == self.capacity())]
    #[requires(self.invariant())]
    #[ensures(result.1@ <= result.2@)]
    #[ensures(result.2@ <= isize::MAX@)]
    #[ensures(result.0.addr_logic()@ == self.pointer().addr_logic()@)]
    #[ensures(result.0.addr_logic()@ + result.2@ <= usize::MAX@)]
    pub(crate) fn native_parts(&self) -> (NonNull<u8>, usize, usize) {
        (self.ptr, self.len, self.cap)
    }

    #[ensures(*result == self.contents())]
    #[requires(self.invariant())]
    #[ensures(self.length() == (*result).len())]
    pub(crate) fn contents_snapshot(&self) -> Snapshot<Seq<u8>> {
        self.contents
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Invariant for VecPartsWitness {
    #[logic(open(self))]
    fn invariant(self) -> bool {
        pearlite! {
            0 <= self.length() &&
            self.length() == self.contents().len() &&
            self.length() <= self.capacity() &&
            self.capacity() <= isize::MAX@ &&
            self.ptr.addr_logic()@ + self.capacity() <= usize::MAX@
        }
    }
}

impl VecRawParts {
    /// Consume a Vec into its exact native pointer, length, and capacity.
    ///
    /// This generic B1 boundary binds the shared-storage path to a
    /// Vec-backed allocation: it consumes the original Vec, suppresses its
    /// destructor, and binds the exact `as_mut_ptr`/`len`/`capacity` tuple to
    /// one recovery token and the full slot region. The known prefix matches
    /// the Vec model and spare capacity is Unknown. At capacity zero, the
    /// returned non-null dangling pointer is metadata only and the region is
    /// empty; no allocation liveness or byte access is claimed. The native
    /// extraction body is shared by production and verification builds. The
    /// generic B1 interpretation assumes Vec's default global allocator and
    /// `u8` layout, `capacity <= isize::MAX`, a nonwrapping base-plus-capacity
    /// address range, and exclusive ownership transferred by consuming Vec.
    /// The resulting affine region prevents overlapping mutable access; the
    /// helper does not create permissions from pointer equality alone.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), trusted)]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.native_ptr_model() == creusot_std::std::vec::pointer_model(input)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.native_len_model() == input@.len()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.native_capacity_model() == creusot_std::std::vec::capacity_model(input)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.native_len_model() <= result.native_capacity_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.native_capacity_model() <= isize::MAX@
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.raw_invariant_model()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.raw_pointer_model() == result.native_ptr_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.raw_base_address_model() == creusot_std::std::vec::base_model(input)
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.raw_base_address_model() == result.native_ptr_model().addr_logic()@
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.raw_capacity_model() == result.native_capacity_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.recovery_invariant_model() && result.region_invariant_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.recovery_capacity_model() == result.native_capacity_model() &&
        result.region_capacity_model() == result.native_capacity_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.recovery_namespace_model() == result.raw_namespace_model() &&
        result.region_namespace_model() == result.raw_namespace_model() &&
        result.region_resource_id_model() == result.raw_namespace_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.region_lo_model() == 0 &&
        result.region_hi_model() == result.native_capacity_model()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        0 <= i && i < result.native_capacity_model() ==>
            result.region_slot_model(i) ==
                if i < result.native_len_model() { Some(Some(input@[i])) } else { Some(None) }
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.invariant()))]
    pub(crate) fn take(input: Vec<u8>) -> Self {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let input_model = snapshot!(input@);

        let mut input = ManuallyDrop::new(input);
        let ptr = input.as_mut_ptr();
        let len = input.len();
        let cap = input.capacity();
        // SAFETY: Vec::as_mut_ptr is non-null even when capacity is zero.
        // For zero capacity this is the dangling metadata pointer and the
        // empty region below carries no dereference or allocation authority.
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        #[cfg(all(creusot, bytes_original_unique_gate))]
        let witness = VecPartsWitness { ptr, len, cap, contents: input_model };
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let (raw, capabilities) = raw_vec::bind_consumed_vec(witness);

        Self {
            ptr,
            len,
            cap,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            raw,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            capabilities,
        }
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.native_ptr_model()))]
    pub(crate) fn native_ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.native_len_model()))]
    pub(crate) fn logical_len(&self) -> usize {
        self.len
    }

    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.native_capacity_model()))]
    pub(crate) fn logical_capacity(&self) -> usize {
        self.cap
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[requires(self.invariant())]
    #[ensures(result.0.raw_pointer() == self.native_ptr_model())]
    #[ensures(result.0.base_address() == self.raw_base_address_model())]
    #[ensures(result.0.capacity() == self.native_capacity_model())]
    #[ensures(result.0.invariant())]
    #[ensures(result.1.inner_logic().0@ == self.recovery_model())]
    #[ensures(result.1.inner_logic().1@ == self.region_model())]
    #[ensures(result.1.inner_logic().0.invariant() && result.1.inner_logic().1.invariant())]
    #[ensures(result.1.inner_logic().0.capacity() == self.native_capacity_model() && result.1.inner_logic().1.capacity() == self.native_capacity_model())]
    #[ensures(result.1.inner_logic().0.namespace() == result.0.namespace())]
    #[ensures(result.1.inner_logic().1.namespace() == result.0.namespace())]
    #[ensures(result.1.inner_logic().1.resource_id() == result.0.namespace())]
    #[ensures(result.1.inner_logic().1.lo() == 0 && result.1.inner_logic().1.hi() == self.native_capacity_model())]
    #[ensures(self.native_len_model() <= self.native_capacity_model() && self.native_capacity_model() <= isize::MAX@)]
    #[ensures(result.0.base_address() + self.native_capacity_model() <= usize::MAX@)]
    #[ensures(forall<i:Int> 0 <= i && i < self.native_capacity_model() ==>
        result.1.inner_logic().1.slot(i) == self.region_slot_model(i))]
    pub(crate) fn into_proof_authority(
        self,
    ) -> (RawAllocation, Ghost<(Recovery, PhysicalRegion)>) {
        (self.raw, self.capabilities)
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn native_ptr_model(self) -> *mut u8 {
        pearlite! { self.ptr@ }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn native_len_model(self) -> Int {
        pearlite! { self.len@ }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn native_capacity_model(self) -> Int {
        pearlite! { self.cap@ }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn raw_invariant_model(self) -> bool {
        pearlite! { self.raw.invariant() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn raw_pointer_model(self) -> *mut u8 {
        pearlite! { self.raw.raw_pointer() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn raw_base_address_model(self) -> Int {
        pearlite! { self.raw.base_address() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn raw_capacity_model(self) -> Int {
        pearlite! { self.raw.capacity() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn raw_namespace_model(self) -> Id {
        pearlite! { self.raw.namespace() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn recovery_invariant_model(self) -> bool {
        pearlite! { self.capabilities.inner_logic().0.invariant() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self), prophetic)]
    pub(crate) fn region_invariant_model(self) -> bool {
        pearlite! { self.capabilities.inner_logic().1.invariant() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn recovery_capacity_model(self) -> Int {
        pearlite! { self.capabilities.inner_logic().0.capacity() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_capacity_model(self) -> Int {
        pearlite! { self.capabilities.inner_logic().1.capacity() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn recovery_namespace_model(self) -> Id {
        pearlite! { self.capabilities.inner_logic().0.namespace() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_namespace_model(self) -> Id {
        pearlite! { self.capabilities.inner_logic().1.namespace() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_resource_id_model(self) -> Id {
        pearlite! { self.capabilities.inner_logic().1.resource_id() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_lo_model(self) -> Int {
        pearlite! { self.capabilities.inner_logic().1.lo() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_hi_model(self) -> Int {
        pearlite! { self.capabilities.inner_logic().1.hi() }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_slot_model(self, index: Int) -> Option<Option<u8>> {
        pearlite! { self.capabilities.inner_logic().1.slot(index) }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn recovery_model(self) -> <Recovery as View>::ViewTy {
        pearlite! { self.capabilities.inner_logic().0@ }
    }

    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[logic(open(self))]
    pub(crate) fn region_model(self) -> <PhysicalRegion as View>::ViewTy {
        pearlite! { self.capabilities.inner_logic().1@ }
    }

    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    pub(crate) fn discard_unmodeled(self) {}
}

/// A native Vec allocation retained behind a non-accessible storage handle.
///
/// `repr(transparent)` preserves the size, alignment, and ABI of the wrapped
/// `ManuallyDrop<Vec<u8>>`; `ManuallyDrop` itself has the layout of `Vec<u8>`.
/// The wrapper is private to the crate and does not expose initialized bytes.
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub(crate) struct VecStorage(ManuallyDrop<Vec<u8>>);

impl Default for VecStorage {
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn default() -> Self {
        Self::hold(Vec::new())
    }
}

impl Drop for VecStorage {
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn drop(&mut self) {
        // The wrapper's normal terminal path is the native Vec destructor.
        // Shared's proof-side drop contract must establish that no packet or
        // borrow remains before this generic storage retirement takes place.
        unsafe { ManuallyDrop::drop(&mut self.0) }
    }
}

#[cfg(not(creusot))]
const _: [(); core::mem::size_of::<Vec<u8>>()] = [(); core::mem::size_of::<VecStorage>()];
#[cfg(not(creusot))]
const _: [(); core::mem::align_of::<Vec<u8>>()] = [(); core::mem::align_of::<VecStorage>()];

impl VecStorage {
    /// Retain a Vec without exposing it to ordinary Vec operations.
    ///
    /// The matching physical allocation authority must be established by the
    /// B1 bridge before this value is used as shared storage.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn hold(vec: Vec<u8>) -> Self {
        Self(ManuallyDrop::new(vec))
    }

    /// Observe only the native Vec pointer and capacity metadata.
    ///
    /// No initialized slice or reference to the Vec is returned. The raw
    /// pointer grants no byte access by itself; access still requires the
    /// matching physical region capability.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn metadata(&mut self) -> (*mut u8, usize, usize) {
        let vec = &mut *self.0;
        (vec.as_mut_ptr(), vec.len(), vec.capacity())
    }

    /// Mutate Vec length after the caller has physically initialized the
    /// entire newly exposed interval and holds the matching exclusive region.
    ///
    /// # Safety
    ///
    /// `len` must not exceed capacity and every element in `0..len` must be
    /// initialized. The caller must also exclude every overlapping view.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn set_len(&mut self, len: usize) {
        unsafe { (&mut *self.0).set_len(len) }
    }

    /// Reserve Vec capacity while the caller exclusively owns the allocation.
    ///
    /// # Safety
    ///
    /// No pointer or region capability may be used across a possible
    /// reallocation. The caller must repair or replace all allocation-relative
    /// metadata after this call.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn reserve(&mut self, additional: usize) {
        (&mut *self.0).reserve(additional)
    }

    /// Transfer the hidden Vec to the generic B2 restoration boundary.
    ///
    /// # Safety
    ///
    /// The caller must have recovered all physical and lifetime authority for
    /// this allocation. It must use the returned Vec only after proving its
    /// requested initialized prefix and capacity. This method moves the Vec
    /// value; it does not rebind a stale Vec model after raw-pointer writes.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn take_for_restore(self) -> Vec<u8> {
        let mut storage = ManuallyDrop::new(self);
        // SAFETY: `storage` owns an initialized VecStorage and is itself
        // ManuallyDrop, so taking its sole Vec field transfers that owner
        // without running VecStorage's destructor.
        unsafe { ManuallyDrop::take(&mut storage.0) }
    }

    /// Explicitly transfer the retained Vec to a B2/B3 ownership boundary.
    ///
    /// # Safety
    ///
    /// The caller must own the matching Recovery and complete unborrowed
    /// PhysicalRegion and must invoke the exact generic B3 deallocator once.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn take_for_retirement(self) -> Vec<u8> {
        let mut storage = ManuallyDrop::new(self);
        // SAFETY: same ownership transfer as `take_for_restore`; the caller
        // must dispose the returned Vec exactly once through the B3 boundary.
        unsafe { ManuallyDrop::take(&mut storage.0) }
    }
}
