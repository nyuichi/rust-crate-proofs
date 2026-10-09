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
use crate::{event::ScopedProtocol,free_effect};
use creusot_std::{ghost::invariant::Protocol, logic::{FMap,ra::excl::Excl}};
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


type Cursor = crate::event::ScopeCursor<lifecycle::State<Payload>>;
type CloneInput<'a> = (&'a OriginalSharedProof, &'a mut Cursor);
type CloneSpec<'a> = fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<CloneInput<'a>>)->Bytes;
type DropInput<'a> = (OriginalSharedProof, &'a mut Cursor, &'a mut Option<Completion>);
pub(crate) enum Completion {
    KeptAlive,
    Reclaimed(physical_projection::FreeReceipt, free_effect::TypedFreeReceipt<Shared>),
}
impl Completion {
    #[logic(open(crate))] fn reclaimed(self) -> bool {
        match self { Self::KeptAlive => false, Self::Reclaimed(_,_) => true }
    }
    #[logic] fn valid(self, metadata: <Payload as lifecycle::RecoveryPayload>::Metadata) -> bool {
        pearlite! { match self {
            Self::KeptAlive => true,
            Self::Reclaimed(buffer, control) =>
                buffer.namespace() == metadata.1 && buffer.pointer() == metadata.6 &&
                buffer.size() == metadata.2 && buffer.align() == 1 && buffer.allocated() &&
                control.pointer() as *const Shared == metadata.5 &&
                control.size() == creusot_std::std::mem::size_of_logic::<Shared>() &&
                control.align() == creusot_std::std::mem::align_of_logic::<Shared>() &&
                control.allocated()
        }}
    }
}

pub(crate) struct OriginalSharedProof {
    shared:*mut Shared,
    bound:raw_vec::BoundPtr,
    capacity:usize,
    control:Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    physical:Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    invariant:Ghost<GhostShared<field_event::ScopedFieldInvariant<lifecycle::State<Payload>>>>,
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
    #[logic] fn accepts(self,cursor:Cursor)->bool {
        cursor.model() == self.original_shared.invariant.inner_logic().val().model() &&
        cursor.public() == self.original_shared.invariant.inner_logic().val().public()
    }
    #[logic] fn ticket_id(self)->Int { self.original_shared.ticket.id() }
    #[logic] fn ticket_fraction(self)->PositiveReal { self.original_shared.ticket.fraction() }

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
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input)) == shared_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes>
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

#[requires(source.original_shared_valid() && source.accepts(*cursor.inner_logic()))]
#[ensures(result.original_shared_valid() && result.accepts(^cursor))]
#[ensures(source.accepts(^cursor))]
#[ensures(result.original_shared_bytes() == source.original_shared_bytes())]
#[ensures(result.ptr == source.ptr && result.len == source.len)]
#[ensures(!(*cursor.inner_logic().observation()).0.contains(result.ticket_id()))]
#[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.insert(result.ticket_id(), Excl(result.ticket_fraction())))]
#[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1 + 1)]
fn original_shared_clone(source:&Bytes,mut cursor:Ghost<&mut Cursor>)->Bytes {
    let native = source.vtable.clone;
    let (table,spec) = shared_registration();
    proof_assert!(source.vtable == table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),
        ghost! {(&*source.original_shared,&mut **cursor)},spec)
}

#[requires(input.inner_logic().0.valid_for(ptr,len,pointer_event::pointer_model(data)))]
#[requires(input.inner_logic().1.model() == input.inner_logic().0.invariant.inner_logic().val().model() &&
    input.inner_logic().1.public() == input.inner_logic().0.invariant.inner_logic().val().public())]
#[ensures(result.original_shared_valid())]
#[ensures(result.original_shared_bytes() == input.inner_logic().0.invariant.inner_logic().val().public().3.3)]
#[ensures(result.ptr == ptr && result.len == len)]
#[ensures(result.accepts(^input.inner_logic().1))]
#[ensures((^input.inner_logic().1).model() == input.inner_logic().1.model() &&
    (^input.inner_logic().1).public() == input.inner_logic().1.public())]
#[ensures(!(*input.inner_logic().1.observation()).0.contains(result.ticket_id()))]
#[ensures((*((^input.inner_logic().1).observation())).0 == (*input.inner_logic().1.observation()).0.insert(result.ticket_id(),Excl(result.ticket_fraction())))]
#[ensures((*((^input.inner_logic().1).observation())).1 == (*input.inner_logic().1.observation()).1 + 1)]
fn shared_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (source,cursor)=input.split();
    let shared_word=pointer_event::load_relaxed(data,ghost! {&*source.data_binding});
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared_word.cast::<Shared>(),ghost! {(*source.control).to_ref()},
        ghost! {&source.ticket.token},ghost! {(*source.invariant).to_ref()},cursor,
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared_word,pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared_word,permission);
    let (vtable,_spec)=shared_registration();
    Bytes {ptr,len,data,vtable,original_shared:ghost! {
        OriginalSharedProof {shared:source.shared,bound:source.bound,capacity:source.capacity,
            control:source.control,physical:source.physical,invariant:source.invariant,
            ticket:Ghost::new(ticket.into_inner().unwrap()),data_binding:binding}
    }}
}

/// Checked selected len<cap constructor instrumentation; native allocation,
/// record fields and data pointer correspond to the actual From<Vec> branch.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result.0.original_shared_valid() && result.0.accepts(result.1.inner_logic()))]
#[ensures(result.0.original_shared_bytes() == input@)]
#[ensures((*result.1.inner_logic().observation()).0 == FMap::singleton(result.0.ticket_id(),Excl(result.0.ticket_fraction())))]
#[ensures((*result.1.inner_logic().observation()).1 == 1)]
fn original_shared_from_vec(input:Vec<u8>)->(Bytes,Ghost<Cursor>) {
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
    let (field_invariant,cursor) = field_event::ScopedFieldInvariant::bind(&shared_ref.ref_cnt, state).split();
    let invariant = GhostShared::new(field_invariant);

    let (vtable,_spec) = shared_registration();
    (Bytes { ptr,len,data,vtable,original_shared:ghost! {
        OriginalSharedProof {shared,bound:base,capacity:cap,control,physical,invariant,ticket,data_binding}
    }},cursor)
}

// Native trait calls are mapped at the closed client; no narrowed From refinement here.

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


type DropSpec<'a> = fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<DropInput<'a>>);

#[trusted]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(result.0 == shared_table())]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DropInput>>
    result.1.inner_logic().precondition((data,ptr,len,input)) == shared_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DropInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),()) == shared_drop_checked.postcondition((data,ptr,len,input),()))]
fn shared_drop_registration<'a>()->(&'static Vtable,Ghost<DropSpec<'a>>) {
    #[cfg(not(creusot))]
    { (original_shared_table_native(), Ghost::conjure()) }
    #[cfg(creusot)]
    { unreachable!("generic closed-table consuming ghost-erasure reification") }
}

/// Consuming normal cleanup; completion is an extra erased output channel.
#[requires(value.original_shared_valid() && value.accepts(*cursor.inner_logic()))]
#[requires(*output.inner_logic() == None)]
#[ensures((^cursor).model() == cursor.inner_logic().model() && (^cursor).public() == cursor.inner_logic().public())]
#[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.remove(value.ticket_id()))]
#[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1)]
#[ensures(^output != None && (^output).unwrap_logic().valid(cursor.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().reclaimed() == ((*cursor.inner_logic().observation()).0.len() == 1))]
fn original_shared_cleanup(mut value:Bytes,mut cursor:Ghost<&mut Cursor>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let (table,spec)=shared_drop_registration();
    proof_assert!(value.vtable == table);
    let proof=value.original_shared;
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(proof.into_inner(),&mut **cursor,&mut **output)},spec)
}

#[requires(input.inner_logic().0.valid_for(ptr,len,pointer_event::pointer_model(data)))]
#[requires(input.inner_logic().1.model() == input.inner_logic().0.invariant.inner_logic().val().model() &&
    input.inner_logic().1.public() == input.inner_logic().0.invariant.inner_logic().val().public())]
#[requires(*input.inner_logic().2 == None)]
#[ensures((^input.inner_logic().1).model() == input.inner_logic().1.model() &&
    (^input.inner_logic().1).public() == input.inner_logic().1.public())]
#[ensures((*((^input.inner_logic().1).observation())).0 == (*input.inner_logic().1.observation()).0.remove(input.inner_logic().0.ticket.id()))]
#[ensures((*((^input.inner_logic().1).observation())).1 == (*input.inner_logic().1.observation()).1)]
#[ensures(^input.inner_logic().2 != None && (^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.public().3))]
#[ensures((^input.inner_logic().2).unwrap_logic().reclaimed() == ((*input.inner_logic().1.observation()).0.len() == 1))]
fn shared_drop_checked(data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DropInput>) {
    let (proof,mut cursor,mut output)=input.split();
    let shared=pointer_event::get_mut(data,ghost! {&*proof.data_binding}).cast::<Shared>();
    let public=snapshot!(proof.invariant.inner_logic().val().public());
    let control=ghost! {(*proof.control).to_ref()};
    let invariant=ghost! {(*proof.invariant).to_ref()};
    let ticket=ghost! {proof.into_inner().ticket.into_inner()};
    let (mut current,retiring)=lifecycle::prepare(ticket,public);
    let (lease,rest)=lifecycle::Retiring::split_token(retiring).split();
    let mut pending=ghost! {None::<lifecycle::Pending<Payload>>};
    let old=field_event::decrement_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,ghost! {&mut **cursor},
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Release>,token:LifetimeToken| {
            let retiring=lifecycle::RetiringRest::with_token(rest,Ghost::new(token));
            *pending=lifecycle::State::on_release(Ghost::new(state),Ghost::new(c),retiring,current.borrow_mut()).into_inner();
        }});
    if old == 1 {
        let lease=ghost! {pending.as_ref().unwrap().borrow_token_for(public,snapshot!(*current))};
        #[cfg(not(feature="negative_missing_acquire"))]
        field_event::acquire_owned::<Shared,lifecycle::State<Payload>,_>(shared,control,lease,invariant,ghost! {&mut **cursor},
            ghost! {|state:&mut lifecycle::State<Payload>,c:&Committer<ModelAtomic,usize,Acquire,NoStore>| {
                lifecycle::State::on_acquire(Ghost::new(state),Ghost::new(c),
                    ghost! {pending.as_ref().unwrap()},current.borrow_mut());
            }});
        let recovered=lifecycle::Pending::recover(ghost! {pending.into_inner().unwrap()},public,current);
        let receipts=free_recovered(shared,recovered);
        ghost! { let (buffer,control)=receipts.into_inner(); **output=Some(Completion::Reclaimed(buffer,control)); };
    } else { ghost! { **output=Some(Completion::KeptAlive); }; }
}
#[requires(recovered.inner_logic().0.wellformed())]
#[requires(recovered.inner_logic().1.frac() == PositiveReal::from_int(1))]
#[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.control.val().lft())]
#[requires(recovered.inner_logic().1.lft() == recovered.inner_logic().0.physical.val().lft())]
#[requires(recovered.inner_logic().0.control.val().cur().pointer() == pointer as *const Shared)]
#[ensures(Completion::Reclaimed(result.inner_logic().0,result.inner_logic().1).valid(recovered.inner_logic().0.metadata()))]
#[ensures(result.inner_logic().1.consumed(recovered.inner_logic().0.control.val().cur().owner.inner_logic()))]
fn free_recovered(pointer:*mut Shared,recovered:Ghost<(Payload,LifetimeToken)>)->Ghost<(physical_projection::FreeReceipt,free_effect::TypedFreeReceipt<Shared>)> {
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
        #[cfg(not(feature="negative_missing_payload_free"))]
        let buffer=physical_projection::deallocate(shared.buf,shared.cap,bound,capabilities);
        #[cfg(feature="negative_missing_payload_free")]
        let buffer=ghost! { Ghost::<physical_projection::FreeReceipt>::conjure().into_inner() };
        #[cfg(not(feature="negative_missing_control_free"))]
        let control=free_effect::deallocate_typed_box(pointer,owner);
        #[cfg(feature="negative_missing_control_free")]
        let control=ghost! { Ghost::<free_effect::TypedFreeReceipt<Shared>>::conjure().into_inner() };
        ghost! { (buffer.into_inner(),control.into_inner()) }
    }
}


/// Checked source shadow of the native closed client; no numeric ticket IDs.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn actual_public_shared_driver(input:Vec<u8>)->Vec<u8> {
    let (first,mut cursor)=original_shared_from_vec(input);
    let second=original_shared_clone(&first,cursor.borrow_mut());
    let third=original_shared_clone(&first,cursor.borrow_mut());
    let mut third_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(third,cursor.borrow_mut(),third_receipt.borrow_mut());
    proof_assert!(third_receipt.inner_logic() != None && !third_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut first_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(first,cursor.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic() != None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    let fourth=original_shared_clone(&second,cursor.borrow_mut());
    let borrowed=original_shared_as_slice(&fourth);
    let mut second_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(second,cursor.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    let observed=borrowed.to_vec();
    let mut fourth_receipt=ghost! {None::<Completion>};
    original_shared_cleanup(fourth,cursor.borrow_mut(),fourth_receipt.borrow_mut());
    proof_assert!(fourth_receipt.inner_logic() != None && fourth_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0.len() == 0);
    observed
}

#[requires(value.original_shared_valid() && value.accepts(*cursor.inner_logic()))]
#[requires(*output.inner_logic() == None)]
#[ensures((^cursor).model() == cursor.inner_logic().model() && (^cursor).public() == cursor.inner_logic().public())]
#[ensures((*(^cursor).observation()).0 == (*cursor.inner_logic().observation()).0.remove(value.ticket_id()))]
#[ensures((*(^cursor).observation()).1 == (*cursor.inner_logic().observation()).1)]
#[ensures(^output != None && (^output).unwrap_logic().valid(cursor.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().reclaimed() == ((*cursor.inner_logic().observation()).0.len() == 1))]
fn bytes_terminal_drop(value:Bytes,cursor:Ghost<&mut Cursor>,output:Ghost<&mut Option<Completion>>) {
    original_shared_cleanup(value,cursor,output)
}

/// Saved return value followed by certified native terminal normal Drop edges.
#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result@ == input@)]
pub(crate) fn shared_automatic_scope(input:Vec<u8>)->Vec<u8> {
    let (first,mut cursor)=original_shared_from_vec(input);
    let second=original_shared_clone(&first,cursor.borrow_mut());
    let first_id=snapshot!(first.ticket_id());
    let second_id=snapshot!(second.ticket_id());
    let live_before=snapshot!((*cursor.inner_logic().observation()).0);
    let borrowed=original_shared_as_slice(&second);
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut second_receipt=ghost! {None::<Completion>};
    bytes_terminal_drop(second,cursor.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic() != None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0 == (*live_before).remove(*second_id));
    proof_assert!((*cursor.inner_logic().observation()).0.contains(*first_id));
    proof_assert!(!(*cursor.inner_logic().observation()).0.contains(*second_id));
    let mut first_receipt=ghost! {None::<Completion>};
    bytes_terminal_drop(first,cursor.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic() != None && first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*cursor.inner_logic().observation()).0.len() == 0);
    saved_return
}
