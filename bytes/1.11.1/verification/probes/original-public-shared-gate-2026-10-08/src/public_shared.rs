//! Selected actual public Bytes fields and Clone/From trait integration.
use alloc::{alloc::{dealloc,Layout},boxed::Box,vec::Vec};
use core::sync::atomic::{AtomicPtr,AtomicUsize};
use creusot_std::{prelude::*,ghost::{GhostShared,perm::Perm,
    lifetime_logic::{EndBorrow,FullBorrow,Lifetime,LifetimeToken}},
    logic::{Id,Int,real::PositiveReal},std::{ops::FnExt,sync::{
        atomic::{AtomicUsize as ModelAtomic,ordering::{Relaxed,Release,Acquire,None as NoStore}},
        committer::Committer,view::{ReleaseSyncView,SyncView}}}};
use crate::{boxed_alignment,field_event,pointer_event,physical_projection,raw_vec,lifecycle,erased_call};
use lifecycle::RecoveryPayload as _;
include!("../../../../src/bytes/shared_record.rs");
impl field_event::AtomicField for Shared {
    #[cfg_attr(creusot, ensures(field_event::atomic_model(result) == self.field_model()))]
    fn atomic_field(&self)->&AtomicUsize {&self.ref_cnt}
    #[cfg(creusot)] #[logic(open(crate))]
    fn field_model(&self)->ModelAtomic {field_event::atomic_model(&self.ref_cnt)}
}
// Exact native declarations are extracted from production by build.rs.
include!(concat!(env!("OUT_DIR"),"/public_records.rs"));
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

impl lifecycle::RecoveryPayload for Payload {
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


pub(crate) type CloneSpec<'a> = fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<&'a OriginalSharedProof>)->Bytes;
pub(crate) struct OriginalSharedProof {
    shared:*mut Shared,
    bound:raw_vec::BoundPtr,
    capacity:usize,
    control:Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    physical:Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    invariant:Ghost<GhostShared<field_event::FieldInvariant<lifecycle::State<Payload>>>>,
    ticket:Ghost<lifecycle::Ticket<Payload>>,
    data_binding:Ghost<pointer_event::ReadOnlyPointer>,
}
impl OriginalSharedProof {
    #[logic(prophetic)]
    pub(crate) fn valid_for(self,ptr:*const u8,len:usize,data:creusot_std::std::sync::atomic::AtomicPtr<()>)->bool {
        let p = self;
        pearlite! {
            p.bound.invariant() &&
            p.bound@ == Some((p.invariant.inner_logic().val().public().3.1,
                p.capacity@, 0int)) &&
            p.capacity@ == p.invariant.inner_logic().val().public().3.2 &&
            len@ == p.invariant.inner_logic().val().public().3.3.len() &&
            p.capacity@ > 0 &&
            len@ <= p.capacity@ &&
            ptr == p.bound.raw_pointer() as *const u8 &&
            p.bound.current_address() == ptr.addr_logic()@ &&
            p.data_binding.inner_logic().model() ==
                data &&
            p.data_binding.inner_logic().value() == p.shared as *mut () &&
            p.shared as *const Shared == p.control.inner_logic().val().cur().pointer() &&
            p.invariant.inner_logic().val().public().3.5 == p.shared as *const Shared &&
            p.invariant.inner_logic().val().public().3.6 == p.bound.raw_pointer() &&
            p.invariant.inner_logic().val().public().3.0 ==
                p.invariant.inner_logic().val().public().2 &&
            p.control.inner_logic().val().lft() == p.invariant.inner_logic().val().public().2 &&
            p.physical.inner_logic().val().lft() == p.invariant.inner_logic().val().public().2 &&
            p.control.inner_logic().val().cur().wellformed() &&
            p.control.inner_logic().val().cur().model() == p.invariant.inner_logic().val().model() &&
            p.control.inner_logic().val().cur().owner.val().buf == p.bound.raw_pointer() &&
            p.control.inner_logic().val().cur().owner.val().cap@ == p.capacity@ &&
            p.physical.inner_logic().val().cur().invariant() &&
            p.physical.inner_logic().val().cur().namespace() == p.bound@.unwrap_logic().0 &&
            p.physical.inner_logic().val().cur().capacity() == p.capacity@ &&
            p.physical.inner_logic().val().cur().resource_id() == p.bound@.unwrap_logic().0 &&
            p.physical.inner_logic().val().cur().lo() == 0 &&
            p.physical.inner_logic().val().cur().hi() == p.capacity@ &&
            (forall<i: Int> 0 <= i && i < len@ ==>
                p.physical.inner_logic().val().cur().slot(i) ==
                    Some(Some(p.invariant.inner_logic().val().public().3.3[i]))) &&
            p.ticket.inner_logic().token.lft() == p.invariant.inner_logic().val().public().2 &&
            p.ticket.inner_logic().valid(p.invariant.inner_logic().val().public())
        }
    }

}

impl Bytes {
    #[logic(prophetic)]
    pub(crate) fn original_shared_valid(self)->bool {
        pearlite! {self.original_shared.inner_logic().valid_for(self.ptr,self.len,pointer_event::pointer_model(&self.data)) &&
            self.vtable == shared_table()}
    }
    #[logic]
    pub(crate) fn original_shared_bytes(self)->Seq<u8> {
        self.original_shared.inner_logic().invariant.inner_logic().val().public().3.3
    }
}

#[logic(opaque)]
fn shared_table()->&'static Vtable {dead}

#[trusted]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(result.0 == shared_table())]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<&OriginalSharedProof>>
    result.1.inner_logic().precondition((data,ptr,len,input)) == shared_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<&OriginalSharedProof>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output) == shared_clone_checked.postcondition((data,ptr,len,input),output))]
fn shared_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {
    // The native branch names the exact extracted production getter. The
    // closed table and target/helper checks are in generated/native_bindings.rs
    // and extract_public.py. Reification/ghost erasure is explicit generic TCB.
    #[cfg(not(creusot))]
    { (original_shared_table_native(), Ghost::conjure()) }
    #[cfg(creusot)]
    { unreachable!("generic closed-table ghost-erasure reification") }
}

#[requires(source.original_shared_valid())]
#[ensures(result.original_shared_valid())]
#[ensures(result.original_shared_bytes() == source.original_shared_bytes())]
#[ensures(result.ptr == source.ptr && result.len == source.len)]
fn original_shared_clone(source:&Bytes)->Bytes {
    let native = source.vtable.clone;
    let (table,spec) = shared_registration();
    proof_assert!(source.vtable == table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),
        ghost! {&*source.original_shared},spec)
}

#[requires(input.valid_for(ptr,len,pointer_event::pointer_model(data)))]
#[ensures(result.original_shared_valid())]
#[ensures(result.original_shared_bytes() == input.invariant.inner_logic().val().public().3.3)]
#[ensures(result.ptr == ptr && result.len == len)]
fn shared_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<&OriginalSharedProof>)->Bytes {
    let shared_word=pointer_event::load_relaxed(data,ghost! {&*input.data_binding});
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared_word.cast::<Shared>(),ghost! {(*input.control).to_ref()},
        ghost! {&input.ticket.token},ghost! {(*input.invariant).to_ref()},
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*input.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared_word,pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared_word,permission);
    let (vtable,_spec)=shared_registration();
    Bytes {ptr,len,data,vtable,original_shared:ghost! {
        OriginalSharedProof {shared:input.shared,bound:input.bound,capacity:input.capacity,
            control:input.control,physical:input.physical,invariant:input.invariant,
            ticket:Ghost::new(ticket.into_inner().unwrap()),data_binding:binding}
    }}
}

/// Checked selected len<cap constructor instrumentation; native allocation,
/// record fields and data pointer correspond to the actual From<Vec> branch.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result.original_shared_valid())]
#[ensures(result.original_shared_bytes() == input@)]
fn original_shared_from_vec(input:Vec<u8>)->Bytes {
    let mut current=ghost! {SyncView::new().into_inner()};
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
    let data_binding = pointer_event::bind_read_only(&data, shared.cast::<()>(), data_permission);


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
    let initialized = lifecycle::State::<Payload>::initialize(
        count_permission,
        count_view,
        payload,
        lifetime,
    );
    let (state, rest) = initialized.split();
    let ticket = rest;

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

    let (vtable,_spec) = shared_registration();
    Bytes { ptr,len,data,vtable,original_shared:ghost! {
        OriginalSharedProof {shared,bound:base,capacity:cap,control,physical,invariant,ticket,data_binding}
    }}
}

include!(concat!(env!("OUT_DIR"),"/public_traits.rs"));

impl creusot_std::invariant::Invariant for Bytes {
    #[logic(prophetic)]
    fn invariant(self)->bool {self.original_shared_valid()}
}

    #[requires(value.original_shared_valid())]
    #[ensures(result@ == value.original_shared_bytes())]
    fn original_shared_as_slice(value:&Bytes)->&[u8] {
        if value.len == 0 {
            unsafe {physical_projection::borrow_empty(value.ptr,ghost! {&value.original_shared.bound})}
        } else {
            let region=ghost! {
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*value.original_shared.physical).to_ref();
                full.borrow(&value.original_shared.ticket.token)
            };
            unsafe {physical_projection::borrow(value.ptr,value.len,ghost! {&value.original_shared.bound},region)}
        }
    }


type DropSpec<'a> = fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<OriginalSharedProof>);

#[trusted]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(result.0 == shared_table())]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<OriginalSharedProof>>
    result.1.inner_logic().precondition((data,ptr,len,input)) == shared_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<OriginalSharedProof>>
    result.1.inner_logic().postcondition((data,ptr,len,input),()) == shared_drop_checked.postcondition((data,ptr,len,input),()))]
fn shared_drop_registration<'a>()->(&'static Vtable,Ghost<DropSpec<'a>>) {
    #[cfg(not(creusot))]
    { (original_shared_table_native(), Ghost::conjure()) }
    #[cfg(creusot)]
    { unreachable!("generic closed-table consuming ghost-erasure reification") }
}

/// The actual source cleanup method suppresses later automatic Drop. Its
/// proof branch dispatches the actual stored callback with the consumed sidecar.
#[requires(value.original_shared_valid())]
fn original_shared_cleanup(mut value:Bytes) {
    let native=value.vtable.drop;
    let (table,spec)=shared_drop_registration();
    proof_assert!(value.vtable == table);
    let proof=value.original_shared;
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),proof,spec)
}

#[requires(proof.valid_for(ptr,len,pointer_event::pointer_model(data)))]
fn shared_drop_checked(data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,proof:Ghost<OriginalSharedProof>) {
        let shared=pointer_event::get_mut(data,ghost! {&*proof.data_binding}).cast::<Shared>();
        let public=snapshot!(proof.invariant.inner_logic().val().public());
        let control=ghost! {(*proof.control).to_ref()};
        let invariant=ghost! {(*proof.invariant).to_ref()};
        let ticket=ghost! {proof.into_inner().ticket.into_inner()};
        let (mut current,retiring)=lifecycle::prepare(ticket,public);
        let (lease,rest)=lifecycle::Retiring::split_token(retiring).split();
        let mut pending=ghost! {None::<lifecycle::Pending<Payload>>};
        let old=field_event::decrement_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,
            ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>,token:LifetimeToken| {
                let retiring=lifecycle::RetiringRest::with_token(rest,Ghost::new(token));
                *pending=lifecycle::State::on_release(Ghost::new(state),Ghost::new(c),retiring,current.borrow_mut()).into_inner();
            }});
        if old == 1 {
            let lease=ghost! {pending.as_ref().unwrap().borrow_token_for(public,snapshot!(*current))};
            field_event::acquire_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,
                ghost! {|state:&mut lifecycle::State<Payload>,c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
                    lifecycle::State::on_acquire(Ghost::new(state),Ghost::new(c),
                        ghost! {pending.as_ref().unwrap()},current.borrow_mut());
                }});
            let recovered=lifecycle::Pending::recover(ghost! {pending.into_inner().unwrap()},public,current);
            free_recovered(shared,recovered);
        }
    }
#[requires(recovered.inner_logic().0.wellformed())]
#[requires(recovered.inner_logic().1.frac() == PositiveReal::from_int(1))]
#[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.control.val().lft())]
#[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.physical.val().lft())]
#[requires(recovered.inner_logic().0.control.val().cur().pointer() == pointer as *const Shared)]
fn free_recovered(pointer:*mut Shared,recovered:Ghost<(Payload,LifetimeToken)>) {
    let (owner,rest)=ghost! {
        let (payload,lifetime)=recovered.into_inner();
        let dead=lifetime.end();
        let control=payload.control_end.get(dead);
        let physical=payload.physical_end.get(dead);
        (control.owner.into_inner(),(payload.base,(payload.recovery,physical)))
    }.split();
    let (bound,capabilities)=rest.split();
    let permission:Ghost<&Perm<*const Shared>>=ghost! {&**owner};
    let shared=unsafe {Perm::as_ref(pointer,permission)};
    unsafe {
        physical_projection::deallocate(shared.buf,shared.cap,bound,capabilities);
        deallocate_typed_box(pointer,owner);
    }
}

#[trusted]
#[requires(*owner.inner_logic().ward() == pointer as *const T)]
unsafe fn deallocate_typed_box<T>(pointer:*mut T,owner:Ghost<Box<Perm<*const T>>>) {
    if core::mem::size_of::<T>() != 0 {
        unsafe {dealloc(pointer.cast(),Layout::new::<T>())}
    }
}

#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn actual_public_shared_driver(input:Vec<u8>)->Vec<u8> {
    let first=Bytes::from(input);
    let second=first.clone();
    let third=first.clone();
    let fourth=first.clone();
    second.cleanup();
    first.cleanup();
    // Root ticket has retired; a surviving actual Bytes can still clone.
    let later=third.clone();
    let observed=AsRef::<[u8]>::as_ref(&later).to_vec();
    fourth.cleanup();
    third.cleanup();
    later.cleanup();
    observed
}
