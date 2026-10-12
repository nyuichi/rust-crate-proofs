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
use crate::ownership_proof::raw_vec::{self, BoundPtr, PhysicalRegion, RawAllocation, Recovery};
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
    /// Compose the generic Vec capture with the body-proved B1 resource bind.
    /// The known prefix matches the Vec model and spare capacity is Unknown.
    /// At capacity zero, the returned non-null dangling pointer is metadata
    /// only and the region is empty; no allocation liveness or byte access is
    /// claimed. Native builds retain the original one-shot extraction body.
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
        {
            let (storage, witness) = VecStorage::capture_vec_owner(input);
            let (ptr, len, cap) = witness.native_parts();
            let (raw, capabilities) = raw_vec::bind_consumed_vec(witness);

            // This path transfers the retained allocation to VecRawParts.
            // Suppress VecStorage::drop: the raw pointer authority is now
            // represented by this raw-parts value.
            core::mem::forget(storage);
            Self { ptr, len, cap, raw, capabilities }
        }

        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        {
            let mut input = ManuallyDrop::new(input);
            let ptr = input.as_mut_ptr();
            let len = input.len();
            let cap = input.capacity();
            // SAFETY: Vec::as_mut_ptr is non-null and aligned even when
            // capacity is zero; in that case it remains metadata only.
            let ptr = unsafe { NonNull::new_unchecked(ptr) };
            Self { ptr, len, cap }
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
    #[requires(forall<i:Int> 0 <= i && i < self.native_len_model() ==>
        raw_vec::slot_known(self.region_slot_model(i)))]
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
    #[ensures(forall<i:Int> 0 <= i && i < self.native_len_model() ==>
        result.1.inner_logic().1.slot(i) == self.region_slot_model(i) &&
        raw_vec::slot_known(result.1.inner_logic().1.slot(i)))]
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

#[cfg(all(creusot, bytes_original_unique_gate))]
#[derive(core::clone::Clone, core::marker::Copy)]
pub(crate) enum StorageModel {
    Pending { pointer: NonNull<u8>, len: Int, capacity: Int },
    Owned { key: Id, pointer: NonNull<u8>, len: Int, capacity: Int },
    Empty { pointer: *mut u8 },
}

/// The one native Vec owner. Its proof view exposes metadata only, not Vec's
/// byte sequence, so long-lived Shared state does not pair an ordinary Vec
/// model with the separate PhysicalRegion slot model.
#[allow(missing_debug_implementations)]
#[cfg_attr(creusot, opaque)]
#[repr(transparent)]
struct NativeStoredVec(ManuallyDrop<Vec<u8>>);

#[cfg(creusot)]
impl View for NativeStoredVec {
    type ViewTy = (*mut u8, Int, Int);

    // Keep the native Vec's element model out of this storage model. The
    // capture boundary below binds this abstract metadata tuple to the exact
    // consumed Vec; later proof code reasons through the separate physical
    // slot ledger.
    #[logic(opaque)]
    fn view(self) -> Self::ViewTy {
        dead
    }
}

/// A native Vec allocation retained behind a non-accessible storage handle.
///
/// The outer ghost metadata is body-updatable. Its zero-sized field preserves
/// the layout of NativeStoredVec and hence Vec<u8>.
#[allow(missing_debug_implementations)]
#[repr(C)]
pub(crate) struct VecStorage(
    NativeStoredVec,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    Ghost<Option<StorageModel>>,
);

const _: [(); core::mem::size_of::<Vec<u8>>()] = [(); core::mem::size_of::<NativeStoredVec>()];
const _: [(); core::mem::align_of::<Vec<u8>>()] = [(); core::mem::align_of::<NativeStoredVec>()];
const _: [(); core::mem::size_of::<Vec<u8>>()] = [(); core::mem::size_of::<VecStorage>()];
const _: [(); core::mem::align_of::<Vec<u8>>()] = [(); core::mem::align_of::<VecStorage>()];

#[cfg(all(creusot, bytes_original_unique_gate))]
#[allow(missing_debug_implementations)]
pub(crate) struct StoredAuthority {
    raw: RawAllocation,
    recovery: Recovery,
    binding: Id,
    stored_len: usize,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl StoredAuthority {
    #[logic(open(self))]
    pub(crate) fn binding(self) -> Id {
        pearlite! { self.binding }
    }

    #[logic(open(self))]
    pub(crate) fn stored_len(self) -> Int {
        pearlite! { self.stored_len@ }
    }

    /// Raw pointer carried by the sealed storage allocation. This preserves
    /// pointer identity but does not expose raw-allocation access permission.
    #[logic(open(self))]
    pub(crate) fn raw_pointer(self) -> *mut u8 {
        pearlite! { self.raw.raw_pointer() }
    }

    /// Capacity of the exact allocation bound to this stored Vec.
    #[logic(open(self))]
    pub(crate) fn capacity(self) -> Int {
        pearlite! { self.raw.capacity() }
    }

    /// Numeric base address of the sealed storage allocation. This is a
    /// location fact only; it does not expose or recreate allocation access.
    #[logic(open(self))]
    pub(crate) fn base_address(self) -> Int {
        pearlite! { self.raw.base_address() }
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Invariant for StoredAuthority {
    #[logic(prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            self.raw.invariant() &&
            self.recovery.invariant() &&
            self.binding() == self.raw.namespace() &&
            self.recovery.namespace() == self.binding() &&
            self.recovery.capacity() == self.raw.capacity() &&
            0 <= self.stored_len() && self.stored_len() <= self.raw.capacity()
        }
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl VecStorage {
    #[logic(open(self))]
    pub(crate) fn storage_model(self) -> Option<StorageModel> {
        pearlite! { self.1.inner_logic() }
    }

    #[logic(open(self))]
    fn native_metadata(self) -> (*mut u8, Int, Int) {
        pearlite! { self.0@ }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn matches_authority(self, authority: StoredAuthority) -> bool {
        pearlite! {
            self.invariant() && authority.invariant() &&
            match self.storage_model() {
                Some(StorageModel::Owned { key, pointer, len, capacity }) =>
                    key == authority.binding() &&
                    pointer@ == authority.raw.raw_pointer() &&
                    len == authority.stored_len() &&
                    capacity == authority.raw.capacity(),
                _ => false,
            }
        }
    }

    #[logic(open(self))]
    fn pending_model(self) -> Option<(*mut u8, Int, Int)> {
        pearlite! {
            match self.storage_model() {
                Some(StorageModel::Pending { pointer, len, capacity }) =>
                    Some((pointer@, len, capacity)),
                _ => None,
            }
        }
    }

    /// Generic native Vec-to-storage capture. This is the only trusted
    /// operation here: the consumed Vec's exact native owner, pointer, len,
    /// capacity and contents are bound to the opaque storage field and sealed
    /// witness. It does not mint RA/region resources or assert any Bytes
    /// owner, refcount, retirement, or reclamation rule.
    #[trusted]
    #[ensures(result.0.invariant())]
    #[ensures(result.0.pending_model() == Some((
        creusot_std::std::vec::pointer_model(input),
        input@.len(),
        creusot_std::std::vec::capacity_model(input),
    )))]
    #[ensures(result.0.native_metadata() == (
        creusot_std::std::vec::pointer_model(input),
        input@.len(),
        creusot_std::std::vec::capacity_model(input),
    ))]
    #[ensures(result.1.invariant())]
    #[ensures(result.1.pointer() == creusot_std::std::vec::pointer_model(input))]
    #[ensures(result.1.pointer().addr_logic()@ == creusot_std::std::vec::base_model(input))]
    #[ensures(result.1.length() == input@.len())]
    #[ensures(result.1.capacity() == creusot_std::std::vec::capacity_model(input))]
    #[ensures(result.1.contents() == input@)]
    fn capture_vec_owner(input: Vec<u8>) -> (Self, VecPartsWitness) {
        let contents = snapshot!(input@);
        let mut input = ManuallyDrop::new(input);
        let raw_pointer = input.as_mut_ptr();
        let len = input.len();
        let capacity = input.capacity();
        // SAFETY: Vec::as_mut_ptr is non-null even at capacity zero; the
        // zero-capacity pointer remains metadata only.
        let pointer = unsafe { NonNull::new_unchecked(raw_pointer) };
        let witness = VecPartsWitness { ptr: pointer, len, cap: capacity, contents };
        let model: Ghost<Option<StorageModel>> = ghost! {
            let len_model = raw_vec::int_from_usize(len).into_inner();
            let capacity_model = raw_vec::int_from_usize(capacity).into_inner();
            Some(StorageModel::Pending {
                pointer,
                len: len_model,
                capacity: capacity_model,
            })
        };
        let storage = Self(
            NativeStoredVec(input),
            model,
        );
        (storage, witness)
    }

    /// Consume a native Vec into storage plus its separately returned B1
    /// authority. The capture boundary above relates the native owner to the
    /// sealed witness; `bind_consumed_vec` body-proves the RA/map construction.
    #[ensures(result.0.matches_authority(result.1.inner_logic().0))]
    #[ensures(result.0.native_metadata() == (
        creusot_std::std::vec::pointer_model(input),
        input@.len(),
        creusot_std::std::vec::capacity_model(input),
    ))]
    #[ensures(result.1.inner_logic().0.invariant())]
    #[ensures(result.1.inner_logic().0.raw_pointer() ==
        creusot_std::std::vec::pointer_model(input))]
    #[ensures(result.1.inner_logic().0.capacity() ==
        creusot_std::std::vec::capacity_model(input))]
    #[ensures(result.1.inner_logic().0.base_address() ==
        creusot_std::std::vec::base_model(input))]
    #[ensures(result.1.inner_logic().0.stored_len() == input@.len())]
    #[ensures(result.1.inner_logic().1.invariant())]
    #[ensures(result.1.inner_logic().1.namespace() == result.1.inner_logic().0.binding())]
    #[ensures(result.1.inner_logic().1.resource_id() == result.1.inner_logic().0.binding())]
    #[ensures(result.1.inner_logic().1.capacity() == result.1.inner_logic().0.capacity())]
    #[ensures(result.1.inner_logic().1.lo() == 0)]
    #[ensures(result.1.inner_logic().1.hi() == result.1.inner_logic().0.capacity())]
    #[ensures(result.1.inner_logic().0.base_address() + result.1.inner_logic().0.capacity() <= usize::MAX@)]
    #[ensures(result.1.inner_logic().0.capacity() <= isize::MAX@)]
    #[ensures(result.2.invariant())]
    #[ensures(result.2@ == Some((result.1.inner_logic().0.binding(),
        result.1.inner_logic().0.capacity(), 0int)))]
    #[ensures(result.2.raw_pointer() == creusot_std::std::vec::pointer_model(input))]
    #[ensures(result.2.current_address() == creusot_std::std::vec::base_model(input))]
    #[ensures(forall<i: Int> 0 <= i && i < input@.len() ==>
        result.1.inner_logic().1.slot(i) == Some(Some(input@[i])))]
    #[ensures(forall<i: Int> input@.len() <= i &&
        i < creusot_std::std::vec::capacity_model(input) ==>
        result.1.inner_logic().1.slot(i) == Some(None))]
    #[ensures(result.3.inner_logic() == (
        result.1.inner_logic().0.binding(),
        result.1.inner_logic().0.raw_pointer(),
        result.1.inner_logic().0.capacity()
    ))]
    pub(crate) fn hold_owned(
        input: Vec<u8>,
    ) -> (
        Self,
        Ghost<(StoredAuthority, PhysicalRegion)>,
        BoundPtr,
        Ghost<(Id, *mut u8, Int)>,
    ) {
        let (mut storage, witness) = Self::capture_vec_owner(input);
        let (pointer, stored_len, capacity) = witness.native_parts();
        let (raw, capabilities) = raw_vec::bind_consumed_vec(witness);
        let (base, _) = raw.bound_ptr_at_zero();
        let binding: Ghost<Id> = ghost!(raw.namespace_ghost().into_inner());
        let allocation_base = pointer.as_ptr();
        let allocation_binding: Ghost<(Id, *mut u8, Int)> = ghost! {
            (
                binding.into_inner(),
                allocation_base,
                raw_vec::int_from_usize(capacity).into_inner(),
            )
        };
        let ghost_raw = raw.into_ghost();
        storage.1 = ghost! {
            Some(StorageModel::Owned {
                key: binding.into_inner(),
                pointer,
                len: raw_vec::int_from_usize(stored_len).into_inner(),
                capacity: raw_vec::int_from_usize(capacity).into_inner(),
            })
        };
        let (recovery, region) = capabilities.split();
        let authority_and_region: Ghost<(StoredAuthority, PhysicalRegion)> = ghost! {
            (
                StoredAuthority {
                    raw: ghost_raw.into_inner(),
                    recovery: recovery.into_inner(),
                    binding: binding.into_inner(),
                    stored_len,
                },
                region.into_inner(),
            )
        };
        (storage, authority_and_region, base, allocation_binding)
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Invariant for VecStorage {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! {
            match self.storage_model() {
                None => true,
                Some(StorageModel::Pending { pointer, len, capacity }) =>
                    self.native_metadata() == (pointer@, len, capacity) &&
                    0 <= len && len <= capacity && capacity <= isize::MAX@ &&
                    pointer.addr_logic()@ + capacity <= usize::MAX@,
                Some(StorageModel::Owned { key: _, pointer, len, capacity }) =>
                    self.native_metadata() == (pointer@, len, capacity) &&
                    0 <= len && len <= capacity && capacity <= isize::MAX@ &&
                    pointer.addr_logic()@ + capacity <= usize::MAX@,
                Some(StorageModel::Empty { pointer }) =>
                    self.native_metadata() == (pointer, 0int, 0int),
            }
        }
    }
}

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
        unsafe { ManuallyDrop::drop(&mut self.0.0) }
    }
}

impl VecStorage {
    /// Retain a Vec without exposing it to ordinary Vec operations.
    ///
    /// The matching physical allocation authority must be established by the
    /// B1 bridge before this value is used as shared storage.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn hold(vec: Vec<u8>) -> Self {
        let native = NativeStoredVec(ManuallyDrop::new(vec));
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            Self(native, Ghost::new(None))
        }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        {
            Self(native)
        }
    }

    /// Observe only the native Vec pointer and capacity metadata.
    ///
    /// No initialized slice or reference to the Vec is returned. The raw
    /// pointer grants no byte access by itself; access still requires the
    /// matching physical region capability.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn metadata(&mut self) -> (*mut u8, usize, usize) {
        let vec = &mut *self.0.0;
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
        unsafe { (&mut *self.0.0).set_len(len) }
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
        (&mut *self.0.0).reserve(additional)
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
        unsafe { ManuallyDrop::take(&mut storage.0.0) }
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
        unsafe { ManuallyDrop::take(&mut storage.0.0) }
    }
}
