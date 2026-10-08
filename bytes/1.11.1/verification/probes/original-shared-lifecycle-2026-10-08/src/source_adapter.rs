//! Source-correspondent leaf for the original `bytes::Shared` allocation.
//!
//! `source_map.py` checks the exact production Shared fields, selected
//! `From<Vec<u8>>` branch, clone increment, and explicit cleanup bodies. Under
//! the default non-loom/non-extra-platforms configuration, the production
//! `loom::sync::atomic::AtomicUsize` alias is `core::sync::atomic::AtomicUsize`,
//! which is the field type used here. Public vtable dispatch and automatic Drop
//! are outside this leaf.

use alloc::{alloc::{dealloc, Layout}, boxed::Box};
use core::sync::atomic::{AtomicPtr, AtomicUsize};
use creusot_std::{
    ghost::{
        lifetime_logic::{EndBorrow, FullBorrow, Lifetime, LifetimeToken},
        perm::Perm,
        GhostShared,
    },
    logic::{Id, Int, real::PositiveReal},
    prelude::*,
    std::sync::{
        atomic::{AtomicUsize as ModelAtomic, ordering::{Acquire, None as NoStore, Relaxed, Release}},
        committer::Committer,
        view::{ReleaseSyncView, SyncView},
    },
};

use crate::{boxed_alignment, bounded, field_event, pointer_event, raw_vec};
use bounded::RecoveryPayload as _;

/// Exact production field order and field types from bytes.rs::Shared.
pub(crate) struct Shared {
    pub(crate) buf: *mut u8,
    pub(crate) cap: usize,
    pub(crate) ref_cnt: AtomicUsize,
}

impl field_event::AtomicField for Shared {
    #[cfg_attr(creusot, ensures(field_event::atomic_model(result) == self.field_model()))]
    fn atomic_field(&self) -> &AtomicUsize { &self.ref_cnt }

    #[cfg(creusot)]
    #[logic(open(crate))]
    fn field_model(&self) -> creusot_std::std::sync::atomic::AtomicUsize {
        field_event::atomic_model(&self.ref_cnt)
    }
}

// The source `Shared` has a Drop implementation for ordinary Box destruction.
// `free_shared` deliberately bypasses that destructor and performs the byte
// allocation deallocation itself before releasing the typed control box.
#[cfg(not(creusot))]
impl Drop for Shared {
    fn drop(&mut self) {
        unsafe { dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap()) }
    }
}

/// The actual `Bytes` field types and order from bytes.rs. The vtable value is
/// supplied by the caller because this leaf proves the concrete handle fields
/// and their pointer relation, while dynamic callback selection remains open.
/// Keeping `Vtable` opaque here prevents this adapter from suggesting that it
/// has verified the callback bodies or `SHARED_VTABLE` identity.
pub(crate) struct Bytes {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
    pub(crate) data: AtomicPtr<()>,
    pub(crate) vtable: &'static Vtable,
}

pub(crate) struct Vtable {
    _opaque: (),
}

#[cfg(not(creusot))]
const _: [(); 0] = [(); core::mem::needs_drop::<AtomicUsize>() as usize];

/// Bounded recovery payload tying the original Box permission and B1 physical
/// allocation to the same full lifetime used by the actual ref-count field.
/// The only copies are GhostShared descriptors; all allocation authority stays
/// in the two EndBorrow values until the final Acquire recovery.
pub(crate) struct Payload {
    recovery: raw_vec::Recovery,
    physical_end: EndBorrow<raw_vec::PhysicalRegion>,
    control_end: EndBorrow<field_event::OwnedControl<Shared>>,
    physical: Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    control: Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    base: raw_vec::BoundPtr,
    capacity: usize,
    len: usize,
    expected: Snapshot<Seq<u8>>,
}

impl bounded::RecoveryPayload for Payload {
    type Metadata = (Lifetime, Id, Int, Seq<u8>, Int, *const Shared, *mut u8);

    #[logic]
    fn metadata(self) -> Self::Metadata {
        (
            self.control.val().lft(),
            self.recovery.namespace(),
            self.recovery.capacity(),
            *self.expected,
            pearlite! { self.control.val().cur().pointer().addr_logic()@ },
            self.control.val().cur().pointer(),
            self.base.raw_pointer(),
        )
    }

    #[logic(prophetic)]
    fn wellformed(self) -> bool {
        pearlite! {
            self.recovery.invariant() && self.base.invariant() &&
            self.base@ == Some((self.recovery.namespace(), self.recovery.capacity(), 0int)) &&
            self.capacity@ == self.recovery.capacity() && self.len@ == (*self.expected).len() &&
            self.capacity@ > 0 &&
            self.len@ <= self.capacity@ &&
            self.physical_end.lft() == self.physical.val().lft() &&
            self.physical.val().lft() == self.control.val().lft() &&
            ^self.physical_end == self.physical.val().cur() &&
            self.physical.val().cur().invariant() &&
            self.physical.val().cur().namespace() == self.recovery.namespace() &&
            self.physical.val().cur().capacity() == self.recovery.capacity() &&
            self.physical.val().cur().resource_id() == self.recovery.namespace() &&
            self.physical.val().cur().lo() == 0 &&
            self.physical.val().cur().hi() == self.recovery.capacity() &&
            self.control_end.lft() == self.control.val().lft() &&
            ^self.control_end == self.control.val().cur() &&
            self.control.val().cur().wellformed() &&
            self.control.val().cur().owner.val().buf == self.base.raw_pointer() &&
            self.control.val().cur().owner.val().cap@ == self.recovery.capacity() &&
            self.control.val().cur().model() ==
                field_event::atomic_model(&self.control.val().cur().owner.val().ref_cnt) &&
            (forall<i: Int> 0 <= i && i < self.len@ ==>
                self.physical.val().cur().slot(i) == Some(Some((*self.expected)[i])))
        }
    }
}

/// The source-correspondent handle keeps actual Bytes fields together with
/// proof-only shared descriptors. No exclusive Shared permission remains in a
/// handle once the full control lease is created.
pub(crate) struct OriginalSharedHandle {
    pub(crate) bytes: Bytes,
    pub(crate) shared: *mut Shared,
    bound: raw_vec::BoundPtr,
    capacity: usize,
    control: Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    physical: Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    invariant: Ghost<GhostShared<field_event::FieldInvariant<bounded::State<Payload>>>>,
    ticket: Ghost<bounded::Ticket<Payload>>,
    pub(crate) data_binding: Ghost<pointer_event::ReadOnlyPointer>,
}

impl OriginalSharedHandle {
    /// Tie the four actual Bytes fields, actual Shared address and field
    /// descriptor to the same B1 allocation and bounded lifecycle ticket.
    #[logic(prophetic)]
    pub(crate) fn valid(self) -> bool {
        pearlite! {
            self.bound.invariant() &&
            self.bound@ == Some((self.invariant.inner_logic().val().public().4.1,
                self.capacity@, 0int)) &&
            self.capacity@ == self.invariant.inner_logic().val().public().4.2 &&
            self.bytes.len@ == self.invariant.inner_logic().val().public().4.3.len() &&
            self.capacity@ > 0 &&
            self.bytes.len@ <= self.capacity@ &&
            self.bytes.ptr == self.bound.raw_pointer() as *const u8 &&
            self.bound.current_address() == self.bytes.ptr.addr_logic()@ &&
            self.data_binding.inner_logic().model() ==
                pointer_event::pointer_model(&self.bytes.data) &&
            self.data_binding.inner_logic().value() == self.shared as *mut () &&
            self.shared as *const Shared == self.control.inner_logic().val().cur().pointer() &&
            self.invariant.inner_logic().val().public().4.5 == self.shared as *const Shared &&
            self.invariant.inner_logic().val().public().4.6 == self.bound.raw_pointer() &&
            self.invariant.inner_logic().val().public().4.0 ==
                self.invariant.inner_logic().val().public().3 &&
            self.control.inner_logic().val().lft() == self.invariant.inner_logic().val().public().3 &&
            self.physical.inner_logic().val().lft() == self.invariant.inner_logic().val().public().3 &&
            self.control.inner_logic().val().cur().wellformed() &&
            self.control.inner_logic().val().cur().model() == self.invariant.inner_logic().val().model() &&
            self.control.inner_logic().val().cur().owner.val().buf == self.bound.raw_pointer() &&
            self.control.inner_logic().val().cur().owner.val().cap@ == self.capacity@ &&
            self.physical.inner_logic().val().cur().invariant() &&
            self.physical.inner_logic().val().cur().namespace() == self.bound@.unwrap_logic().0 &&
            self.physical.inner_logic().val().cur().capacity() == self.capacity@ &&
            self.physical.inner_logic().val().cur().resource_id() == self.bound@.unwrap_logic().0 &&
            self.physical.inner_logic().val().cur().lo() == 0 &&
            self.physical.inner_logic().val().cur().hi() == self.capacity@ &&
            (forall<i: Int> 0 <= i && i < self.bytes.len@ ==>
                self.physical.inner_logic().val().cur().slot(i) ==
                    Some(Some(self.invariant.inner_logic().val().public().4.3[i]))) &&
            self.ticket.inner_logic().token.lft() == self.invariant.inner_logic().val().public().3 &&
            self.ticket.inner_logic().valid(self.invariant.inner_logic().val().public())
        }
    }

    /// Borrow the exact initialized byte prefix under this handle's live
    /// physical fraction. The returned Rust lifetime prevents this same
    /// handle's cleanup from ending the B4 borrow while the slice is in use.
    #[requires(self.valid())]
    #[ensures(result@ == self.invariant.inner_logic().val().public().4.3)]
    pub(crate) fn borrow_contents<'a>(&'a self) -> &'a [u8] {
        let slice = if self.bytes.len == 0 {
            // SAFETY: zero-length B4 access needs only the bound metadata.
            unsafe { raw_vec::borrow_empty_bound(&self.bound) }
        } else {
            let region = ghost! {
                let full: &FullBorrow<raw_vec::PhysicalRegion> = (*self.physical).to_ref();
                full.borrow(&(*self.ticket).token)
            };
            // SAFETY: valid() ties the token, region, pointer, len and Known
            // slots to the same B1 allocation.
            unsafe { raw_vec::borrow_bound(&self.bound, self.bytes.len, region) }
        };
        proof_assert!(slice@.ext_eq(self.invariant.inner_logic().val().public().4.3));
        slice
    }

    /// Copy convenience used by the explicit source cleanup tests.
    #[requires(self.valid())]
    #[ensures(result@ == self.invariant.inner_logic().val().public().4.3)]
    pub(crate) fn read_contents(&self) -> alloc::vec::Vec<u8> {
        self.borrow_contents().to_vec()
    }

    /// Body-proved explicit `free_shared` cleanup after `Pending::recover`.
    /// Both exact provenance pointers are checked against the recovered
    /// payload before its EndBorrow values return allocation permissions.
    #[requires(recovered.inner_logic().0.wellformed())]
    #[requires(recovered.inner_logic().1.frac() == PositiveReal::from_int(1))]
    #[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.control.val().lft())]
    #[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.physical.val().lft())]
    #[requires(recovered.inner_logic().0.control.val().cur().pointer() == pointer as *const Shared)]
    #[requires(recovered.inner_logic().0.base.raw_pointer() == bound.raw_pointer())]
    #[requires(bound@ == recovered.inner_logic().0.base@)]
    #[requires(recovered.inner_logic().0.capacity@ == capacity@)]
    #[requires(capacity@ > 0)]
    pub(crate) fn free_recovered_shared(
        pointer: *mut Shared,
        bound: raw_vec::BoundPtr,
        capacity: usize,
        recovered: Ghost<(Payload, LifetimeToken)>,
    ) {
        let (owner, capabilities) = ghost! {
            let (payload, lifetime) = recovered.into_inner();
            let dead = lifetime.end();
            let control = payload.control_end.get(dead);
            let physical = payload.physical_end.get(dead);
            (control.owner.into_inner(), (payload.recovery, physical))
        }.split();

        // Match free_shared's actual reads from the recovered Shared block.
        let permission: Ghost<&Perm<*const Shared>> = ghost! { &**owner };
        let shared_ref = unsafe { Perm::as_ref(pointer, permission) };
        proof_assert!(shared_ref.buf == bound.raw_pointer());
        proof_assert!(shared_ref.cap@ == capacity@);

        // SAFETY: Pending::recover supplied the matching B1 Recovery and the
        // full physical region, and the typed permission names this Shared.
        unsafe {
            raw_vec::deallocate_bound_vec(bound, shared_ref.cap, capabilities);
            deallocate_typed_box(pointer, owner);
        }
    }

    /// Selected source leaf for shallow_clone_arc. This uses the actual
    /// Bytes.data load and actual Shared.ref_cnt RMW, then creates the actual
    /// second Bytes fields and its read-only data binding.
    #[requires(self.valid())]
    #[requires(quota.inner_logic().valid(self.invariant.inner_logic().val().public().2))]
    #[ensures(result.valid())]
    #[ensures(result.invariant.inner_logic().val().public() ==
        self.invariant.inner_logic().val().public())]
    #[ensures(result.shared == self.shared && result.bytes.ptr == self.bytes.ptr &&
        result.bytes.len == self.bytes.len && result.bytes.vtable == self.bytes.vtable)]
    pub(crate) fn shallow_clone_arc(&self, quota: Ghost<bounded::CloneQuota>) -> Self {
        let shared_word = pointer_event::load_relaxed(&self.bytes.data, self.data_binding.borrow());
        let mut current = ghost! { SyncView::new().into_inner() };
        let release = ghost! { ReleaseSyncView::new().into_inner() };
        let mut new_ticket = ghost! { None::<bounded::Ticket<Payload>> };
        let source = ghost! { &*self.ticket };
        let control_full = ghost! { (*self.control).to_ref() };
        let invariant = ghost! { (*self.invariant).to_ref() };
        let old_count = field_event::increment_owned::<Shared, bounded::State<Payload>, _>(
            shared_word.cast::<Shared>(),
            control_full,
            ghost! { &(*self.ticket).token },
            invariant,
            ghost! {|state: &mut bounded::State<Payload>, committer: &mut Committer<ModelAtomic, usize, Relaxed, Relaxed>| {
                let _ = bounded::State::observe_atomic(Ghost::new(&*state));
                *new_ticket = Some(bounded::State::on_clone(
                    Ghost::new(state),
                    Ghost::new(committer),
                    source,
                    quota,
                    current.borrow_mut(),
                    release,
                ).into_inner());
            }},
        );
        proof_assert!(old_count == 1usize);
        // Exact source guard from bytes.rs::shallow_clone_arc. The bounded B
        // transition establishes old_count == 1, so this generic abort branch
        // is unreachable in the selected one-clone source domain.
        if old_count > usize::MAX >> 1 {
            abort_on_refcount_overflow();
        }

        let mut pointer_view = ghost! { SyncView::new().into_inner() };
        let (new_data, new_data_permission) =
            pointer_event::new_pointer(shared_word, pointer_view.borrow_mut());
        let bytes = Bytes {
            ptr: self.bytes.ptr,
            len: self.bytes.len,
            data: new_data,
            vtable: self.bytes.vtable,
        };
        let data_binding = pointer_event::bind_read_only(
            &bytes.data,
            shared_word,
            new_data_permission,
        );
        let new_ticket = ghost! { new_ticket.into_inner().unwrap() };
        Self {
            bytes,
            shared: self.shared,
            bound: self.bound,
            capacity: self.capacity,
            control: self.control,
            physical: self.physical,
            invariant: self.invariant,
            ticket: new_ticket,
            data_binding,
        }
    }

    /// One explicit source `release_shared` leaf: Release fetch_sub, early
    /// return for a nonlast owner, and on last owner the actual Acquire load
    /// before B3 byte deallocation and typed Shared allocation deallocation.
    #[requires(self.valid())]
    #[ensures(result.1.inner_logic().valid::<Payload>(
        self.invariant.inner_logic().val().public(), result.0))]
    pub(crate) fn release_one(self) -> (bool, Ghost<bounded::Receipt>) {
        self.release_one_observed(ghost! { None::<&bounded::Ticket<Payload>> })
    }

    /// Source leaf used when a sibling handle is known live. The observation
    /// is ghost-only and strengthens this same Release event; it adds no load
    /// and does not move the peer ticket.
    #[requires(self.valid())]
    #[requires(peer.inner_logic().valid(self.invariant.inner_logic().val().public()))]
    #[ensures(!result.0)]
    #[ensures(result.1.inner_logic().valid::<Payload>(
        self.invariant.inner_logic().val().public(), result.0))]
    pub(crate) fn release_one_with_live_peer(
        self,
        peer: Ghost<&bounded::Ticket<Payload>>,
    ) -> (bool, Ghost<bounded::Receipt>) {
        self.release_one_observed(ghost! { Some(peer.into_inner()) })
    }

    #[requires(self.valid())]
    #[requires(peer.inner_logic() == None ||
        peer.inner_logic().unwrap_logic().valid(self.invariant.inner_logic().val().public()))]
    #[ensures(peer.inner_logic() == None || !result.0)]
    #[ensures(result.1.inner_logic().valid::<Payload>(
        self.invariant.inner_logic().val().public(), result.0))]
    fn release_one_observed(
        mut self,
        peer: Ghost<Option<&bounded::Ticket<Payload>>>,
    ) -> (bool, Ghost<bounded::Receipt>) {
        // Production `shared_drop` reaches the control block through
        // `Bytes.data.with_mut`; select that same actual AtomicPtr field here.
        let shared = pointer_event::get_mut(&mut self.bytes.data, self.data_binding.borrow())
            .cast::<Shared>();
        proof_assert!(shared == self.shared);

        let public = snapshot!(self.invariant.inner_logic().val().public());
        let (mut current, retiring) = bounded::prepare(self.ticket, public);
        let (lease, rest) = bounded::Retiring::split_token(retiring).split();
        let mut pending = ghost! { None::<bounded::Pending<Payload>> };
        let mut receipt = ghost! { None::<bounded::Receipt> };
        let control_full = ghost! { (*self.control).to_ref() };
        let invariant = ghost! { (*self.invariant).to_ref() };
        let old_count = field_event::decrement_owned::<Shared, bounded::State<Payload>, _>(
            shared,
            control_full,
            lease,
            invariant,
            ghost! {|state: &mut bounded::State<Payload>,
                     committer: &mut Committer<ModelAtomic, usize, Relaxed, Release>,
                     token: LifetimeToken| {
                let _ = bounded::State::observe_atomic(Ghost::new(&*state));
                let retiring = bounded::RetiringRest::with_token(rest, Ghost::new(token));
                let (collected, done) = match peer.into_inner() {
                    Some(peer_ticket) => bounded::State::on_release_with_live_peer(
                        Ghost::new(state),
                        Ghost::new(committer),
                        retiring,
                        current.borrow_mut(),
                        Ghost::new(peer_ticket),
                    ).into_inner(),
                    None => bounded::State::on_release(
                        Ghost::new(state),
                        Ghost::new(committer),
                        retiring,
                        current.borrow_mut(),
                    ).into_inner(),
                };
                *pending = collected;
                *receipt = Some(done);
            }},
        );

        if old_count == 1 {
            // For the negative semantic mutant, omit both the native Acquire
            // and its event callback. Native code ignores this proof-only
            // feature and always retains the production operation below.
            #[cfg(any(not(creusot), not(feature = "negative_source_no_acquire")))]
            {
                let acquire_lease = ghost! {
                    pending.as_ref().unwrap().borrow_token_for(public, snapshot!(*current))
                };
                field_event::acquire_owned::<Shared, bounded::State<Payload>, _>(
                    shared,
                    control_full,
                    acquire_lease,
                    invariant,
                    ghost! {|state: &mut bounded::State<Payload>,
                             committer: &Committer<ModelAtomic, usize, Acquire, NoStore>| {
                        let _ = bounded::State::observe_atomic(Ghost::new(&*state));
                        bounded::State::on_acquire(
                            Ghost::new(state),
                            Ghost::new(committer),
                            Ghost::new(pending.as_ref().unwrap()),
                            current.borrow_mut(),
                        );
                    }},
                );
            }

            let recovered = bounded::Pending::recover(
                ghost! { pending.into_inner().unwrap() },
                public,
                current,
            );
            Self::free_recovered_shared(shared, self.bound, self.capacity, recovered);
        }

        let was_last = old_count == 1;
        let receipt = ghost! { receipt.into_inner().unwrap() };
        (was_last, receipt)
    }
}

/// Generic typed Box deallocation boundary for the explicit source cleanup.
/// It consumes the exact permission for the supplied allocation and applies
/// `Layout::new::<T>()`; it asserts no lastness or reference-count fact.
#[trusted]
#[requires(*owner.inner_logic().ward() == pointer as *const T)]
pub(crate) unsafe fn deallocate_typed_box<T>(
    pointer: *mut T,
    owner: Ghost<Box<Perm<*const T>>>,
) {
    let _ = owner;
    // SAFETY: the consumed typed Box permission names this allocation.
    unsafe { dealloc(pointer.cast(), Layout::new::<T>()) };
}

/// Generic diverging boundary matching the library's overflow-abort branch.
/// The preceding `old_count == 1` fact makes the exact source guard false;
/// this generic boundary conveys only that abort does not return if invoked.
#[trusted]
#[ensures(false)]
#[cold]
fn abort_on_refcount_overflow() {
    std::process::abort()
}

/// Compose two independently executed original release leaves and reconcile
/// their authenticated receipts after the final Shared allocation is gone.
/// B's bounded source domain has exactly one initial owner and one clone.
#[requires(first.valid() && second.valid())]
#[requires(first.invariant.inner_logic().val().public() == second.invariant.inner_logic().val().public())]
#[ensures(result.0 != result.1)]
pub(crate) fn release_both(
    first: OriginalSharedHandle,
    second: OriginalSharedHandle,
) -> (bool, bool) {
    let public = snapshot!(first.invariant.inner_logic().val().public());
    let (first_last, first_receipt) = first.release_one();
    let (second_last, second_receipt) = second.release_one();
    bounded::reconcile::<Payload>(public, first_last, first_receipt, second_last, second_receipt);
    (first_last, second_last)
}

/// Keep one B4 slice live while the peer handle executes its nonfinal Release
/// event, use the slice after that cleanup call, then end the borrow before
/// the last-owner Acquire and reclamation. This is the positive counterpart
/// to `negative_release_while_borrowed` below.
#[requires(reader.valid() && peer.valid())]
#[requires(reader.invariant.inner_logic().val().public() == peer.invariant.inner_logic().val().public())]
#[ensures(result@ == reader.invariant.inner_logic().val().public().4.3)]
pub(crate) fn borrow_across_peer_release(
    reader: OriginalSharedHandle,
    peer: OriginalSharedHandle,
) -> alloc::vec::Vec<u8> {
    let public = snapshot!(reader.invariant.inner_logic().val().public());
    let slice = reader.borrow_contents();
    let (peer_last, peer_receipt) =
        peer.release_one_with_live_peer(ghost! { &*reader.ticket });
    proof_assert!(!peer_last);
    let copied = slice.to_vec();
    let (reader_last, reader_receipt) = reader.release_one();
    bounded::reconcile::<Payload>(public, peer_last, peer_receipt, reader_last, reader_receipt);
    proof_assert!(reader_last);
    copied
}

/// Expected Rust borrow-check failure: a handle cannot be consumed for its
/// last-owner cleanup while a slice borrowed from that same handle is live.
#[cfg(feature = "negative_release_while_borrowed")]
pub(crate) fn negative_release_while_borrowed(handle: OriginalSharedHandle) -> u8 {
    let slice = handle.borrow_contents();
    let _retired = handle.release_one();
    slice[0]
}

#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result.0.bytes.len@ == input@.len())]
#[ensures(result.0.capacity@ > 0)]
#[ensures(result.0.bytes.ptr == result.0.bound.raw_pointer() as *const u8)]
#[ensures(result.0.bound@ == Some((result.0.invariant.inner_logic().val().public().4.1,
    creusot_std::std::vec::capacity_model(input), 0int)))]
#[ensures(result.0.bound.current_address() == result.0.bytes.ptr.addr_logic()@)]
#[ensures(result.0.data_binding.inner_logic().model() == pointer_event::pointer_model(&result.0.bytes.data))]
#[ensures(result.0.data_binding.inner_logic().value() == result.0.shared as *mut ())]
#[ensures(result.0.bytes.vtable == vtable)]
#[ensures(result.0.ticket.inner_logic().valid(result.0.invariant.inner_logic().val().public()))]
#[ensures(result.0.invariant.inner_logic().val().public().4.3 == input@)]
#[ensures(result.0.invariant.inner_logic().val().public().4.5 == result.0.shared as *const Shared)]
#[ensures(result.0.invariant.inner_logic().val().public().4.6 == result.0.bound.raw_pointer())]
#[ensures(result.0.valid())]
#[ensures(result.1.inner_logic().valid(result.0.invariant.inner_logic().val().public().2))]
pub(crate) fn from_vec_spare_capacity(
    input: alloc::vec::Vec<u8>,
    vtable: &'static Vtable,
    mut current: Ghost<creusot_std::std::sync::view::SyncView>,
) -> (OriginalSharedHandle, Ghost<bounded::CloneQuota>) {
    // This is the selected len < cap arm of bytes.rs::From<Vec<u8>>: B1 detaches
    // the actual Vec allocation, and the actual core atomic returned by the
    // generic field constructor is moved directly into the production-shaped
    // Shared record.
    let expected = snapshot!(input@);
    let (raw, len, capabilities) = raw_vec::detach_vec(input);
    let (base, cap) = raw.into_bound_ptr_at_zero();
    let ptr = base.as_ptr();
    let (ref_cnt, count_permission) = field_event::new(1, current.borrow_mut());
    let count_view = snapshot!(*current);
    let boxed = Box::new(Shared {
        buf: ptr,
        cap,
        ref_cnt,
    });
    let (shared, shared_owner) = boxed_alignment::into_raw_aligned(boxed);
    let shared_addr = crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(
        shared_addr,
        core::mem::align_of::<Shared>(),
    );
    debug_assert_eq!(shared_addr & 1, 0);
    let (data, data_permission) = pointer_event::new_pointer(shared.cast::<()>(), current.borrow_mut());
    let bytes = Bytes {
        ptr,
        len,
        data,
        vtable,
    };
    let data_binding = pointer_event::bind_read_only(&bytes.data, shared.cast::<()>(), data_permission);

    let (recovery, region) = capabilities.split();
    let lifetime = ghost! { LifetimeToken::new() };
    let lifetime_id = snapshot!(lifetime.lft());
    let (control_full, control_end) = FullBorrow::new(
        field_event::own_control(shared, shared_owner),
        lifetime_id,
    );
    let (physical_full, physical_end) = FullBorrow::new(region, lifetime_id);
    let control = ghost! { GhostShared::new(control_full).into_inner() };
    let physical = ghost! { GhostShared::new(physical_full).into_inner() };
    let payload = ghost! {
        Payload {
            recovery: recovery.into_inner(),
            physical_end: physical_end.into_inner(),
            control_end: control_end.into_inner(),
            physical,
            control,
            base,
            capacity: cap,
            len,
            expected,
        }
    };
    let initialized = bounded::State::<Payload>::initialize(
        count_permission,
        count_view,
        payload,
        lifetime,
    );
    let (state, rest) = initialized.split();
    let (ticket, quota) = rest.split();

    // Borrow the typed owner through the exact first B-ticket, then select the
    // native field by ordinary body-proved projection before binding the
    // external protocol descriptor.
    let permission: Ghost<&Perm<*const Shared>> = ghost! {
        let full: &FullBorrow<field_event::OwnedControl<Shared>> = (*control).to_ref();
        let owner = full.borrow(&ticket.token);
        &**owner.owner
    };
    let shared_ref = unsafe { Perm::as_ref(shared, permission) };
    let field_invariant = field_event::FieldInvariant::bind(&shared_ref.ref_cnt, state);
    let invariant = GhostShared::new(field_invariant);

    let handle = OriginalSharedHandle {
        bytes,
        shared,
        bound: base,
        capacity: cap,
        control,
        physical,
        invariant,
        ticket,
        data_binding,
    };
    (handle, quota)
}

/// End-to-end bounded source path: selected From<Vec<u8>> construction, one
/// non-consuming shallow clone, a borrowed read across the peer's nonfinal
/// cleanup, then final Acquire recovery and explicit physical/control cleanup.
/// `reverse` selects which of the original and clone supplies the live reader.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn source_lifecycle_driver(
    input: alloc::vec::Vec<u8>,
    vtable: &'static Vtable,
    reverse: bool,
) -> alloc::vec::Vec<u8> {
    let (source, quota) = from_vec_spare_capacity(
        input,
        vtable,
        ghost! { SyncView::new().into_inner() },
    );
    let clone = source.shallow_clone_arc(quota);
    if reverse {
        borrow_across_peer_release(clone, source)
    } else {
        borrow_across_peer_release(source, clone)
    }
}

#[cfg(all(test, not(creusot)))]
mod native_tests {
    use super::*;
    use core::sync::atomic::Ordering;

    static VTABLE: Vtable = Vtable { _opaque: () };

    fn make(contents: &[u8]) -> (OriginalSharedHandle, Ghost<bounded::CloneQuota>) {
        let mut input = alloc::vec::Vec::with_capacity(contents.len() + 9);
        input.extend_from_slice(contents);
        assert!(input.len() < input.capacity());
        from_vec_spare_capacity(
            input,
            &VTABLE,
            ghost! { SyncView::new().into_inner() },
        )
    }

    fn count(handle: &OriginalSharedHandle) -> usize {
        // SAFETY: the selected handle owns a live ticket for this actual
        // Shared allocation. Tests call this only before final cleanup.
        unsafe { &*handle.shared }.ref_cnt.load(Ordering::Acquire)
    }

    #[test]
    fn constructor_clone_and_both_release_orders_use_actual_fields() {
        for reverse in [false, true] {
            let source_bytes = [3, 5, 8, 13];
            let (source, quota) = make(&source_bytes);
            let shared = source.shared;
            assert_eq!(count(&source), 1);
            assert_eq!(source.bytes.data.load(Ordering::Relaxed), shared.cast());

            let clone = source.shallow_clone_arc(quota);
            assert_eq!(count(&source), 2);
            assert_eq!(source.bytes.ptr, clone.bytes.ptr);
            assert_eq!(source.bytes.len, clone.bytes.len);
            assert_ne!(
                core::ptr::addr_of!(source.bytes.data),
                core::ptr::addr_of!(clone.bytes.data),
            );
            assert_eq!(source.bytes.data.load(Ordering::Relaxed), shared.cast());
            assert_eq!(clone.bytes.data.load(Ordering::Relaxed), shared.cast());

            let results = if reverse {
                release_both(clone, source)
            } else {
                release_both(source, clone)
            };
            assert_eq!(results, (false, true));
        }
    }

    #[test]
    fn borrowed_bytes_survive_peer_release_for_empty_and_spare_capacity() {
        for contents in [&[][..], &[21, 34, 55][..]] {
            for reverse in [false, true] {
                let mut input = alloc::vec::Vec::with_capacity(contents.len() + 9);
                input.extend_from_slice(contents);
                assert!(input.len() < input.capacity());
                let expected = input.clone();
                let result = source_lifecycle_driver(input, &VTABLE, reverse);
                assert_eq!(result, expected);
            }
        }
    }
}
