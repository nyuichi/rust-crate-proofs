//! Closed first-promotion refinement: root Bytes plus an external affine scope.
//! No standalone weak Bytes invariant, hidden state getter, or native registry.
use alloc::{alloc::{dealloc,Layout},boxed::Box,vec::Vec};
use core::sync::atomic::{AtomicPtr,AtomicUsize};
use creusot_std::{prelude::*,ghost::{GhostShared,perm::Perm,
    lifetime_logic::{EndBorrow,FullBorrow,Lifetime,LifetimeToken}},
    logic::{Id,Int,real::PositiveReal},std::{ops::FnExt,sync::{
        atomic::{AtomicUsize as ModelAtomic,ordering::{Relaxed,Release,Acquire,None as NoStore}},
        committer::Committer,view::{ReleaseSyncView,SyncView}}}};
use crate::{boxed_alignment,field_event,pointer_event,physical_projection,raw_vec,lifecycle,erased_call};
use lifecycle::RecoveryPayload as _;
use creusot_std::prelude::Clone;
use crate::{event::ScopedProtocol,free_effect,owned_pointer,promotion_tags as tag_specs};
use creusot_std::std::sync::{atomic::AtomicPtr as ModelAtomicPtr,view::HasTimestamp};
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

pub(crate) struct SharedCore {
    shared:*mut Shared,
    bound:raw_vec::BoundPtr,
    capacity:usize,
    control:Ghost<GhostShared<FullBorrow<field_event::OwnedControl<Shared>>>>,
    physical:Ghost<GhostShared<FullBorrow<raw_vec::PhysicalRegion>>>,
    invariant:Ghost<GhostShared<field_event::ScopedFieldInvariant<lifecycle::State<Payload>>>>,
    ticket:Ghost<lifecycle::Ticket<Payload>>,
}
impl SharedCore {
    #[logic(prophetic)]
    pub(crate) fn valid_for(self,ptr:*const u8,len:usize)->bool {
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

#[derive(Clone, Copy)]
struct RootDescriptor {
    base: raw_vec::BoundPtr,
    capacity: usize,
    expected: Snapshot<Seq<u8>>,
    model: Snapshot<ModelAtomicPtr<()>>,
}
impl RootDescriptor {
    #[logic] fn word(self)->*mut () {
        if self.base.raw_pointer().addr_logic() & 1usize == 0usize {
            tag_specs::tagged_data(self.base.raw_pointer())
        } else { self.base.raw_pointer() as *mut () }
    }
    #[logic] fn table(self)->&'static Vtable {
        if self.base.raw_pointer().addr_logic() & 1usize == 0usize { even_table() } else { odd_table() }
    }
    #[logic] fn valid(self)->bool {
        pearlite! {self.base.invariant() && self.base@ != None &&
            self.base.current_address()==self.base.raw_pointer().addr_logic()@ &&
            self.base@.unwrap_logic().1==self.capacity@ && self.base@.unwrap_logic().2==0 &&
            self.capacity>0usize && (*self.expected).len()==self.capacity@ &&
            self.word().addr_logic() & 1usize == 1usize}
    }
    #[logic] fn matches(self,ptr:*const u8,len:usize,data:ModelAtomicPtr<()>,table:&'static Vtable)->bool {
        self.valid() && ptr==self.base.raw_pointer() as *const u8 && len==self.capacity &&
            data==*self.model && table==self.table()
    }
}
struct ChildProof { core:SharedCore, binding:Ghost<pointer_event::ReadOnlyPointer> }
enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof), View(ChildProof,raw_vec::BoundPtr), Empty(EmptyViewProof), Vacant }

struct RawPhase {
    recovery:raw_vec::Recovery,
    physical:raw_vec::PhysicalRegion,
    own:Perm<ModelAtomicPtr<()>>,
    current:SyncView,
}
impl RawPhase {
    #[logic(prophetic)] fn valid(self,d:RootDescriptor)->bool {
        pearlite! {d.valid() && self.recovery.invariant() && self.physical.invariant() &&
            d.base@==Some((self.recovery.namespace(),d.capacity@,0int)) &&
            self.recovery.capacity()==d.capacity@ && self.physical.capacity()==d.capacity@ &&
            self.physical.namespace()==self.recovery.namespace() &&
            self.physical.resource_id()==self.recovery.namespace() &&
            self.physical.lo()==0 && self.physical.hi()==d.capacity@ &&
            (forall<i:Int> 0<=i && i<d.capacity@ ==> self.physical.slot(i)==Some(Some((*d.expected)[i]))) &&
            *self.own.ward()==*d.model &&
            (exists<t:Int,published:SyncView> self.own.val()==FMap::singleton(t,(d.word(),published)))}
    }
}
struct SharedPhase {
    root:SharedCore,
    cursor:Cursor,
    own:Perm<ModelAtomicPtr<()>>,
    current:SyncView,
}
impl SharedPhase {
    #[logic(prophetic)] fn valid(self,d:RootDescriptor)->bool {
        pearlite! {d.valid() && self.root.valid() && self.root.accepts(self.cursor) &&
            self.root.bound==d.base && self.root.capacity==d.capacity && self.root.content()==*d.expected &&
            self.root.shared.addr_logic() & 1usize == 0usize &&
            *self.own.ward()==*d.model &&
            owned_pointer::latest_is(self.own.val(),self.root.shared as *mut ()) &&
            owned_pointer::visible_is(self.own.val(),(*d.model).get_timestamp(self.current),self.root.shared as *mut ())}
    }
}
enum Phase { Raw(RawPhase), Shared(SharedPhase) }
struct PromotionScope { descriptor:RootDescriptor, phase:Option<Phase> }
impl PromotionScope {
    #[logic] fn is_raw(self)->bool {match self.phase {Some(Phase::Raw(_))=>true,_=>false}}
    #[logic] fn is_shared(self)->bool {match self.phase {Some(Phase::Shared(_))=>true,_=>false}}
    #[logic(prophetic)] fn valid(self)->bool {match self.phase {
        Some(Phase::Raw(raw))=>raw.valid(self.descriptor),
        Some(Phase::Shared(shared))=>shared.valid(self.descriptor),
        None=>self.descriptor.valid(),
    }}
    #[logic(prophetic)] fn root_valid(self,value:Bytes)->bool {
        self.valid() && self.phase!=None && match value.original_shared.inner_logic() {
            OriginalSharedProof::Root(d)=>d==self.descriptor &&
                d.matches(value.ptr,value.len,pointer_event::pointer_model(&value.data),value.vtable),
            _=>false,
        }
    }
    #[logic] #[requires(self.is_shared())]
    fn observation(self)->Snapshot<(crate::fraction_map::LiveFractions,Int)> {
        match self.phase {Some(Phase::Shared(s))=>s.cursor.observation(),_=>creusot_std::logic::any()}
    }
    #[logic] #[requires(self.is_shared())]
    fn cursor_model(self)->ModelAtomic {match self.phase {Some(Phase::Shared(s))=>s.cursor.model(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_shared())]
    fn cursor_public(self)-><lifecycle::State<Payload> as Protocol>::Public {
        match self.phase {Some(Phase::Shared(s))=>s.cursor.public(),_=>creusot_std::logic::any()}
    }
    #[logic] #[requires(self.is_shared())]
    fn root_id(self)->Int {match self.phase {Some(Phase::Shared(s))=>s.root.ticket.id(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_shared())]
    fn root_fraction(self)->PositiveReal {match self.phase {Some(Phase::Shared(s))=>s.root.ticket.fraction(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_shared())]
    fn root_metadata(self)-><Payload as lifecycle::RecoveryPayload>::Metadata {
        match self.phase {Some(Phase::Shared(s))=>s.root.public().3,_=>creusot_std::logic::any()}
    }
    #[logic(prophetic)] fn same_root(self,before:Self)->bool {
        match (self.phase,before.phase) {(Some(Phase::Shared(a)),Some(Phase::Shared(b)))=>a.root==b.root,_=>false}
    }
    #[logic] fn same_pointer_owner(self,before:Self)->bool {
        match (self.phase,before.phase) {(Some(Phase::Shared(a)),Some(Phase::Shared(b)))=>a.own==b.own && a.current==b.current,_=>false}
    }
}
impl Bytes {
    #[logic(prophetic)] fn child_valid(self)->bool {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)=>p.core.valid() && self.ptr==p.core.bound.raw_pointer() as *const u8 &&
            self.len==p.core.capacity && self.vtable==shared_table() &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==p.core.shared as *mut (),
        _=>false,
    }}
    #[logic] fn is_child(self)->bool {match self.original_shared.inner_logic() {OriginalSharedProof::Child(_)=>true,_=>false}}
    #[logic] #[requires(self.is_child())]
    fn child_id(self)->Int {match self.original_shared.inner_logic() {OriginalSharedProof::Child(p)=>p.core.ticket.id(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_child())]
    fn child_fraction(self)->PositiveReal {match self.original_shared.inner_logic() {OriginalSharedProof::Child(p)=>p.core.ticket.fraction(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_child())]
    fn child_content(self)->Seq<u8> {match self.original_shared.inner_logic() {OriginalSharedProof::Child(p)=>p.core.content(),_=>creusot_std::logic::any()}}
    #[logic] #[requires(self.is_child())]
    fn child_public(self)-><lifecycle::State<Payload> as Protocol>::Public {match self.original_shared.inner_logic() {OriginalSharedProof::Child(p)=>p.core.public(),_=>creusot_std::logic::any()}}
    #[logic] fn child_accepts(self,scope:PromotionScope)->bool {
        match (self.original_shared.inner_logic(),scope.phase) {
            (OriginalSharedProof::Child(p),Some(Phase::Shared(s)))=>p.core.accepts(s.cursor),_=>false
        }
    }
}

#[logic(opaque)] fn even_table()->&'static Vtable {dead}
#[logic(opaque)] fn odd_table()->&'static Vtable {dead}
#[logic(opaque)] fn shared_table()->&'static Vtable {dead}
#[trusted] #[ensures(result==even_table())]
fn even_table_reification()->&'static Vtable {unreachable!("closed native even-table reification")}
#[trusted] #[ensures(result==odd_table())]
fn odd_table_reification()->&'static Vtable {unreachable!("closed native odd-table reification")}
#[trusted] #[ensures(result==shared_table())]
fn shared_table_reification()->&'static Vtable {unreachable!("closed native Shared-table reification")}

// Same assumed exposed-provenance forward tag as AE/AK, with one generic symbol.
#[trusted]
#[requires(ptr.addr_logic() & 1usize==0usize)]
#[ensures(result==tag_specs::tagged_data(ptr))]
#[ensures(result.addr_logic()==(ptr.addr_logic() | 1usize))]
fn tag_pointer(ptr:*mut u8)->*mut () {
    #[cfg(creusot)] {unreachable!("generic native exposed-provenance forward tag")}
    #[cfg(not(creusot))] {((ptr as usize)|1) as *mut ()}
}

#[requires(input@.len()>0)]
#[ensures(result.1.inner_logic().root_valid(result.0) && result.1.inner_logic().is_raw())]
#[ensures(*result.1.inner_logic().descriptor.expected==input@)]
fn from_box_scoped(input:Box<[u8]>)->(Bytes,Ghost<PromotionScope>) {
    if input.len()==0 {proof_assert!(false);unreachable!("nonempty selected client excludes static branch");}
    let expected=snapshot!(input@);
    let (raw,len,capabilities)=raw_vec::detach_boxed_slice(input);
    let (base,capacity)=raw.into_bound_ptr_at_zero();
    let ptr=base.as_ptr();
    let address=crate::provenance_specs::pointer_addr(ptr);
    ghost! {tag_specs::classify_low_bit(address);tag_specs::tagged_low_bit(address);};
    let (word,vtable)=if address & 1usize==0usize {
        (tag_pointer(ptr),even_table_reification())
    } else {(ptr.cast::<()>(),odd_table_reification())};
    let mut current=ghost! {SyncView::new().into_inner()};
    let (data,own)=pointer_event::new_pointer(word,current.borrow_mut());
    let descriptor=ghost! {RootDescriptor {base,capacity,expected,model:snapshot!(pointer_event::pointer_model(&data))}};
    let scope=ghost! {
        let (recovery,physical)=capabilities.into_inner();
        PromotionScope {descriptor:*descriptor,phase:Some(Phase::Raw(RawPhase {
            recovery,physical,own:own.into_inner(),current:current.into_inner(),
        }))}
    };
    (Bytes {ptr,len,data,vtable,original_shared:ghost! {OriginalSharedProof::Root(*descriptor)}},scope)
}

type CloneInput<'a>=(&'a RootDescriptor,&'a mut PromotionScope);
type CloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<CloneInput<'a>>)->Bytes;

#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(pointer_event::pointer_model(data)==*input.inner_logic().0.model)]
#[requires(offset==input.inner_logic().0.base.raw_pointer() as *const u8 && len==input.inner_logic().0.capacity)]
#[requires(expected==input.inner_logic().0.word() && buf==input.inner_logic().0.base.raw_pointer())]
#[ensures((^input.inner_logic().1).valid() && (^input.inner_logic().1).is_shared())]
#[ensures((^input.inner_logic().1).descriptor==input.inner_logic().1.descriptor)]
#[ensures(result.child_valid() && result.child_content()==*input.inner_logic().0.expected)]
#[ensures(result.child_accepts(^input.inner_logic().1))]
#[ensures(result.ptr==offset && result.len==len)]
#[ensures((*((^input.inner_logic().1).observation())).0==FMap::singleton(
    (^input.inner_logic().1).root_id(),Excl((^input.inner_logic().1).root_fraction())).insert(
    result.child_id(),Excl(result.child_fraction())))]
#[ensures((^input.inner_logic().1).root_id()!=result.child_id())]
fn shallow_clone_vec_checked(data:&AtomicPtr<()>,expected:*mut (),buf:*mut u8,offset:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (descriptor,mut scope)=input.split();
    let mut raw=ghost! {match scope.phase.take().unwrap() {Phase::Raw(raw)=>raw,_=>{proof_assert!(false);panic!()}}};
    let distance=unsafe {tag_specs::equal_pointer_distance(offset,buf)};
    let cap=distance as usize + len;
    let mut control_view=ghost! {SyncView::new().into_inner()};
    let (ref_cnt,count_permission)=field_event::new(2,control_view.borrow_mut());
    let count_view=snapshot!(*control_view);
    let boxed=Box::new(Shared {buf,cap,ref_cnt});
    let (shared,shared_owner)=boxed_alignment::into_raw_aligned(boxed);
    let address=crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(address,core::mem::align_of::<Shared>());
    debug_assert!(address & 1usize==0usize);
    let (own_borrow,current_borrow)=ghost! {
        let raw=&mut *raw; (&mut raw.own,&mut raw.current)
    }.split();
    let exchanged=owned_pointer::exchange_singleton(data,expected,shared.cast(),own_borrow,current_borrow);
    match exchanged {
        Ok(actual)=>{
            debug_assert!(actual==expected);
            let (recovery,region,root_own,root_current)=ghost! {
                let raw=raw.into_inner(); (raw.recovery,raw.physical,raw.own,raw.current)
            }.split();
            let lifetime=ghost! {LifetimeToken::new()};
            let lifetime_id=snapshot!(lifetime.lft());
            let (control_full,control_end)=FullBorrow::new(field_event::own_control(shared,shared_owner),lifetime_id);
            let (physical_full,physical_end)=FullBorrow::new(region,lifetime_id);
            let control=ghost! {GhostShared::new(control_full).into_inner()};
            let physical=ghost! {GhostShared::new(physical_full).into_inner()};
            let payload=ghost! {Payload {recovery:recovery.into_inner(),physical_end:physical_end.into_inner(),control_end:control_end.into_inner(),
                physical,control,base:descriptor.base,capacity:cap,len,expected:descriptor.expected}};
            let initialized=lifecycle::State::initialize_pair(count_permission,count_view,payload,lifetime);
            let (state,root_ticket,child_ticket)=initialized.split();
            let permission:Ghost<&Perm<*const Shared>>=ghost! {
                let full:&FullBorrow<field_event::OwnedControl<Shared>>=(*control).to_ref();
                let owner=full.borrow(&root_ticket.token); &**owner.owner
            };
            let shared_ref=unsafe {Perm::as_ref(shared,permission)};
            let (field_invariant,cursor)=field_event::ScopedFieldInvariant::bind(&shared_ref.ref_cnt,state).split();
            let invariant=GhostShared::new(field_invariant);
            let root=ghost! {SharedCore {shared,bound:descriptor.base,capacity:cap,control,physical,invariant,ticket:root_ticket}};
            let child=ghost! {SharedCore {shared,bound:descriptor.base,capacity:cap,control,physical,invariant,ticket:child_ticket}};
            let mut child_view=ghost! {SyncView::new().into_inner()};
            let (child_data,child_permission)=pointer_event::new_pointer(shared.cast(),child_view.borrow_mut());
            let binding=pointer_event::bind_read_only(&child_data,shared.cast(),child_permission);
            ghost! {scope.phase=Some(Phase::Shared(SharedPhase {root:root.into_inner(),cursor:cursor.into_inner(),own:root_own.into_inner(),current:root_current.into_inner()}));};
            Bytes {ptr:offset,len,data:child_data,vtable:shared_table_reification(),original_shared:ghost! {
                OriginalSharedProof::Child(ChildProof {core:child.into_inner(),binding})
            }}
        }
        Err(_actual)=>{
            // Native Box::from_raw / forget(candidate) / shallow_clone_arc loser
            // branch is retained in the source map, and impossible in this scope.
            proof_assert!(false);unreachable!("owned singleton excludes first-CAS failure")
        }
    }
}

impl SharedCore {
    #[logic(prophetic)] fn valid(self)->bool {
        self.valid_for(self.bound.raw_pointer() as *const u8,self.capacity)
    }
    #[logic] fn content(self)->Seq<u8> { self.invariant.inner_logic().val().public().3.3 }
    #[logic] fn public(self)-><lifecycle::State<Payload> as Protocol>::Public {
        self.invariant.inner_logic().val().public()
    }
    #[logic] fn accepts(self,cursor:Cursor)->bool {
        cursor.model()==self.invariant.inner_logic().val().model() && cursor.public()==self.public()
    }
}

#[logic(prophetic)]
fn clone_result(d:RootDescriptor,s:PromotionScope,result:Bytes)->bool {
    pearlite! {s.valid() && s.is_shared() && s.descriptor==d && result.child_valid() &&
        result.ptr==d.base.raw_pointer() as *const u8 && result.len==d.capacity &&
        result.child_content()==*d.expected && result.child_accepts(s) &&
        (*s.observation()).0==FMap::singleton(
            s.root_id(),Excl(s.root_fraction())).insert(
            result.child_id(),Excl(result.child_fraction())) &&
        s.root_id()!=result.child_id()}
}

#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize==0usize)]
#[ensures(clone_result(*input.inner_logic().0,^input.inner_logic().1,result))]
fn even_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (descriptor,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                _=>{proof_assert!(false);panic!()},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        proof_assert!(false);unreachable!("first clone raw phase excludes existing ARC branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let buf=tag_specs::clear_low_bit(stored,ghost! {&descriptor.base});
        shallow_clone_vec_checked(data,stored,buf,offset,len,ghost! {(*descriptor,&mut **scope)})
    }
}
#[trusted]
#[ensures(result.0==even_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==even_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==even_clone_checked.postcondition((data,ptr,len,input),output))]
fn even_clone_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {unreachable!("checked closed table/clone erasure")}

#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize!=0usize)]
#[ensures(clone_result(*input.inner_logic().0,^input.inner_logic().1,result))]
fn odd_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (descriptor,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                _=>{proof_assert!(false);panic!()},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        proof_assert!(false);unreachable!("first clone raw phase excludes existing ARC branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let buf=stored.cast::<u8>();
        shallow_clone_vec_checked(data,stored,buf,offset,len,ghost! {(*descriptor,&mut **scope)})
    }
}
#[trusted]
#[ensures(result.0==odd_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==odd_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==odd_clone_checked.postcondition((data,ptr,len,input),output))]
fn odd_clone_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {unreachable!("checked closed table/clone erasure")}

#[requires(scope.inner_logic().root_valid(*source) && scope.inner_logic().is_raw())]
#[ensures((^scope).root_valid(*source))]
#[ensures(clone_result(scope.inner_logic().descriptor,^scope,result))]
fn clone_root(source:&Bytes,mut scope:Ghost<&mut PromotionScope>)->Bytes {
    let descriptor=ghost! {match &*source.original_shared {OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
    let native=source.vtable.clone;
    let address=crate::provenance_specs::pointer_addr(source.ptr);
    if address & 1usize==0usize {
        let (table,spec)=even_clone_registration();
        proof_assert!(source.vtable==table);
        erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*descriptor,&mut **scope)},spec)
    } else {
        let (table,spec)=odd_clone_registration();
        proof_assert!(source.vtable==table);
        erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*descriptor,&mut **scope)},spec)
    }
}

#[requires(proof.inner_logic().valid() && proof.inner_logic().shared==shared)]
#[requires(proof.inner_logic().accepts(*cursor.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^cursor).model()==cursor.inner_logic().model() && (^cursor).public()==cursor.inner_logic().public())]
#[ensures((*((^cursor).observation())).0==(*cursor.inner_logic().observation()).0.remove(proof.inner_logic().ticket.id()))]
#[ensures((*((^cursor).observation())).1==(*cursor.inner_logic().observation()).1)]
#[ensures(^output!=None && (^output).unwrap_logic().valid(proof.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().reclaimed()==((*cursor.inner_logic().observation()).0.len()==1))]
#[ensures((^output).unwrap_logic().reclaimed() ==> (*((^cursor).observation())).0.len()==0)]
fn release_core(shared:*mut Shared,proof:Ghost<SharedCore>,mut cursor:Ghost<&mut Cursor>,mut output:Ghost<&mut Option<Completion>>) {
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
}#[requires(recovered.inner_logic().0.wellformed())]
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

type ChildDropInput<'a>=(ChildProof,&'a mut Cursor,&'a mut Option<Completion>);
type ChildDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<ChildDropInput<'a>>);
#[requires(input.inner_logic().0.core.valid() && input.inner_logic().0.core.accepts(*input.inner_logic().1))]
#[requires(input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data))]
#[requires(input.inner_logic().0.binding.inner_logic().value()==input.inner_logic().0.core.shared as *mut ())]
#[requires(*input.inner_logic().2==None)]
#[ensures((^input.inner_logic().1).model()==input.inner_logic().1.model() && (^input.inner_logic().1).public()==input.inner_logic().1.public())]
#[ensures((*((^input.inner_logic().1).observation())).0==(*input.inner_logic().1.observation()).0.remove(input.inner_logic().0.core.ticket.id()))]
#[ensures((*((^input.inner_logic().1).observation())).1==(*input.inner_logic().1.observation()).1)]
#[ensures(^input.inner_logic().2!=None && (^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().0.core.public().3))]
#[ensures((^input.inner_logic().2).unwrap_logic().reclaimed()==((*input.inner_logic().1.observation()).0.len()==1))]
fn child_drop_checked(data:&mut AtomicPtr<()>,_ptr:*const u8,_len:usize,input:Ghost<ChildDropInput>) {
    let (proof,cursor,output)=input.split();
    let shared=pointer_event::get_mut(data,ghost! {&*proof.binding}).cast::<Shared>();
    release_core(shared,ghost! {proof.into_inner().core},cursor,output);
}
#[trusted]
#[ensures(result.0==shared_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ChildDropInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==child_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ChildDropInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==child_drop_checked.postcondition((data,ptr,len,input),()))]
fn child_drop_registration<'a>()->(&'static Vtable,Ghost<ChildDropSpec<'a>>) {unreachable!("closed native shared-child drop erasure")}

#[requires(value.child_valid() && scope.inner_logic().valid() && scope.inner_logic().is_shared())]
#[requires(value.child_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).is_shared() && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures((^scope).same_root(*scope.inner_logic()))]
#[ensures((^scope).same_pointer_owner(*scope.inner_logic()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.remove(value.child_id()))]
#[ensures(^output!=None && (^output).unwrap_logic().valid(value.child_public().3))]
#[ensures((^output).unwrap_logic().reclaimed()==((*scope.inner_logic().observation()).0.len()==1))]
fn cleanup_child(mut value:Bytes,mut scope:Ghost<&mut PromotionScope>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let proof=ghost! {match value.original_shared.into_inner() {OriginalSharedProof::Child(p)=>p,_=>{proof_assert!(false);panic!()}}};
    let mut shared=ghost! {match scope.phase.as_mut().unwrap() {Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}}};
    let (table,spec)=child_drop_registration();
    proof_assert!(value.vtable==table);
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut shared.cursor,&mut **output)},spec);
}

type RootDropInput<'a>=(RootDescriptor,&'a mut PromotionScope,&'a mut Option<Completion>);
type RootDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<RootDropInput<'a>>);
#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_shared())]
#[requires(input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),input.inner_logic().0.table()))]
#[requires((*input.inner_logic().1.observation()).0.len()==1)]
#[requires(*input.inner_logic().2==None)]
#[ensures((^input.inner_logic().1).valid() && (^input.inner_logic().1).phase==None)]
#[ensures((^input.inner_logic().1).descriptor==input.inner_logic().1.descriptor)]
#[ensures(^input.inner_logic().2!=None && (^input.inner_logic().2).unwrap_logic().reclaimed())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.root_metadata()))]
fn root_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootDropInput>) {
    let (_descriptor,mut scope,output)=input.split();
    let (root,mut cursor,own,_current)=ghost! {
        let shared=match scope.phase.take().unwrap() {Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}};
        (shared.root,shared.cursor,shared.own,shared.current)
    }.split();
    let (word,_stamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word) & 1usize;
    if kind==0usize {
        release_core(word.cast(),root,cursor.borrow_mut(),output);
        proof_assert!((*cursor.observation()).0.len()==0);
    } else {
        // The real unpromoted free_boxed_slice branch is present in both
        // native promotable destructors; paired Shared phase excludes it.
        proof_assert!(false);unreachable!("updated root history excludes raw free branch")
    }
}

#[trusted]
#[ensures(result.0==even_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==root_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_root_drop_registration<'a>()->(&'static Vtable,Ghost<RootDropSpec<'a>>) {unreachable!("closed native promotable ARC-drop erasure")}

#[trusted]
#[ensures(result.0==odd_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootDropInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==root_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_root_drop_registration<'a>()->(&'static Vtable,Ghost<RootDropSpec<'a>>) {unreachable!("closed native promotable ARC-drop erasure")}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().is_shared())]
#[requires((*scope.inner_logic().observation()).0.len()==1)]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).phase==None && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures(^output!=None && (^output).unwrap_logic().reclaimed())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().root_metadata()))]
fn cleanup_root(mut value:Bytes,mut scope:Ghost<&mut PromotionScope>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {match value.original_shared.into_inner() {OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
    let address=crate::provenance_specs::pointer_addr(value.ptr);
    if address & 1usize==0usize {
        let (table,spec)=even_root_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(descriptor.into_inner(),&mut **scope,&mut **output)},spec);
    } else {
        let (table,spec)=odd_root_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(descriptor.into_inner(),&mut **scope,&mut **output)},spec);
    }
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().is_shared())]
#[ensures(result@==*scope.inner_logic().descriptor.expected)]
fn read_root<'a>(value:&'a Bytes,scope:Ghost<&'a PromotionScope>)->&'a [u8] {
    let root=ghost! {match scope.phase.as_ref().unwrap() {Phase::Shared(s)=>&s.root,_=>{proof_assert!(false);panic!()}}};
    let region=ghost! {
        let full:&FullBorrow<raw_vec::PhysicalRegion>=(*root.physical).to_ref();
        full.borrow(&root.ticket.token)
    };
    unsafe {physical_projection::borrow(value.ptr,value.len,ghost! {&root.bound},region)}
}

/// Actual nonempty-box first clone, child retirement, original read and final
/// original retirement. No guessed ticket IDs; the constructor pair exports
/// the complete two-entry ledger. Native APIs carry no explicit scope.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn first_promotion_client(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let child=clone_root(&original,scope.borrow_mut());
    let mut child_completion=ghost! {None::<Completion>};
    cleanup_child(child,scope.borrow_mut(),child_completion.borrow_mut());
    proof_assert!(child_completion.inner_logic()!=None && !child_completion.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0.len()==1);
    let observed=read_root(&original,scope.borrow()).to_vec();
    let mut root_completion=ghost! {None::<Completion>};
    cleanup_root(original,scope.borrow_mut(),root_completion.borrow_mut());
    proof_assert!(scope.phase==None && root_completion.inner_logic()!=None && root_completion.inner_logic().unwrap_logic().reclaimed());
    observed
}

#[requires(value.child_valid() && scope.inner_logic().valid() && scope.inner_logic().is_shared())]
#[requires(value.child_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).is_shared() && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures((^scope).same_root(*scope.inner_logic()))]
#[ensures((^scope).same_pointer_owner(*scope.inner_logic()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.remove(value.child_id()))]
#[ensures(^output!=None && (^output).unwrap_logic().valid(value.child_public().3))]
#[ensures((^output).unwrap_logic().reclaimed()==((*scope.inner_logic().observation()).0.len()==1))]
fn bytes_child_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_child(value,scope,output)
}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().is_shared())]
#[requires((*scope.inner_logic().observation()).0.len()==1)]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).phase==None && (^scope).descriptor==scope.inner_logic().descriptor)]
#[ensures(^output!=None && (^output).unwrap_logic().reclaimed())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().root_metadata()))]
fn bytes_root_terminal_drop(value:Bytes,scope:Ghost<&mut PromotionScope>,output:Ghost<&mut Option<Completion>>) {
    cleanup_root(value,scope,output)
}

// AN adds the existing-ARC branch of Clone on the original promotable handle.
// AL's first-promotion core and AM's consuming terminal adapters stay exact.
impl PromotionScope {
    /// The native Acquire may advance the view; it neither consumes nor
    /// rewrites the root's owned pointer history. This predicate returns no
    /// permission, State, token, or executable pointer capability.
    #[logic]
    fn pointer_history_progresses(self,before:Self)->bool {
        match (self.phase,before.phase) {
            (Some(Phase::Shared(after)),Some(Phase::Shared(before)))=>
                after.own==before.own && before.current<=after.current,
            _=>false,
        }
    }
}

/// Body-proved generic Acquire observation with an erased expected value.
/// The expected pointer participates only in specifications. The only native
/// operation is the existing load_acquire adapter and its operation callback.
#[requires(*own.ward()==pointer_event::pointer_model(field))]
#[requires(owned_pointer::visible_is(own.val(),pointer_event::pointer_model(field).get_timestamp(**current),*expected))]
#[ensures(result==*expected)]
#[ensures(**current<=^current)]
#[ensures(owned_pointer::visible_is(own.val(),pointer_event::pointer_model(field).get_timestamp(^current),*expected))]
fn load_visible_snapshot(field:&AtomicPtr<()>,expected:Snapshot<*mut ()>,
    own:Ghost<&Perm<ModelAtomicPtr<()>>>,mut current:Ghost<&mut SyncView>)->*mut () {
    owned_pointer::load_acquire(field,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            c.shoot_load(&**own,&mut **current);
        }
    })
}

type ArcCloneInput<'a>=(&'a SharedCore,&'a mut Cursor);

#[requires(input.inner_logic().0.valid())]
#[requires(shared==input.inner_logic().0.shared)]
#[requires(ptr==input.inner_logic().0.bound.raw_pointer() as *const u8 && len==input.inner_logic().0.capacity)]
#[requires(input.inner_logic().0.accepts(*input.inner_logic().1))]
#[ensures(result.child_valid() && result.child_content()==input.inner_logic().0.content())]
#[ensures(result.ptr==ptr && result.len==len)]
#[ensures(result.child_public()==input.inner_logic().0.public())]
#[ensures(match result.original_shared.inner_logic() {
    OriginalSharedProof::Child(p)=>p.core.accepts(^input.inner_logic().1),_=>false,
})]
#[ensures((^input.inner_logic().1).model()==input.inner_logic().1.model())]
#[ensures((^input.inner_logic().1).public()==input.inner_logic().1.public())]
#[ensures(!(*input.inner_logic().1.observation()).0.contains(result.child_id()))]
#[ensures((*((^input.inner_logic().1).observation())).0==(*input.inner_logic().1.observation()).0.insert(result.child_id(),Excl(result.child_fraction())))]
#[ensures((*((^input.inner_logic().1).observation())).1==(*input.inner_logic().1.observation()).1+1)]
fn shallow_clone_arc_checked(shared:*mut Shared,ptr:*const u8,len:usize,input:Ghost<ArcCloneInput>)->Bytes {
    let (source,cursor)=input.split();
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared,ghost! {(*source.control).to_ref()},ghost! {&source.ticket.token},
        ghost! {(*source.invariant).to_ref()},cursor,
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared.cast(),pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared.cast(),permission);
    Bytes {ptr,len,data,vtable:shared_table_reification(),original_shared:ghost! {
        OriginalSharedProof::Child(ChildProof {
            core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,
                control:source.control,physical:source.physical,invariant:source.invariant,
                ticket:Ghost::new(ticket.into_inner().unwrap())},
            binding,
        })
    }}
}

#[logic(prophetic)]
fn reclone_result(before:PromotionScope,after:PromotionScope,result:Bytes)->bool {
    pearlite! {before.is_shared() && after.valid() && after.is_shared() &&
        after.descriptor==before.descriptor && after.same_root(before) &&
        after.pointer_history_progresses(before) &&
        after.cursor_model()==before.cursor_model() && after.cursor_public()==before.cursor_public() &&
        result.child_valid() && result.child_accepts(after) &&
        result.ptr==before.descriptor.base.raw_pointer() as *const u8 && result.len==before.descriptor.capacity &&
        result.child_content()==*before.descriptor.expected &&
        !(*before.observation()).0.contains(result.child_id()) &&
        (*after.observation()).0==(*before.observation()).0.insert(result.child_id(),Excl(result.child_fraction())) &&
        (*after.observation()).1==(*before.observation()).1+1}
}

#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_shared())]
#[requires(*input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize==0usize)]
#[ensures(reclone_result(*input.inner_logic().1,^input.inner_logic().1,result))]
fn even_reclone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (_descriptor,mut scope)=input.split();
    let mut phase=ghost! {match scope.phase.as_mut().unwrap() {
        Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()},
    }};
    let expected=snapshot!(phase.root.shared as *mut ());
    let (own,current)=ghost! {let phase=&mut **phase;(&phase.own,&mut phase.current)}.split();
    let stored=load_visible_snapshot(data,expected,own,current);
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        shallow_clone_arc_checked(stored.cast(),offset,len,ghost! {
            let phase=&mut **phase;(&phase.root,&mut phase.cursor)
        })
    } else {
        // The actual even callback retains its raw first-promotion branch.
        // This paired Shared-phase precondition makes that branch unreachable.
        proof_assert!(false);unreachable!("shared phase excludes repeat raw promotion")
    }
}

#[requires(input.inner_logic().1.valid() && input.inner_logic().1.is_shared())]
#[requires(*input.inner_logic().0==input.inner_logic().1.descriptor)]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize!=0usize)]
#[ensures(reclone_result(*input.inner_logic().1,^input.inner_logic().1,result))]
fn odd_reclone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<CloneInput>)->Bytes {
    let (_descriptor,mut scope)=input.split();
    let mut phase=ghost! {match scope.phase.as_mut().unwrap() {
        Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()},
    }};
    let expected=snapshot!(phase.root.shared as *mut ());
    let (own,current)=ghost! {let phase=&mut **phase;(&phase.own,&mut phase.current)}.split();
    let stored=load_visible_snapshot(data,expected,own,current);
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        shallow_clone_arc_checked(stored.cast(),offset,len,ghost! {
            let phase=&mut **phase;(&phase.root,&mut phase.cursor)
        })
    } else {
        proof_assert!(false);unreachable!("shared phase excludes repeat raw promotion")
    }
}

// New instances of the existing generic source-checked callback registration
// boundary, with Shared-phase contracts for the same native vtable functions.
#[trusted]
#[ensures(result.0==even_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==even_reclone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==even_reclone_checked.postcondition((data,ptr,len,input),output))]
fn even_reclone_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {unreachable!("checked closed even-table ARC-clone erasure")}

#[trusted]
#[ensures(result.0==odd_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==odd_reclone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==odd_reclone_checked.postcondition((data,ptr,len,input),output))]
fn odd_reclone_registration<'a>()->(&'static Vtable,Ghost<CloneSpec<'a>>) {unreachable!("checked closed odd-table ARC-clone erasure")}

#[requires(scope.inner_logic().root_valid(*source) && scope.inner_logic().is_shared())]
#[ensures((^scope).root_valid(*source))]
#[ensures(reclone_result(*scope.inner_logic(),^scope,result))]
fn reclone_root(source:&Bytes,mut scope:Ghost<&mut PromotionScope>)->Bytes {
    let descriptor=ghost! {match &*source.original_shared {
        OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()},
    }};
    let native=source.vtable.clone;
    let address=crate::provenance_specs::pointer_addr(source.ptr);
    if address & 1usize==0usize {
        let (table,spec)=even_reclone_registration();
        proof_assert!(source.vtable==table);
        erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*descriptor,&mut **scope)},spec)
    } else {
        let (table,spec)=odd_reclone_registration();
        proof_assert!(source.vtable==table);
        erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*descriptor,&mut **scope)},spec)
    }
}

/// Native lexical second/first Drop, surviving root read, saved return and
/// final root Drop. Every ledger key comes from the actual returned ticket.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_reclone_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let first=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let first_id=snapshot!(first.child_id());
    let before_reclone=snapshot!((*scope.observation()).0);
    let second=reclone_root(&original,scope.borrow_mut());
    let second_id=snapshot!(second.child_id());
    let second_fraction=snapshot!(second.child_fraction());
    let live_three=snapshot!((*scope.observation()).0);
    proof_assert!(!(*before_reclone).contains(*second_id));
    proof_assert!(*live_three==(*before_reclone).insert(*second_id,Excl(*second_fraction)));
    proof_assert!((*live_three).len()==3);
    let mut second_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(second,scope.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic()!=None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_three).remove(*second_id));
    proof_assert!(!(*scope.observation()).0.contains(*second_id));
    proof_assert!((*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==2);
    let live_two=snapshot!((*scope.observation()).0);
    let mut first_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(first,scope.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_two).remove(*first_id));
    proof_assert!(!(*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}


// AO: the root is consumed. The surviving scope contains no root resources.
struct DetachedScope { cursor:Cursor }
impl DetachedScope {
    #[logic] fn observation(self)->Snapshot<(crate::fraction_map::LiveFractions,Int)> { self.cursor.observation() }
    #[logic] fn model(self)->ModelAtomic { self.cursor.model() }
    #[logic] fn public(self)-><lifecycle::State<Payload> as Protocol>::Public { self.cursor.public() }
    #[logic] fn accepts(self,value:Bytes)->bool {
        match value.original_shared.inner_logic() {
            OriginalSharedProof::Child(p)=>p.core.accepts(self.cursor),_=>false,
        }
    }
}

type DetachingRootInput<'a>=(PromotionScope,&'a mut Option<DetachedScope>,&'a mut Option<Completion>);
type DetachingRootSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<DetachingRootInput<'a>>);
#[requires(input.inner_logic().0.valid() && input.inner_logic().0.is_shared())]
#[requires(input.inner_logic().0.descriptor.matches(offset,len,pointer_event::pointer_model(data),input.inner_logic().0.descriptor.table()))]
#[requires((*input.inner_logic().0.observation()).0.len()>1)]
#[requires(*input.inner_logic().1==None && *input.inner_logic().2==None)]
#[ensures(^input.inner_logic().1!=None)]
#[ensures((^input.inner_logic().1).unwrap_logic().model()==input.inner_logic().0.cursor_model())]
#[ensures((^input.inner_logic().1).unwrap_logic().public()==input.inner_logic().0.cursor_public())]
#[ensures((*((^input.inner_logic().1).unwrap_logic().observation())).0==(*input.inner_logic().0.observation()).0.remove(input.inner_logic().0.root_id()))]
#[ensures((*((^input.inner_logic().1).unwrap_logic().observation())).1==(*input.inner_logic().0.observation()).1)]
#[ensures(^input.inner_logic().2!=None && !(^input.inner_logic().2).unwrap_logic().reclaimed())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().0.root_metadata()))]
fn detaching_root_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<DetachingRootInput>) {
    let (scope,mut detached,output)=input.split();
    let (root,mut cursor,own,_current)=ghost! {
        let shared=match scope.into_inner().phase.unwrap() {Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}};
        (shared.root,shared.cursor,shared.own,shared.current)
    }.split();
    let (word,_stamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word) & 1usize;
    if kind==0usize {
        release_core(word.cast(),root,cursor.borrow_mut(),output);
        ghost! {**detached=Some(DetachedScope {cursor:cursor.into_inner()});};
    } else {
        proof_assert!(false);unreachable!("updated root history excludes raw free branch")
    }
}

#[trusted]
#[ensures(result.0==even_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DetachingRootInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==detaching_root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DetachingRootInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==detaching_root_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_detaching_root_registration<'a>()->(&'static Vtable,Ghost<DetachingRootSpec<'a>>) {unreachable!("closed native nonfinal root drop erasure")}

#[trusted]
#[ensures(result.0==odd_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DetachingRootInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==detaching_root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<DetachingRootInput>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==detaching_root_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_detaching_root_registration<'a>()->(&'static Vtable,Ghost<DetachingRootSpec<'a>>) {unreachable!("closed native nonfinal root drop erasure")}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().is_shared())]
#[requires((*scope.inner_logic().observation()).0.len()>1)]
#[requires(*detached.inner_logic()==None && *output.inner_logic()==None)]
#[ensures(^detached!=None)]
#[ensures((^detached).unwrap_logic().model()==scope.inner_logic().cursor_model())]
#[ensures((^detached).unwrap_logic().public()==scope.inner_logic().cursor_public())]
#[ensures((*((^detached).unwrap_logic().observation())).0==(*scope.inner_logic().observation()).0.remove(scope.inner_logic().root_id()))]
#[ensures((*((^detached).unwrap_logic().observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures(^output!=None && !(^output).unwrap_logic().reclaimed())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().root_metadata()))]
fn bytes_root_detaching_terminal_drop(mut value:Bytes,scope:Ghost<PromotionScope>,mut detached:Ghost<&mut Option<DetachedScope>>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {match value.original_shared.into_inner() {OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
    let address=crate::provenance_specs::pointer_addr(value.ptr);
    if address & 1usize==0usize {
        let (table,spec)=even_detaching_root_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **detached,&mut **output)},spec);
    } else {
        let (table,spec)=odd_detaching_root_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **detached,&mut **output)},spec);
    }
}

#[requires(value.child_valid() && scope.inner_logic().accepts(*value))]
#[ensures(result@==value.child_content())]
fn read_surviving_child<'a>(value:&'a Bytes,scope:Ghost<&'a DetachedScope>)->&'a [u8] {
    let child=ghost! {match &*value.original_shared {OriginalSharedProof::Child(p)=>p,_=>{proof_assert!(false);panic!()}}};
    let region=ghost! {
        let full:&FullBorrow<raw_vec::PhysicalRegion>=(*child.core.physical).to_ref();
        full.borrow(&child.core.ticket.token)
    };
    unsafe {physical_projection::borrow(value.ptr,value.len,ghost! {&child.core.bound},region)}
}

#[requires(value.child_valid() && scope.inner_logic().accepts(value))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.remove(value.child_id()))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures(^output!=None && (^output).unwrap_logic().valid(value.child_public().3))]
#[ensures((^output).unwrap_logic().reclaimed()==((*scope.inner_logic().observation()).0.len()==1))]
fn bytes_detached_child_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut DetachedScope>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let proof=ghost! {match value.original_shared.into_inner() {OriginalSharedProof::Child(p)=>p,_=>{proof_assert!(false);panic!()}}};
    let (table,spec)=child_drop_registration();
    proof_assert!(value.vtable==table);
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut scope.cursor,&mut **output)},spec);
}

/// Inner original Drop publishes recovery; the surviving child's final Drop
/// recovers it. No original owner or root permission survives the handoff.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_surviving_child_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!(!(*detached.observation()).0.contains(*root_id));
    proof_assert!((*detached.observation()).0.contains(*child_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!(detached.accepts(survivor));
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}


// AP: reusable original Shared-vtable clone with an external affine cursor.
type SharedChildCloneInput<'a>=(&'a ChildProof,&'a mut Cursor);
type SharedChildCloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<SharedChildCloneInput<'a>>)->Bytes;
#[requires(input.inner_logic().0.core.valid() && input.inner_logic().0.core.accepts(*input.inner_logic().1))]
#[requires(input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data))]
#[requires(input.inner_logic().0.binding.inner_logic().value()==input.inner_logic().0.core.shared as *mut ())]
#[requires(ptr==input.inner_logic().0.core.bound.raw_pointer() as *const u8 && len==input.inner_logic().0.core.capacity)]
#[ensures(result.child_valid() && result.child_content()==input.inner_logic().0.core.content())]
#[ensures(result.child_public()==input.inner_logic().0.core.public())]
#[ensures(result.ptr==ptr && result.len==len)]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::Child(p)=>p.core.accepts(^input.inner_logic().1),_=>false})]
#[ensures((^input.inner_logic().1).model()==input.inner_logic().1.model() && (^input.inner_logic().1).public()==input.inner_logic().1.public())]
#[ensures(!(*input.inner_logic().1.observation()).0.contains(result.child_id()))]
#[ensures((*((^input.inner_logic().1).observation())).0==(*input.inner_logic().1.observation()).0.insert(result.child_id(),Excl(result.child_fraction())))]
#[ensures((*((^input.inner_logic().1).observation())).1==(*input.inner_logic().1.observation()).1+1)]
fn shared_child_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SharedChildCloneInput>)->Bytes {
    let (source,cursor)=input.split();
    let shared=pointer_event::load_relaxed(data,ghost! {&*source.binding});
    shallow_clone_arc_checked(shared.cast(),ptr,len,ghost! {(&source.core,cursor.into_inner())})
}

#[trusted]
#[ensures(result.0==shared_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SharedChildCloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==shared_child_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SharedChildCloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==shared_child_clone_checked.postcondition((data,ptr,len,input),output))]
fn shared_child_clone_registration<'a>()->(&'static Vtable,Ghost<SharedChildCloneSpec<'a>>) {unreachable!("closed native Shared clone erasure")}

#[requires(source.child_valid() && scope.inner_logic().accepts(*source))]
#[ensures(result.child_valid() && result.child_content()==source.child_content())]
#[ensures(result.child_public()==source.child_public() && (^scope).accepts(result))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.child_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.child_id(),Excl(result.child_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_surviving_child(source:&Bytes,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let proof=ghost! {match &*source.original_shared {OriginalSharedProof::Child(p)=>p,_=>{proof_assert!(false);panic!()}}};
    let native=source.vtable.clone;
    let (table,spec)=shared_child_clone_registration();
    proof_assert!(source.vtable==table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*proof,&mut scope.cursor)},spec)
}

/// Exact, resource-free Boolean description of the actual affine owners.
/// No State, Perm, ticket or native pointer authority can be obtained from it.
#[logic(prophetic)]
fn finite_inventory(owners:Seq<Bytes>,survivor:Bytes,scope:DetachedScope)->bool {
    pearlite! {
        survivor.child_valid() && scope.accepts(survivor) &&
        (*scope.observation()).0.get(survivor.child_id())==Some(Excl(survivor.child_fraction())) &&
        (*scope.observation()).0.len()==owners.len()+1 &&
        (forall<i:Int> 0<=i && i<owners.len() ==>
            owners[i].child_valid() && scope.accepts(owners[i]) &&
            owners[i].child_content()==survivor.child_content() &&
            owners[i].child_public()==survivor.child_public() &&
            owners[i].child_id()!=survivor.child_id() &&
            (*scope.observation()).0.get(owners[i].child_id())==Some(Excl(owners[i].child_fraction()))) &&
        (forall<i:Int,j:Int> 0<=i && i<owners.len() && 0<=j && j<owners.len() && i!=j ==>
            owners[i].child_id()!=owners[j].child_id()) &&
        (forall<id:Int> (*scope.observation()).0.contains(id) ==
            (id==survivor.child_id() || exists<i:Int> 0<=i && i<owners.len() && owners[i].child_id()==id))
    }
}

/// Separate the sequence/map induction from the native client. All parameters
/// are erased observations; this lemma cannot return or extract affine owners.
#[check(ghost)]
#[requires(finite_inventory(*owners,*survivor,*before))]
#[requires((*peer).child_valid() && (*peer).child_content()==(*survivor).child_content())]
#[requires((*peer).child_public()==(*survivor).child_public() && (*after).accepts(*peer))]
#[requires((*after).model()==(*before).model() && (*after).public()==(*before).public())]
#[requires(!(*(*before).observation()).0.contains((*peer).child_id()))]
#[requires((*(*after).observation()).0==(*(*before).observation()).0.insert((*peer).child_id(),Excl((*peer).child_fraction())))]
#[ensures(finite_inventory((*owners).push_back(*peer),*survivor,*after))]
fn prove_inventory_push(owners:Snapshot<Seq<Bytes>>,survivor:Snapshot<Bytes>,
    peer:Snapshot<Bytes>,before:Snapshot<DetachedScope>,after:Snapshot<DetachedScope>) {
    proof_assert!((*after).accepts(*survivor));
    proof_assert!((*peer).child_id()!=(*survivor).child_id());
    proof_assert!(forall<i:Int> 0<=i && i<(*owners).len() ==>
        (*after).accepts((*owners)[i]) && (*owners)[i].child_id()!=(*peer).child_id());
    proof_assert!((*owners).push_back(*peer)[(*owners).len()]==*peer);
    proof_assert!(forall<i:Int> 0<=i && i<(*owners).push_back(*peer).len() ==>
        if i<(*owners).len() {(*owners).push_back(*peer)[i]==(*owners)[i]}
        else {i==(*owners).len() && (*owners).push_back(*peer)[i]==*peer});
    proof_assert!(forall<i:Int> 0<=i && i<(*owners).push_back(*peer).len() ==>
        (*owners).push_back(*peer)[i].child_valid() &&
        (*after).accepts((*owners).push_back(*peer)[i]) &&
        (*owners).push_back(*peer)[i].child_content()==(*survivor).child_content() &&
        (*owners).push_back(*peer)[i].child_public()==(*survivor).child_public() &&
        (*owners).push_back(*peer)[i].child_id()!=(*survivor).child_id() &&
        (*(*after).observation()).0.get((*owners).push_back(*peer)[i].child_id())==
            Some(Excl((*owners).push_back(*peer)[i].child_fraction())));
    proof_assert!(forall<i:Int,j:Int> 0<=i && i<(*owners).push_back(*peer).len() &&
        0<=j && j<(*owners).push_back(*peer).len() && i!=j ==>
        (*owners).push_back(*peer)[i].child_id()!=(*owners).push_back(*peer)[j].child_id());
    proof_assert!(forall<id:Int> (*(*after).observation()).0.contains(id)==
        (id==(*peer).child_id() || (*(*before).observation()).0.contains(id)));
    proof_assert!(forall<id:Int> (exists<i:Int> 0<=i && i<(*owners).push_back(*peer).len() &&
        (*owners).push_back(*peer)[i].child_id()==id)==
        (id==(*peer).child_id() || exists<i:Int> 0<=i && i<(*owners).len() && (*owners)[i].child_id()==id));
}

/// Proof-only consuming adapter at the actual Vec normal Drop edge. The real
/// standard-library destructor still deallocates vector storage. Empty contents
/// exclude element Drop effects; this is not a Bytes recovery/free receipt.
#[requires(value@.len()==0)]
fn empty_vec_terminal_drop<T>(value:Vec<T>) {
    let _=value;
}

/// Runtime count creates an arbitrary finite inventory. Every popped peer is
/// retired by its actual lexical Drop; empty Vec Drop precedes final survivor.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn finite_shared_scope(input:Box<[u8]>,count:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1);
    let mut owners:Vec<Bytes>=Vec::new();
    let mut made=0usize;
    #[invariant(made<=count)]
    #[invariant(owners@.len()==made@)]
    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(count@-made@)]
    while made<count {
        let old_owners=snapshot!(owners@);
        let old_scope=snapshot!(detached.inner_logic());
        let next=clone_surviving_child(&survivor,detached.borrow_mut());
        ghost! {prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),
            old_scope,snapshot!(detached.inner_logic()));};
        owners.push(next);
        made+=1;
    }
    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(owners@.len())]
    while let Some(peer)=owners.pop() {
        proof_assert!(finite_inventory(owners@.push_back(peer),survivor,detached.inner_logic()));
        let mut peer_receipt=ghost! {None::<Completion>};
        bytes_detached_child_terminal_drop(peer,detached.borrow_mut(),peer_receipt.borrow_mut());
        proof_assert!(peer_receipt.inner_logic()!=None && !peer_receipt.inner_logic().unwrap_logic().reclaimed());
        proof_assert!(peer_receipt.inner_logic().unwrap_logic().valid(*metadata));
    }
    proof_assert!(owners@.len()==0 && (*detached.observation()).0.len()==1);
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    empty_vec_terminal_drop(owners);
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}


// AQ: full-allocation ownership and separately bounded public views.
struct EmptyViewProof {
    bound:raw_vec::BoundPtr,
    binding:Ghost<pointer_event::ReadOnlyPointer>,
}
#[logic(opaque)] fn static_view_table()->&'static Vtable {dead}
#[trusted] #[ensures(result==static_view_table())]
fn static_view_table_reification()->&'static Vtable {unreachable!("native STATIC_VTABLE reification")}

#[logic(prophetic)]
fn bounded_view(core:SharedCore,bound:raw_vec::BoundPtr,ptr:*const u8,len:usize)->bool {
    pearlite! {core.valid() && bound.invariant() && bound@!=None &&
        bound@.unwrap_logic().0==core.bound@.unwrap_logic().0 &&
        bound@.unwrap_logic().1==core.capacity@ &&
        0<=bound@.unwrap_logic().2 && bound@.unwrap_logic().2+len@<=core.capacity@ &&
        ptr==bound.raw_pointer() as *const u8 &&
        bound.current_address()==ptr.addr_logic()@ &&
        bound.current_address()==core.bound.current_address()+bound@.unwrap_logic().2}
}
impl Bytes {
    #[logic] fn view_owned(self)->bool {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)|OriginalSharedProof::View(_,_)=>true,_=>false,
    }}
    #[logic] fn view_bound(self)->raw_vec::BoundPtr {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)=>p.core.bound,
        OriginalSharedProof::View(_,b)=>b,
        OriginalSharedProof::Empty(p)=>p.bound,
        _=>creusot_std::logic::any(),
    }}
    #[logic(prophetic)] fn view_valid(self)->bool {pearlite! {!self.ptr.is_null_logic() && match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)=>self.child_valid(),
        OriginalSharedProof::View(p,b)=>bounded_view(p.core,b,self.ptr,self.len) && self.len>0usize &&
            self.vtable==shared_table() &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==p.core.shared as *mut (),
        OriginalSharedProof::Empty(p)=>self.len==0usize && self.vtable==static_view_table() &&
            p.bound.invariant() && p.bound@==None && self.ptr==p.bound.raw_pointer() as *const u8 &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==crate::view_pointer::null_word(),
        _=>false,
    }}}
    #[logic] fn view_content(self)->Seq<u8> {pearlite! {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)=>p.core.content(),
        OriginalSharedProof::View(p,b)=>p.core.content().subsequence(b@.unwrap_logic().2,b@.unwrap_logic().2+self.len@),
        OriginalSharedProof::Empty(_)=>Seq::empty(),
        _=>creusot_std::logic::any(),
    }}}
    #[logic] fn view_id(self)->Int {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.ticket.id(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_fraction(self)->PositiveReal {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.ticket.fraction(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_public(self)-><lifecycle::State<Payload> as Protocol>::Public {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.public(),
        _=>creusot_std::logic::any(),
    }}
    #[logic] fn view_accepts(self,scope:DetachedScope)->bool {match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.accepts(scope.cursor),
        OriginalSharedProof::Empty(_)=>true,_=>false,
    }}
}

#[requires(!ptr.is_null_logic())]
#[ensures(result.view_valid() && !result.view_owned() && result.view_content()==Seq::empty())]
#[ensures(result.len==0usize && result.ptr.addr_logic()==ptr.addr_logic())]
fn new_empty_view(ptr:*const u8)->Bytes {
    debug_assert!(!ptr.is_null());
    let (ptr,bound)=crate::view_pointer::without_provenance(ptr);
    let mut current=ghost! {SyncView::new().into_inner()};
    let null=crate::view_pointer::null_pointer();
    let (data,permission)=pointer_event::new_pointer(null,current.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,null,permission);
    Bytes {ptr,len:0,data,vtable:static_view_table_reification(),
        original_shared:ghost! {OriginalSharedProof::Empty(EmptyViewProof {bound:bound.into_inner(),binding})}}
}

type ViewArcInput<'a>=(&'a SharedCore,raw_vec::BoundPtr,&'a mut Cursor);
#[requires(bounded_view(*input.inner_logic().0,input.inner_logic().1,ptr,len) && len>0usize)]
#[requires(shared==input.inner_logic().0.shared && input.inner_logic().0.accepts(*input.inner_logic().2))]
#[ensures(result.view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.public())]
#[ensures(result.view_content()==input.inner_logic().0.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
fn shallow_clone_view_checked(shared:*mut Shared,ptr:*const u8,len:usize,input:Ghost<ViewArcInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared,ghost! {(*source.control).to_ref()},ghost! {&source.ticket.token},
        ghost! {(*source.invariant).to_ref()},cursor,
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared.cast(),pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared.cast(),permission);
    Bytes {ptr,len,data,vtable:shared_table_reification(),original_shared:ghost! {
        OriginalSharedProof::View(ChildProof {
            core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,
                control:source.control,physical:source.physical,invariant:source.invariant,
                ticket:Ghost::new(ticket.into_inner().unwrap())},binding,
        },bound.into_inner())
    }}
}

type ViewCloneInput<'a>=(&'a ChildProof,raw_vec::BoundPtr,&'a mut Cursor);
type ViewCloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<ViewCloneInput<'a>>)->Bytes;
#[requires(bounded_view(input.inner_logic().0.core,input.inner_logic().1,ptr,len) && len>0usize)]
#[requires(input.inner_logic().0.core.accepts(*input.inner_logic().2))]
#[requires(input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data))]
#[requires(input.inner_logic().0.binding.inner_logic().value()==input.inner_logic().0.core.shared as *mut ())]
#[ensures(result.view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.core.public())]
#[ensures(result.view_content()==input.inner_logic().0.core.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
fn shared_view_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let shared=pointer_event::load_relaxed(data,ghost! {&*source.binding});
    shallow_clone_view_checked(shared.cast(),ptr,len,ghost! {(&source.core,bound.into_inner(),cursor.into_inner())})
}
#[trusted]
#[ensures(result.0==shared_table())]
#[ensures(erased_call::registered3(result.0.clone,result.1.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>>
    result.1.inner_logic().precondition((data,ptr,len,input))==shared_view_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>,output:Bytes>
    result.1.inner_logic().postcondition((data,ptr,len,input),output)==shared_view_clone_checked.postcondition((data,ptr,len,input),output))]
fn shared_view_clone_registration<'a>()->(&'static Vtable,Ghost<ViewCloneSpec<'a>>) {unreachable!("native Shared view clone erasure")}

#[requires(source.view_valid() && source.view_owned() && source.view_accepts(*scope.inner_logic()))]
#[ensures(result.view_valid() && result.view_owned() && result.view_content()==source.view_content())]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(_,_)=>true,_=>false})]
#[ensures(result.view_bound()==source.view_bound() && result.ptr==source.ptr && result.len==source.len)]
#[ensures(result.view_public()==source.view_public() && result.view_accepts(^scope))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.view_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_shared_view(source:&Bytes,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let (proof,bound)=ghost! {match &*source.original_shared {
        OriginalSharedProof::Child(p)=>(p,p.core.bound),OriginalSharedProof::View(p,b)=>(p,*b),
        _=>{proof_assert!(false);panic!()},
    }}.split();
    let native=source.vtable.clone;
    let (table,spec)=shared_view_clone_registration();
    proof_assert!(source.vtable==table);
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*proof,bound.into_inner(),&mut scope.cursor)},spec)
}

#[requires(source.view_valid() && source.view_accepts(*scope.inner_logic()))]
#[requires(range.start<=range.end && range.end<=source.len)]
#[ensures(result.view_valid() && result.view_content()==source.view_content().subsequence(range.start@,range.end@))]
#[ensures(result.len@==range.end@-range.start@ && result.view_owned()==(range.start<range.end))]
#[ensures(result.view_accepts(^scope))]
#[ensures(result.view_owned() ==> result.view_public()==source.view_public())]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(if result.view_owned() {
    !(*scope.inner_logic().observation()).0.contains(result.view_id()) &&
    (*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())) &&
    (*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1
} else {(^scope).observation()==scope.inner_logic().observation()})]
fn slice_view(source:&Bytes,range:core::ops::Range<usize>,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    use core::ops::{Bound,RangeBounds};
    let len=source.len;
    let view_begin=match range.start_bound() {
        Bound::Included(&n)=>n,
        Bound::Excluded(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Unbounded=>0,
    };
    let end=match range.end_bound() {
        Bound::Included(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Excluded(&n)=>n,
        Bound::Unbounded=>len,
    };
    assert!(view_begin<=end,"range start must not be greater than end: {:?} <= {:?}",view_begin,end);
    assert!(end<=len,"range end out of bounds: {:?} <= {:?}",end,len);
    if end==view_begin {
        let bound=ghost! {match &*source.original_shared {
            OriginalSharedProof::Child(p)=>p.core.bound,OriginalSharedProof::View(_,b)=>*b,
            OriginalSharedProof::Empty(p)=>p.bound,_=>{proof_assert!(false);panic!()},
        }};
        let (ptr,_bound)=crate::view_pointer::wrapping_bounded(source.ptr,view_begin,bound.borrow());
        return new_empty_view(ptr);
    }
    let mut ret=clone_shared_view(source,ghost! {&mut **scope});
    ret.len=end-view_begin;
    let (bound,region)=ghost! {
        match &*ret.original_shared {
            OriginalSharedProof::View(p,b)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
                (b,full.borrow(&p.core.ticket.token))
            },_=>{proof_assert!(false);panic!()},
        }
    }.split();
    let (ptr,shifted)=unsafe {crate::view_pointer::add_live(ret.ptr,view_begin,bound,region)};
    ret.ptr=ptr;
    ret.original_shared=ghost! {match ret.original_shared.into_inner() {
        OriginalSharedProof::View(p,_)=>OriginalSharedProof::View(p,shifted.into_inner()),
        _=>{proof_assert!(false);panic!()},
    }};
    ret
}

#[requires(value.view_valid() && value.view_accepts(*scope.inner_logic()))]
#[ensures(result@==value.view_content())]
fn read_view<'a>(value:&'a Bytes,scope:Ghost<&'a DetachedScope>)->&'a [u8] {
    if value.len==0 {
        let bound=ghost! {match &*value.original_shared {
            OriginalSharedProof::Empty(p)=>&p.bound,_=>{proof_assert!(false);panic!()},
        }};
        unsafe {physical_projection::borrow_empty(value.ptr,bound)}
    } else {
        let (proof,bound)=ghost! {match &*value.original_shared {
            OriginalSharedProof::Child(p)=>(p,&p.core.bound),OriginalSharedProof::View(p,b)=>(p,b),
            _=>{proof_assert!(false);panic!()},
        }}.split();
        let region=ghost! {
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*proof.core.physical).to_ref();
            full.borrow(&proof.core.ticket.token)
        };
        unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
    }
}

// Static means no ticket existed. Shared is the actual release_core receipt.
enum ViewEffect { Static, Shared(Completion) }
impl ViewEffect {
    #[logic] fn reclaimed(self)->bool {match self {Self::Static=>false,Self::Shared(c)=>c.reclaimed()}}
    #[logic] fn valid(self,metadata:<Payload as lifecycle::RecoveryPayload>::Metadata)->bool {
        match self {Self::Static=>true,Self::Shared(c)=>c.valid(metadata)}
    }
    #[logic] fn was_static(self)->bool {match self {Self::Static=>true,_=>false}}
}

type StaticDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<()>);
#[ensures(^data==*data)]
fn static_view_drop_checked(data:&mut AtomicPtr<()>,_ptr:*const u8,_len:usize,_input:Ghost<()>) {}
#[trusted]
#[ensures(result.0==static_view_table())]
#[ensures(erased_call::registered3(result.0.drop,result.1.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<()>>
    result.1.inner_logic().precondition((data,ptr,len,input))==static_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<()>>
    result.1.inner_logic().postcondition((data,ptr,len,input),())==static_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn static_view_drop_registration<'a>()->(&'static Vtable,Ghost<StaticDropSpec<'a>>) {unreachable!("native static_drop erasure")}

#[requires(value.view_valid() && value.view_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures((*((^scope).observation())).0==if value.view_owned() {
    (*scope.inner_logic().observation()).0.remove(value.view_id())
} else {(*scope.inner_logic().observation()).0})]
#[ensures(^output!=None && (^output).unwrap_logic().valid(scope.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().was_static()==!value.view_owned())]
#[ensures((^output).unwrap_logic().reclaimed()==(value.view_owned() && (*scope.inner_logic().observation()).0.len()==1))]
fn bytes_view_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut DetachedScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    let native=value.vtable.drop;
    if value.len==0 {
        let (table,spec)=static_view_drop_registration();
        proof_assert!(value.vtable==table);
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {()},spec);
        ghost! {**output=Some(ViewEffect::Static);};
    } else {
        let proof=ghost! {match value.original_shared.into_inner() {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p,
            _=>{proof_assert!(false);panic!()},
        }};
        let (table,spec)=child_drop_registration();
        proof_assert!(value.vtable==table);
        let mut completion=ghost! {None::<Completion>};
        erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(proof.into_inner(),&mut scope.cursor,&mut *completion)},spec);
        ghost! {**output=Some(ViewEffect::Shared(completion.into_inner().unwrap()));};
    }
}

/// All valid concrete ranges, including nested and one-past empty endpoints.
#[requires(input@.len()>0)]
#[requires(a<=b && b@<=input@.len() && c<=d && d@<=b@-a@)]
#[ensures(result@==input@.subsequence(a@+c@,a@+d@))]
pub(crate) fn nested_slice_scope(input:Box<[u8]>,a:usize,b:usize,c:usize,d:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let owner_id=snapshot!(owner.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!((*detached.observation()).0.contains(*owner_id));
    let first=slice_view(&owner,a..b,detached.borrow_mut());
    let first_id=snapshot!(first.view_id());
    proof_assert!(first.view_owned() ==> *first_id!=*owner_id);
    proof_assert!(first.view_content()==(*expected).subsequence(a@,b@));
    let selected=slice_view(&first,c..d,detached.borrow_mut());
    let selected_id=snapshot!(selected.view_id());
    proof_assert!(selected.view_owned() ==> *selected_id!=*owner_id && *selected_id!=*first_id);
    proof_assert!(selected.view_content()==(*expected).subsequence(a@+c@,a@+d@));
    proof_assert!((*detached.observation()).0.len()==1+(if a<b {1int}else{0int})+(if c<d {1int}else{0int}));
    let mut first_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(first,detached.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(first_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==1+(if c<d {1int}else{0int}));
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && owner_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()==(c==d));
    proof_assert!((*detached.observation()).0.len()==(if c<d {1int}else{0int}));
    let borrowed=read_view(&selected,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut selected_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(selected,detached.borrow_mut(),selected_receipt.borrow_mut());
    proof_assert!(selected_receipt.inner_logic()!=None && selected_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(selected_receipt.inner_logic().unwrap_logic().reclaimed()==(c<d));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()!=selected_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}

// Generic library contract missing from the pinned Std slice surface. This
// preserves the actual native slice::is_empty call. Its interpretation is
// exactly slice length == 0, analogous to Std's adjacent slice::len contract;
// no Bytes representation, ownership, recovery or destructor fact is assumed.
creusot_std::macros::extern_spec! {
    impl<T> [T] {
        #[check(ghost)]
        #[ensures(result == (self@.len() == 0))]
        fn is_empty(&self) -> bool;
    }
}

// AR: byte length and ownership are independent.
impl Bytes {
    #[logic(prophetic)] fn api_view_valid(self)->bool {!self.ptr.is_null_logic() && match self.original_shared.inner_logic() {
        OriginalSharedProof::Child(_)=>self.child_valid(),
        OriginalSharedProof::View(p,b)=>bounded_view(p.core,b,self.ptr,self.len) &&
            self.vtable==shared_table() &&
            p.binding.inner_logic().model()==pointer_event::pointer_model(&self.data) &&
            p.binding.inner_logic().value()==p.core.shared as *mut (),
        OriginalSharedProof::Empty(_)=>self.view_valid(),
        _=>false,
    }}
}
impl OriginalSharedProof {
    #[logic] fn api_owned(self)->bool {match self {
        Self::Child(_)|Self::View(_,_)=>true,_=>false,
    }}
    #[logic] fn api_id(self)->Int {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.ticket.id(),_=>creusot_std::logic::any(),
    }}
    #[logic] fn api_public(self)-><lifecycle::State<Payload> as Protocol>::Public {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.public(),_=>creusot_std::logic::any(),
    }}
    #[logic(prophetic)] fn shared_drop_input(self,data:ModelAtomicPtr<()>,cursor:Cursor)->bool {match self {
        Self::Child(p)|Self::View(p,_)=>p.core.valid() && p.core.accepts(cursor) &&
            p.binding.inner_logic().model()==data && p.binding.inner_logic().value()==p.core.shared as *mut (),
        _=>false,
    }}
    #[logic] fn static_drop_input(self)->bool {match self {Self::Empty(_)=>true,_=>false}}
}

type CursorDropInput<'a>=(OriginalSharedProof,&'a mut Cursor,&'a mut Option<ViewEffect>);
type CursorDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<CursorDropInput<'a>>);
#[requires(input.inner_logic().0.shared_drop_input(pointer_event::pointer_model(data),*input.inner_logic().1))]
#[requires(*input.inner_logic().2==None)]
#[ensures((^input.inner_logic().1).model()==input.inner_logic().1.model() && (^input.inner_logic().1).public()==input.inner_logic().1.public())]
#[ensures((*((^input.inner_logic().1).observation())).0==(*input.inner_logic().1.observation()).0.remove(input.inner_logic().0.api_id()))]
#[ensures((*((^input.inner_logic().1).observation())).1==(*input.inner_logic().1.observation()).1)]
#[ensures(^input.inner_logic().2!=None && !(^input.inner_logic().2).unwrap_logic().was_static())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.public().3))]
#[ensures((^input.inner_logic().2).unwrap_logic().reclaimed()==((*input.inner_logic().1.observation()).0.len()==1))]
fn cursor_shared_drop_checked(data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>) {
    let (proof,cursor,mut output)=input.split();
    let child=ghost! {match proof.into_inner() {
        OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p,
        _=>{proof_assert!(false);panic!()},
    }};
    let mut completion=ghost! {None::<Completion>};
    child_drop_checked(data,ptr,len,ghost! {(child.into_inner(),cursor.into_inner(),&mut *completion)});
    ghost! {**output=Some(ViewEffect::Shared(completion.into_inner().unwrap()));};
}

#[requires(input.inner_logic().0.static_drop_input())]
#[requires(*input.inner_logic().2==None)]
#[ensures(^input.inner_logic().1==*input.inner_logic().1)]
#[ensures(^data==*data)]
#[ensures(^input.inner_logic().2!=None && (^input.inner_logic().2).unwrap_logic().was_static())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().1.public().3))]
#[ensures(!(^input.inner_logic().2).unwrap_logic().reclaimed())]
fn cursor_static_drop_checked(data:&mut AtomicPtr<()>,_ptr:*const u8,_len:usize,input:Ghost<CursorDropInput>) {
    ghost! {let (_proof,_cursor,output)=input.into_inner();*output=Some(ViewEffect::Static);};
}

// Mode-correct generic registration: no native function/table value is produced.
// Exact callback contracts are those of the ordinary body-proved helpers above.
#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(shared_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==cursor_shared_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==cursor_shared_drop_checked.postcondition((data,ptr,len,input),()))]
fn cursor_shared_drop_registration<'a>()->Ghost<CursorDropSpec<'a>> {Ghost::conjure()}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(static_view_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==cursor_static_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<CursorDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==cursor_static_drop_checked.postcondition((data,ptr,len,input),()))]
fn cursor_static_drop_registration<'a>()->Ghost<CursorDropSpec<'a>> {Ghost::conjure()}

#[requires(value.api_view_valid() && value.view_accepts(*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1)]
#[ensures((*((^scope).observation())).0==if value.view_owned() {
    (*scope.inner_logic().observation()).0.remove(value.view_id())
} else {(*scope.inner_logic().observation()).0})]
#[ensures(^output!=None && (^output).unwrap_logic().valid(scope.inner_logic().public().3))]
#[ensures((^output).unwrap_logic().was_static()==!value.view_owned())]
#[ensures((^output).unwrap_logic().reclaimed()==(value.view_owned() && (*scope.inner_logic().observation()).0.len()==1))]
fn bytes_cursor_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut DetachedScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    let native=value.vtable.drop;
    let spec:Ghost<CursorDropSpec>=ghost! {match &*value.original_shared {
        OriginalSharedProof::Child(_)|OriginalSharedProof::View(_,_)=>cursor_shared_drop_registration().into_inner(),
        OriginalSharedProof::Empty(_)=>cursor_static_drop_registration().into_inner(),
        _=>{proof_assert!(false);panic!()},
    }};
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(value.original_shared.into_inner(),&mut scope.cursor,&mut **output)},spec);
}

impl Bytes {
    /// Exact affine owner/binding frame, independent of the displayed byte view.
    #[logic(prophetic)] fn same_api_owner(self,before:Bytes)->bool {
        match (before.original_shared.inner_logic(),self.original_shared.inner_logic()) {
            (OriginalSharedProof::Child(a),OriginalSharedProof::View(b,_))|
            (OriginalSharedProof::View(a,_),OriginalSharedProof::View(b,_))=>a==b,
            (OriginalSharedProof::Empty(a),OriginalSharedProof::Empty(b))=>a.binding==b.binding,
            _=>false,
        }
    }
}

#[requires(value.api_view_valid() && cursor_by<=value.len)]
#[ensures((^value).api_view_valid() && (^value).same_api_owner(*value))]
#[ensures((^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-cursor_by@)]
#[ensures((^value).ptr.addr_logic()@==value.ptr.addr_logic()@+cursor_by@)]
#[ensures((^value).view_content()==value.view_content().subsequence(cursor_by@,value.len@))]
#[ensures((^value).view_owned()==value.view_owned())]
#[ensures(value.view_owned() ==> (^value).view_id()==value.view_id() &&
    (^value).view_fraction()==value.view_fraction() && (^value).view_public()==value.view_public())]
unsafe fn inc_start_api(value:&mut Bytes,cursor_by:usize) {
    debug_assert!(value.len>=cursor_by,"internal: inc_start out of bounds");
    value.len-=cursor_by;
    let (bound,lease)=ghost! {match &*value.original_shared {
        OriginalSharedProof::Child(p)=>{
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
            (p.core.bound,crate::cursor_pointer::AdvanceLease::Live(full.borrow(&p.core.ticket.token)))
        },
        OriginalSharedProof::View(p,b)=>{
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
            (*b,crate::cursor_pointer::AdvanceLease::Live(full.borrow(&p.core.ticket.token)))
        },
        OriginalSharedProof::Empty(p)=>(p.bound,crate::cursor_pointer::AdvanceLease::Zero),
        _=>{proof_assert!(false);panic!()},
    }}.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,cursor_by,bound.borrow(),lease)};
    value.ptr=ptr;
    ghost! {
        let old=core::mem::replace(&mut *value.original_shared,OriginalSharedProof::Vacant);
        *value.original_shared=match old {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>
                OriginalSharedProof::View(p,shifted.into_inner()),
            OriginalSharedProof::Empty(mut p)=>{
                p.bound=shifted.into_inner();OriginalSharedProof::Empty(p)
            },
            _=>{proof_assert!(false);panic!()},
        };
    };
}

#[requires(value.api_view_valid() && count<=value.len)]
#[ensures((^value).api_view_valid() && (^value).same_api_owner(*value))]
#[ensures((^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-count@)]
#[ensures((^value).ptr.addr_logic()@==value.ptr.addr_logic()@+count@)]
#[ensures((^value).view_content()==value.view_content().subsequence(count@,value.len@))]
#[ensures((^value).view_owned()==value.view_owned())]
#[ensures(value.view_owned() ==> (^value).view_id()==value.view_id() &&
    (^value).view_fraction()==value.view_fraction() && (^value).view_public()==value.view_public())]
fn advance_api(value:&mut Bytes,count:usize) {
    assert!(count<=value.len,"cannot advance past `remaining`: {:?} <= {:?}",count,value.len);
    unsafe {inc_start_api(value,count)}
}

#[requires(value.api_view_valid())]
#[ensures(result==value.len && result@==value.view_content().len())]
fn remaining_api(value:&Bytes)->usize {value.len}

#[requires(value.api_view_valid())]
#[ensures(result@==value.view_content())]
fn read_api<'a>(value:&'a Bytes)->&'a [u8] {
    if value.len==0 {
        let bound=ghost! {match &*value.original_shared {
            OriginalSharedProof::Child(p)=>&p.core.bound,
            OriginalSharedProof::View(_,b)=>b,
            OriginalSharedProof::Empty(p)=>&p.bound,
            _=>{proof_assert!(false);panic!()},
        }};
        unsafe {physical_projection::borrow_empty(value.ptr,bound)}
    } else {
        let (proof,bound)=ghost! {match &*value.original_shared {
            OriginalSharedProof::Child(p)=>(p,&p.core.bound),OriginalSharedProof::View(p,b)=>(p,b),
            _=>{proof_assert!(false);panic!()},
        }}.split();
        let region=ghost! {
            let full:&FullBorrow<raw_vec::PhysicalRegion>=(*proof.core.physical).to_ref();
            full.borrow(&proof.core.ticket.token)
        };
        unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
    }
}

#[requires(value.api_view_valid())]
#[ensures(result@==value.view_content())]
fn chunk_api<'a>(value:&'a Bytes)->&'a [u8] {read_api(value)}

/// Logical prefix fold: no resource, ticket, state or native effect is returned.
#[logic]
#[requires(0<=count && count<=steps.len() && 0<=capacity)]
#[ensures(0<=result && result<=capacity)]
#[variant(count)]
fn consumed(steps:Seq<usize>,count:Int,capacity:Int)->Int {
    pearlite! {if count==0 {0} else {
        let previous=consumed(steps,count-1,capacity);
        if previous+steps[count-1]@<=capacity {previous+steps[count-1]@} else {capacity}
    }}
}

// Exact AQ slice body with a stronger, body-proved nonnull API entry post.
#[requires(source.view_valid() && source.view_accepts(*scope.inner_logic()))]
#[requires(range.start<=range.end && range.end<=source.len)]
#[ensures(result.api_view_valid())]
#[ensures(result.view_valid() && result.view_content()==source.view_content().subsequence(range.start@,range.end@))]
#[ensures(result.len@==range.end@-range.start@ && result.view_owned()==(range.start<range.end))]
#[ensures(result.view_accepts(^scope))]
#[ensures(result.view_owned() ==> result.view_public()==source.view_public())]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(if result.view_owned() {
    !(*scope.inner_logic().observation()).0.contains(result.view_id()) &&
    (*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())) &&
    (*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1
} else {(^scope).observation()==scope.inner_logic().observation()})]
fn slice_cursor_entry(source:&Bytes,range:core::ops::Range<usize>,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    use core::ops::{Bound,RangeBounds};
    let len=source.len;
    let view_begin=match range.start_bound() {
        Bound::Included(&n)=>n,
        Bound::Excluded(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Unbounded=>0,
    };
    let end=match range.end_bound() {
        Bound::Included(&n)=>n.checked_add(1).expect("out of range"),
        Bound::Excluded(&n)=>n,
        Bound::Unbounded=>len,
    };
    assert!(view_begin<=end,"range start must not be greater than end: {:?} <= {:?}",view_begin,end);
    assert!(end<=len,"range end out of bounds: {:?} <= {:?}",end,len);
    if end==view_begin {
        let bound=ghost! {match &*source.original_shared {
            OriginalSharedProof::Child(p)=>p.core.bound,OriginalSharedProof::View(_,b)=>*b,
            OriginalSharedProof::Empty(p)=>p.bound,_=>{proof_assert!(false);panic!()},
        }};
        let (ptr,_bound)=crate::view_pointer::wrapping_bounded(source.ptr,view_begin,bound.borrow());
        return new_empty_view(ptr);
    }
    let mut ret=clone_shared_view(source,ghost! {&mut **scope});
    ret.len=end-view_begin;
    let (bound,region)=ghost! {
        match &*ret.original_shared {
            OriginalSharedProof::View(p,b)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*p.core.physical).to_ref();
                (b,full.borrow(&p.core.ticket.token))
            },_=>{proof_assert!(false);panic!()},
        }
    }.split();
    let (ptr,shifted)=unsafe {crate::view_pointer::add_live(ret.ptr,view_begin,bound,region)};
    ret.ptr=ptr;
    ret.original_shared=ghost! {match ret.original_shared.into_inner() {
        OriginalSharedProof::View(p,_)=>OriginalSharedProof::View(p,shifted.into_inner()),
        _=>{proof_assert!(false);panic!()},
    }};
    ret
}

/// A runtime sequence of public advances preserves ownership, including the
/// owned zero-length final state. Only the actual final vtable Drop retires it.
#[requires(input@.len()>0)]
#[requires(a<=b && b@<=input@.len())]
#[ensures(result@==input@.subsequence(a@+consumed(steps@,steps@.len(),b@-a@),b@))]
pub(crate) fn cursor_scope(input:Box<[u8]>,a:usize,b:usize,steps:&[usize])->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let owner_id=snapshot!(owner.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*owner_id));
    let mut value=slice_cursor_entry(&owner,a..b,detached.borrow_mut());
    let value_id=snapshot!(value.view_id());
    let value_fraction=snapshot!(value.view_fraction());
    proof_assert!(value.view_owned() ==> *value_id!=*owner_id);
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && owner_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()==(a==b));
    proof_assert!((*detached.observation()).0.len()==(if a<b {1int}else{0int}));
    let mut i=0usize;
    #[invariant(i@<=steps@.len())]
    #[invariant(value.api_view_valid() && value.view_accepts(detached.inner_logic()))]
    #[invariant(value.view_owned()==(a<b))]
    #[invariant(value.view_owned() ==> value.view_id()==*value_id && value.view_fraction()==*value_fraction && value.view_public().3==*metadata)]
    #[invariant(value.view_content()==(*expected).subsequence(a@+consumed(steps@,i@,b@-a@),b@))]
    #[invariant(value.len@==b@-a@-consumed(steps@,i@,b@-a@))]
    #[variant(steps@.len()-i@)]
    while i<steps.len() {
        let cursor_by=core::cmp::min(steps[i],remaining_api(&value));
        advance_api(&mut value,cursor_by);
        i+=1;
    }
    let observed=chunk_api(&value).to_vec();
    let rest=remaining_api(&value);
    advance_api(&mut value,rest);
    assert_eq!(remaining_api(&value),0);
    assert!(chunk_api(&value).is_empty());
    proof_assert!(value.view_owned()==(a<b));
    proof_assert!(value.view_owned() ==> (*detached.observation()).0.get(value.view_id())==Some(Excl(value.view_fraction())));
    let saved_return=observed;
    let mut value_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(value,detached.borrow_mut(),value_receipt.borrow_mut());
    proof_assert!(value_receipt.inner_logic()!=None && value_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(value_receipt.inner_logic().unwrap_logic().reclaimed()==(a<b));
    proof_assert!(value_receipt.inner_logic().unwrap_logic().was_static()==(a==b));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()!=value_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}


// AT: allocation identity is independent of view extent and fresh ticket fraction.
// Pure map algebra only. No ticket, State, permission or lifecycle law is returned.
#[logic]
#[requires(old_id!=new_id)]
#[ensures(FMap::singleton(old_id,Excl(old_fraction)).insert(new_id,Excl(new_fraction)).remove(old_id)
    ==FMap::singleton(new_id,Excl(new_fraction)))]
fn singleton_replacement(old_id:Int,old_fraction:PositiveReal,new_id:Int,new_fraction:PositiveReal) {
    let removed=FMap::singleton(old_id,Excl(old_fraction)).insert(new_id,Excl(new_fraction)).remove(old_id);
    let remaining=FMap::singleton(new_id,Excl(new_fraction));
    proof_assert!(forall<key:Int> removed.get(key)==remaining.get(key));
    proof_assert!(removed.ext_eq(remaining));
}

impl SharedCore {
    #[logic(prophetic)] fn same_allocation(self,other:Self)->bool {
        self.shared==other.shared && self.bound==other.bound && self.capacity==other.capacity &&
        self.control==other.control && self.physical==other.physical && self.invariant==other.invariant
    }
}
impl Bytes {
    #[logic(prophetic)] fn has_allocation(self,core:SharedCore)->bool {
        match self.original_shared.inner_logic() {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>p.core.same_allocation(core),
            _=>false,
        }
    }
    #[logic(prophetic)] fn shares_view_allocation(self,source:Bytes)->bool {
        match source.original_shared.inner_logic() {
            OriginalSharedProof::Child(p)|OriginalSharedProof::View(p,_)=>self.has_allocation(p.core),
            _=>false,
        }
    }
}
#[requires(bounded_view(*input.inner_logic().0,input.inner_logic().1,ptr,len) && !ptr.is_null_logic())]
#[requires(shared==input.inner_logic().0.shared && input.inner_logic().0.accepts(*input.inner_logic().2))]
#[ensures(result.api_view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.public())]
#[ensures(result.view_content()==input.inner_logic().0.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
#[ensures(result.has_allocation(*input.inner_logic().0))]
#[ensures(input.inner_logic().0.accepts(^input.inner_logic().2))]
fn shallow_clone_owned_view_checked(shared:*mut Shared,ptr:*const u8,len:usize,input:Ghost<ViewArcInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let mut current=ghost! {SyncView::new().into_inner()};
    let release=ghost! {ReleaseSyncView::new().into_inner()};
    let mut ticket=ghost! {None::<lifecycle::Ticket<Payload>>};
    let old=field_event::increment_owned::<Shared,lifecycle::State<Payload>,_>(
        shared,ghost! {(*source.control).to_ref()},ghost! {&source.ticket.token},
        ghost! {(*source.invariant).to_ref()},cursor,
        ghost! {|state:&mut lifecycle::State<Payload>,c:&mut Committer<ModelAtomic,usize,Relaxed,Relaxed>| {
            *ticket=Some(lifecycle::State::on_register(Ghost::new(state),Ghost::new(c),
                ghost! {&*source.ticket},current.borrow_mut(),release).into_inner());
        }});
    let mut pointer_view=ghost! {SyncView::new().into_inner()};
    let (data,permission)=pointer_event::new_pointer(shared.cast(),pointer_view.borrow_mut());
    let binding=pointer_event::bind_read_only(&data,shared.cast(),permission);
    Bytes {ptr,len,data,vtable:shared_table_reification(),original_shared:ghost! {
        OriginalSharedProof::View(ChildProof {
            core:SharedCore {shared:source.shared,bound:source.bound,capacity:source.capacity,
                control:source.control,physical:source.physical,invariant:source.invariant,
                ticket:Ghost::new(ticket.into_inner().unwrap())},binding,
        },bound.into_inner())
    }}
}

#[requires(bounded_view(input.inner_logic().0.core,input.inner_logic().1,ptr,len) && !ptr.is_null_logic())]
#[requires(input.inner_logic().0.core.accepts(*input.inner_logic().2))]
#[requires(input.inner_logic().0.binding.inner_logic().model()==pointer_event::pointer_model(data))]
#[requires(input.inner_logic().0.binding.inner_logic().value()==input.inner_logic().0.core.shared as *mut ())]
#[ensures(result.api_view_valid() && result.view_owned() && result.ptr==ptr && result.len==len)]
#[ensures(result.view_bound()==input.inner_logic().1)]
#[ensures(result.view_public()==input.inner_logic().0.core.public())]
#[ensures(result.view_content()==input.inner_logic().0.core.content().subsequence(input.inner_logic().1@.unwrap_logic().2,input.inner_logic().1@.unwrap_logic().2+len@))]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>p.core.accepts(^input.inner_logic().2),_=>false})]
#[ensures((^input.inner_logic().2).model()==input.inner_logic().2.model() && (^input.inner_logic().2).public()==input.inner_logic().2.public())]
#[ensures(!(*input.inner_logic().2.observation()).0.contains(result.view_id()))]
#[ensures((*((^input.inner_logic().2).observation())).0==(*input.inner_logic().2.observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^input.inner_logic().2).observation())).1==(*input.inner_logic().2.observation()).1+1)]
#[ensures(result.has_allocation(input.inner_logic().0.core))]
#[ensures(input.inner_logic().0.core.accepts(^input.inner_logic().2))]
fn shared_owned_view_clone_checked(data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>)->Bytes {
    let (source,bound,cursor)=input.split();
    let shared=pointer_event::load_relaxed(data,ghost! {&*source.binding});
    shallow_clone_owned_view_checked(shared.cast(),ptr,len,ghost! {(&source.core,bound.into_inner(),cursor.into_inner())})
}
// Same native three-argument callback; only its proof specification is returned.
#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(shared_table().clone,result.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==shared_owned_view_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<ViewCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==shared_owned_view_clone_checked.postcondition((data,ptr,len,input),output))]
fn owned_view_clone_registration<'a>()->Ghost<ViewCloneSpec<'a>> {Ghost::conjure()}

#[requires(source.api_view_valid() && source.view_owned() && source.view_accepts(*scope.inner_logic()))]
#[ensures(result.api_view_valid() && result.view_owned() && result.view_content()==source.view_content())]
#[ensures(match result.original_shared.inner_logic() {OriginalSharedProof::View(_,_)=>true,_=>false})]
#[ensures(result.view_bound()==source.view_bound() && result.ptr==source.ptr && result.len==source.len)]
#[ensures(result.shares_view_allocation(*source))]
#[ensures(result.view_public()==source.view_public() && result.view_accepts(^scope) && source.view_accepts(^scope))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.view_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_owned_api(source:&Bytes,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let (proof,bound)=ghost! {match &*source.original_shared {
        OriginalSharedProof::Child(p)=>(p,p.core.bound),OriginalSharedProof::View(p,b)=>(p,*b),
        _=>{proof_assert!(false);panic!()},
    }}.split();
    let native=source.vtable.clone;
    let spec=ghost! {owned_view_clone_registration().into_inner()};
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*proof,bound.into_inner(),&mut scope.cursor)},spec)
}

/// Arbitrary finite clone succession. The current ticket/fraction may change;
/// the actual allocation and exact suffix view do not.
#[requires(input@.len()>0)]
#[requires(a<b && b@<=input@.len() && advance_by@<=b@-a@)]
#[ensures(result@==input@.subsequence(a@+advance_by@,b@))]
pub(crate) fn owned_view_clone_scope(input:Box<[u8]>,a:usize,b:usize,advance_by:usize,rounds:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let root_fraction=snapshot!(scope.root_fraction());
    let owner_id=snapshot!(owner.child_id());
    let owner_fraction=snapshot!(owner.child_fraction());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*owner_id));
    proof_assert!({singleton_replacement(*root_id,*root_fraction,*owner_id,*owner_fraction);true});
    proof_assert!((*detached.observation()).0==FMap::singleton(*owner_id,Excl(*owner_fraction)));
    let mut value=slice_cursor_entry(&owner,a..b,detached.borrow_mut());
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && !owner_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!({singleton_replacement(*owner_id,*owner_fraction,value.view_id(),value.view_fraction());true});
    proof_assert!((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())));
    advance_api(&mut value,advance_by);
    let allocation=snapshot!(value);
    let bound=snapshot!(value.view_bound());
    let ptr=snapshot!(value.ptr);
    let mut i=0usize;
    #[invariant(i<=rounds)]
    #[invariant(value.api_view_valid() && value.view_owned() && value.view_accepts(detached.inner_logic()))]
    #[invariant(value.shares_view_allocation(*allocation) && value.view_public().3==*metadata)]
    #[invariant(value.view_bound()==*bound && value.ptr==*ptr && value.len@==b@-a@-advance_by@)]
    #[invariant(value.view_content()==(*expected).subsequence(a@+advance_by@,b@))]
    #[invariant((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())))]
    #[variant(rounds@-i@)]
    while i<rounds {
        let old_id=snapshot!(value.view_id());
        let old_fraction=snapshot!(value.view_fraction());
        let next=clone_owned_api(&value,detached.borrow_mut());
        proof_assert!(next.view_id()!=*old_id);
        proof_assert!((*detached.observation()).0.len()==2);
        let mut retired=ghost! {None::<ViewEffect>};
        bytes_cursor_terminal_drop(value,detached.borrow_mut(),retired.borrow_mut());
        proof_assert!(retired.inner_logic()!=None && retired.inner_logic().unwrap_logic().valid(*metadata));
        proof_assert!(!retired.inner_logic().unwrap_logic().reclaimed());
        proof_assert!({singleton_replacement(*old_id,*old_fraction,next.view_id(),next.view_fraction());true});
        proof_assert!((*detached.observation()).0==FMap::singleton(next.view_id(),Excl(next.view_fraction())));
        value=next;
        i+=1;
    }
    let observed=chunk_api(&value).to_vec();
    let saved_return=observed;
    let mut final_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(value,detached.borrow_mut(),final_receipt.borrow_mut());
    proof_assert!(final_receipt.inner_logic()!=None && final_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(final_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}


// AU: ordinary, separately verified native function summary. The additional
// scope argument is ghost-erased; no owning resource is extracted from a model.
#[requires(source.api_view_valid() && source.view_owned() && source.view_accepts(*scope.inner_logic()))]
#[requires(amount<=source.len)]
#[ensures(result.api_view_valid() && result.view_owned())]
#[ensures(result.view_content()==source.view_content().subsequence(amount@,source.len@))]
#[ensures(result.len@==source.len@-amount@)]
#[ensures(result.ptr.addr_logic()@==source.ptr.addr_logic()@+amount@)]
#[ensures(result.shares_view_allocation(*source))]
#[ensures(result.view_bound()@==Some((source.view_bound()@.unwrap_logic().0,
    source.view_bound()@.unwrap_logic().1,source.view_bound()@.unwrap_logic().2+amount@)))]
#[ensures(result.view_public()==source.view_public() && result.view_accepts(^scope) && source.view_accepts(^scope))]
#[ensures((^scope).model()==scope.inner_logic().model() && (^scope).public()==scope.inner_logic().public())]
#[ensures(!(*scope.inner_logic().observation()).0.contains(result.view_id()))]
#[ensures((*((^scope).observation())).0==(*scope.inner_logic().observation()).0.insert(result.view_id(),Excl(result.view_fraction())))]
#[ensures((*((^scope).observation())).1==(*scope.inner_logic().observation()).1+1)]
fn clone_suffix_checked(source:&Bytes,amount:usize,mut scope:Ghost<&mut DetachedScope>)->Bytes {
    let mut result=clone_owned_api(source,ghost! {&mut **scope});
    advance_api(&mut result,amount);
    result
}

/// A caller uses a separately proved returned-owner summary. Neither the
/// helper body nor its ghost implementation is inlined at the call site.
#[requires(input@.len()>0)]
#[requires(a<b && b@<=input@.len())]
#[ensures(result@==input@.subsequence(a@+consumed(steps@,steps@.len(),b@-a@),b@))]
pub(crate) fn owned_view_call_scope(input:Box<[u8]>,a:usize,b:usize,steps:&[usize])->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let owner=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let root_fraction=snapshot!(scope.root_fraction());
    let owner_id=snapshot!(owner.child_id());
    let owner_fraction=snapshot!(owner.child_fraction());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(owner.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*owner_id));
    proof_assert!({singleton_replacement(*root_id,*root_fraction,*owner_id,*owner_fraction);true});
    proof_assert!((*detached.observation()).0==FMap::singleton(*owner_id,Excl(*owner_fraction)));
    let mut value=slice_cursor_entry(&owner,a..b,detached.borrow_mut());
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && !owner_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!({singleton_replacement(*owner_id,*owner_fraction,value.view_id(),value.view_fraction());true});
    proof_assert!((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())));
    let allocation=snapshot!(value);
    let mut i=0usize;
    #[invariant(i@<=steps@.len())]
    #[invariant(value.api_view_valid() && value.view_owned() && value.view_accepts(detached.inner_logic()))]
    #[invariant(value.shares_view_allocation(*allocation) && value.view_public().3==*metadata)]
    #[invariant(value.len@==b@-a@-consumed(steps@,i@,b@-a@))]
    #[invariant(value.view_content()==(*expected).subsequence(a@+consumed(steps@,i@,b@-a@),b@))]
    #[invariant((*detached.observation()).0==FMap::singleton(value.view_id(),Excl(value.view_fraction())))]
    #[variant(steps@.len()-i@)]
    while i<steps.len() {
        let old_id=snapshot!(value.view_id());
        let old_fraction=snapshot!(value.view_fraction());
        let amount=core::cmp::min(steps[i],remaining_api(&value));
        let next=clone_suffix_checked(&value,amount,detached.borrow_mut());
        proof_assert!(next.view_id()!=*old_id);
        proof_assert!((*detached.observation()).0.len()==2);
        let mut retired=ghost! {None::<ViewEffect>};
        bytes_cursor_terminal_drop(value,detached.borrow_mut(),retired.borrow_mut());
        proof_assert!(retired.inner_logic()!=None && retired.inner_logic().unwrap_logic().valid(*metadata));
        proof_assert!(!retired.inner_logic().unwrap_logic().reclaimed());
        proof_assert!({singleton_replacement(*old_id,*old_fraction,next.view_id(),next.view_fraction());true});
        proof_assert!((*detached.observation()).0==FMap::singleton(next.view_id(),Excl(next.view_fraction())));
        value=next;
        i+=1;
    }
    let observed=chunk_api(&value).to_vec();
    let saved_return=observed;
    let mut final_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(value,detached.borrow_mut(),final_receipt.borrow_mut());
    proof_assert!(final_receipt.inner_logic()!=None && final_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(final_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}


// AV: current raw suffix metadata wraps the one existing affine authority.
struct SuffixScope { scope:PromotionScope, view:raw_vec::BoundPtr }
impl SuffixScope {
    #[logic(prophetic)] fn valid(self)->bool {
        pearlite! {self.scope.valid() && self.scope.phase!=None && self.view.invariant() &&
            self.view@!=None && self.view@.unwrap_logic().0==self.scope.descriptor.base@.unwrap_logic().0 &&
            self.view@.unwrap_logic().1==self.scope.descriptor.capacity@ &&
            !self.view.raw_pointer().is_null_logic() && !self.scope.descriptor.base.raw_pointer().is_null_logic() &&
            self.view.current_address()==self.view.raw_pointer().addr_logic()@ &&
            self.view.current_address()==self.scope.descriptor.base.current_address()+self.view@.unwrap_logic().2}
    }
    #[logic(prophetic)] fn matches(self,ptr:*const u8,len:usize,model:ModelAtomicPtr<()>,table:&'static Vtable)->bool {pearlite! {
        self.valid() && ptr==self.view.raw_pointer() as *const u8 &&
        self.view@.unwrap_logic().2+len@==self.scope.descriptor.capacity@ &&
        model==*self.scope.descriptor.model && table==self.scope.descriptor.table()
    }}
    #[logic(prophetic)] fn root_valid(self,value:Bytes)->bool {pearlite! {
        self.matches(value.ptr,value.len,pointer_event::pointer_model(&value.data),value.vtable) &&
        match value.original_shared.inner_logic() {OriginalSharedProof::Root(d)=>d==self.scope.descriptor,_=>false}
    }}
    #[logic] fn content(self)->Seq<u8> {pearlite! {
        (*self.scope.descriptor.expected).subsequence(self.view@.unwrap_logic().2,self.scope.descriptor.capacity@)
    }}
    #[check(ghost)]
    #[requires(scope.inner_logic().valid() && scope.inner_logic().is_raw())]
    #[ensures(result.inner_logic().valid() && result.inner_logic().scope==scope.inner_logic())]
    #[ensures(result.inner_logic().view==scope.inner_logic().descriptor.base)]
    fn new(scope:Ghost<PromotionScope>)->Ghost<Self> {
        ghost! {
            Self {view:scope.descriptor.base,scope:scope.into_inner()}
        }
    }
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[requires(amount<=value.len)]
#[ensures((^scope).root_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
unsafe fn inc_start_suffix(value:&mut Bytes,amount:usize,mut scope:Ghost<&mut SuffixScope>) {
    debug_assert!(value.len>=amount,"internal: inc_start out of bounds");
    value.len-=amount;
    let (bound,lease)=ghost! {
        let raw=match scope.scope.phase.as_ref().unwrap() {Phase::Raw(raw)=>raw,_=>{proof_assert!(false);panic!()}};
        (&scope.view,crate::cursor_pointer::AdvanceLease::Live(&raw.physical))
    }.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,amount,bound,lease)};
    value.ptr=ptr;
    ghost! {scope.view=shifted.into_inner();};
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[requires(amount<=value.len)]
#[ensures((^scope).root_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
fn advance_suffix(value:&mut Bytes,amount:usize,scope:Ghost<&mut SuffixScope>) {
    assert!(amount<=value.len,"cannot advance past `remaining`: {:?} <= {:?}",amount,value.len);
    unsafe {inc_start_suffix(value,amount,scope)}
}

type SuffixCloneInput<'a>=(&'a RootDescriptor,raw_vec::BoundPtr,&'a mut SuffixScope);
type SuffixCloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<SuffixCloneInput<'a>>)->Bytes;
#[logic(prophetic)]
fn suffix_clone_result(d:RootDescriptor,view:raw_vec::BoundPtr,s:SuffixScope,result:Bytes)->bool {
    pearlite! {s.valid() && s.scope.is_shared() && s.scope.descriptor==d && s.view==view &&
        result.api_view_valid() && result.view_owned() && result.view_bound()==view &&
        result.ptr==view.raw_pointer() as *const u8 && result.len@+view@.unwrap_logic().2==d.capacity@ &&
        result.view_content()==(*d.expected).subsequence(view@.unwrap_logic().2,d.capacity@) &&
        result.view_public()==s.scope.cursor_public() &&
        match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>
            match s.scope.phase {Some(Phase::Shared(shared))=>p.core.accepts(shared.cursor),_=>false},_=>false} &&
        (*s.scope.observation()).0==FMap::singleton(s.scope.root_id(),Excl(s.scope.root_fraction())).insert(
            result.view_id(),Excl(result.view_fraction())) && s.scope.root_id()!=result.view_id()}
}

#[requires(input.inner_logic().2.valid() && input.inner_logic().2.scope.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().2.matches(offset,len,pointer_event::pointer_model(data),input.inner_logic().0.table()))]
#[requires(expected==input.inner_logic().0.word() && buf==input.inner_logic().0.base.raw_pointer())]
#[ensures(suffix_clone_result(*input.inner_logic().0,input.inner_logic().1,^input.inner_logic().2,result))]
fn shallow_clone_suffix_checked(data:&AtomicPtr<()>,expected:*mut (),buf:*mut u8,offset:*const u8,len:usize,input:Ghost<SuffixCloneInput>)->Bytes {
    let (descriptor,view,mut scope)=input.split();
    let mut raw=ghost! {match scope.scope.phase.take().unwrap() {Phase::Raw(raw)=>raw,_=>{proof_assert!(false);panic!()}}};
    let distance=unsafe {crate::suffix_pointer::distance(offset,buf,view.borrow(),ghost! {&descriptor.base},ghost! {&raw.physical})};
    let cap=distance as usize + len;
    let mut control_view=ghost! {SyncView::new().into_inner()};
    let (ref_cnt,count_permission)=field_event::new(2,control_view.borrow_mut());
    let count_view=snapshot!(*control_view);
    let boxed=Box::new(Shared {buf,cap,ref_cnt});
    let (shared,shared_owner)=boxed_alignment::into_raw_aligned(boxed);
    let address=crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(address,core::mem::align_of::<Shared>());
    debug_assert!(address & 1usize==0usize);
    let (own_borrow,current_borrow)=ghost! {
        let raw=&mut *raw; (&mut raw.own,&mut raw.current)
    }.split();
    let exchanged=owned_pointer::exchange_singleton(data,expected,shared.cast(),own_borrow,current_borrow);
    match exchanged {
        Ok(actual)=>{
            debug_assert!(actual==expected);
            let (recovery,region,root_own,root_current)=ghost! {
                let raw=raw.into_inner(); (raw.recovery,raw.physical,raw.own,raw.current)
            }.split();
            let lifetime=ghost! {LifetimeToken::new()};
            let lifetime_id=snapshot!(lifetime.lft());
            let (control_full,control_end)=FullBorrow::new(field_event::own_control(shared,shared_owner),lifetime_id);
            let (physical_full,physical_end)=FullBorrow::new(region,lifetime_id);
            let control=ghost! {GhostShared::new(control_full).into_inner()};
            let physical=ghost! {GhostShared::new(physical_full).into_inner()};
            let payload=ghost! {Payload {recovery:recovery.into_inner(),physical_end:physical_end.into_inner(),control_end:control_end.into_inner(),
                physical,control,base:descriptor.base,capacity:cap,len:descriptor.capacity,expected:descriptor.expected}};
            let initialized=lifecycle::State::initialize_pair(count_permission,count_view,payload,lifetime);
            let (state,root_ticket,child_ticket)=initialized.split();
            let permission:Ghost<&Perm<*const Shared>>=ghost! {
                let full:&FullBorrow<field_event::OwnedControl<Shared>>=(*control).to_ref();
                let owner=full.borrow(&root_ticket.token); &**owner.owner
            };
            let shared_ref=unsafe {Perm::as_ref(shared,permission)};
            let (field_invariant,cursor)=field_event::ScopedFieldInvariant::bind(&shared_ref.ref_cnt,state).split();
            let invariant=GhostShared::new(field_invariant);
            let root=ghost! {SharedCore {shared,bound:descriptor.base,capacity:cap,control,physical,invariant,ticket:root_ticket}};
            let child=ghost! {SharedCore {shared,bound:descriptor.base,capacity:cap,control,physical,invariant,ticket:child_ticket}};
            let mut child_view=ghost! {SyncView::new().into_inner()};
            let (child_data,child_permission)=pointer_event::new_pointer(shared.cast(),child_view.borrow_mut());
            let binding=pointer_event::bind_read_only(&child_data,shared.cast(),child_permission);
            ghost! {scope.scope.phase=Some(Phase::Shared(SharedPhase {root:root.into_inner(),cursor:cursor.into_inner(),own:root_own.into_inner(),current:root_current.into_inner()}));};
            Bytes {ptr:offset,len,data:child_data,vtable:shared_table_reification(),original_shared:ghost! {
                OriginalSharedProof::View(ChildProof {core:child.into_inner(),binding},view.into_inner())
            }}
        }
        Err(_actual)=>{
            // Native Box::from_raw / forget(candidate) / shallow_clone_arc loser
            // branch is retained in the source map, and impossible in this scope.
            proof_assert!(false);unreachable!("owned singleton excludes first-CAS failure")
        }
    }
}

#[requires(input.inner_logic().2.valid() && input.inner_logic().2.scope.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().2.matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize==0usize)]
#[ensures(suffix_clone_result(*input.inner_logic().0,input.inner_logic().1,^input.inner_logic().2,result))]
fn even_suffix_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<SuffixCloneInput>)->Bytes {
    let (descriptor,view,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                _=>{proof_assert!(false);panic!()},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        proof_assert!(false);unreachable!("first clone raw phase excludes existing ARC branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let buf=tag_specs::clear_low_bit(stored,ghost! {&descriptor.base});
        shallow_clone_suffix_checked(data,stored,buf,offset,len,ghost! {(*descriptor,view.into_inner(),&mut **scope)})
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().clone,result.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==even_suffix_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==even_suffix_clone_checked.postcondition((data,ptr,len,input),output))]
fn even_suffix_clone_registration<'a>()->Ghost<SuffixCloneSpec<'a>> {Ghost::conjure()}

#[requires(input.inner_logic().2.valid() && input.inner_logic().2.scope.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().2.matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic() & 1usize!=0usize)]
#[ensures(suffix_clone_result(*input.inner_logic().0,input.inner_logic().1,^input.inner_logic().2,result))]
fn odd_suffix_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<SuffixCloneInput>)->Bytes {
    let (descriptor,view,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                _=>{proof_assert!(false);panic!()},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored) & 1usize;
    if kind==0usize {
        proof_assert!(false);unreachable!("first clone raw phase excludes existing ARC branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let buf=stored.cast::<u8>();
        shallow_clone_suffix_checked(data,stored,buf,offset,len,ghost! {(*descriptor,view.into_inner(),&mut **scope)})
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().clone,result.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==odd_suffix_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==odd_suffix_clone_checked.postcondition((data,ptr,len,input),output))]
fn odd_suffix_clone_registration<'a>()->Ghost<SuffixCloneSpec<'a>> {Ghost::conjure()}

#[requires(scope.inner_logic().root_valid(*source) && scope.inner_logic().scope.is_raw())]
#[ensures((^scope).root_valid(*source))]
#[ensures(suffix_clone_result(scope.inner_logic().scope.descriptor,scope.inner_logic().view,^scope,result))]
fn clone_suffix_root(source:&Bytes,mut scope:Ghost<&mut SuffixScope>)->Bytes {
    let descriptor=ghost! {match &*source.original_shared {OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address & 1usize==0usize {even_suffix_clone_registration().into_inner()}
        else {odd_suffix_clone_registration().into_inner()}
    };
    let native=source.vtable.clone;
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),ghost! {(*descriptor,scope.view,&mut **scope)},spec)
}

type SuffixRootDropInput<'a>=(SuffixScope,&'a mut Option<DetachedScope>,&'a mut Option<Completion>);
type SuffixRootDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<SuffixRootDropInput<'a>>);
#[requires(input.inner_logic().0.valid() && input.inner_logic().0.scope.is_shared())]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),input.inner_logic().0.scope.descriptor.table()))]
#[requires((*input.inner_logic().0.scope.observation()).0.len()>1)]
#[requires(*input.inner_logic().1==None && *input.inner_logic().2==None)]
#[ensures(^input.inner_logic().1!=None)]
#[ensures((^input.inner_logic().1).unwrap_logic().model()==input.inner_logic().0.scope.cursor_model())]
#[ensures((^input.inner_logic().1).unwrap_logic().public()==input.inner_logic().0.scope.cursor_public())]
#[ensures((*((^input.inner_logic().1).unwrap_logic().observation())).0==(*input.inner_logic().0.scope.observation()).0.remove(input.inner_logic().0.scope.root_id()))]
#[ensures((*((^input.inner_logic().1).unwrap_logic().observation())).1==(*input.inner_logic().0.scope.observation()).1)]
#[ensures(^input.inner_logic().2!=None && !(^input.inner_logic().2).unwrap_logic().reclaimed())]
#[ensures((^input.inner_logic().2).unwrap_logic().valid(input.inner_logic().0.scope.root_metadata()))]
fn suffix_root_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<SuffixRootDropInput>) {
    let (scope,mut detached,output)=input.split();
    let (root,mut cursor,own,_current)=ghost! {
        let shared=match scope.into_inner().scope.phase.unwrap() {Phase::Shared(s)=>s,_=>{proof_assert!(false);panic!()}};
        (shared.root,shared.cursor,shared.own,shared.current)
    }.split();
    let (word,_stamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word) & 1usize;
    if kind==0usize {
        release_core(word.cast(),root,cursor.borrow_mut(),output);
        ghost! {**detached=Some(DetachedScope {cursor:cursor.into_inner()});};
    } else {
        proof_assert!(false);unreachable!("updated root history excludes raw free branch")
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixRootDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==suffix_root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixRootDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==suffix_root_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_suffix_drop_registration<'a>()->Ghost<SuffixRootDropSpec<'a>> {Ghost::conjure()}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixRootDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==suffix_root_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<SuffixRootDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==suffix_root_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_suffix_drop_registration<'a>()->Ghost<SuffixRootDropSpec<'a>> {Ghost::conjure()}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().scope.is_shared())]
#[requires((*scope.inner_logic().scope.observation()).0.len()>1)]
#[requires(*detached.inner_logic()==None && *output.inner_logic()==None)]
#[ensures(^detached!=None)]
#[ensures((^detached).unwrap_logic().model()==scope.inner_logic().scope.cursor_model())]
#[ensures((^detached).unwrap_logic().public()==scope.inner_logic().scope.cursor_public())]
#[ensures((*((^detached).unwrap_logic().observation())).0==(*scope.inner_logic().scope.observation()).0.remove(scope.inner_logic().scope.root_id()))]
#[ensures((*((^detached).unwrap_logic().observation())).1==(*scope.inner_logic().scope.observation()).1)]
#[ensures(^output!=None && !(^output).unwrap_logic().reclaimed())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().scope.root_metadata()))]
fn bytes_suffix_root_terminal_drop(mut value:Bytes,scope:Ghost<SuffixScope>,mut detached:Ghost<&mut Option<DetachedScope>>,mut output:Ghost<&mut Option<Completion>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {match value.original_shared.into_inner() {OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()}}};
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address & 1usize==0usize {even_suffix_drop_registration().into_inner()}
        else {odd_suffix_drop_registration().into_inner()}
    };
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **detached,&mut **output)},spec);
}

/// Actual raw suffix advance precedes first promotion. Root drops before the
/// surviving suffix is read; zero-length suffixes still own the allocation.
#[requires(input@.len()>0 && amount@<=input@.len())]
#[ensures(result@==input@.subsequence(amount@,input@.len()))]
pub(crate) fn promotable_suffix_scope(input:Box<[u8]>,amount:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut root,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_suffix(&mut root,amount,suffix.borrow_mut());
    let child=clone_suffix_root(&root,suffix.borrow_mut());
    let root_id=snapshot!(suffix.scope.root_id());
    let child_id=snapshot!(child.view_id());
    let before=snapshot!((*suffix.scope.observation()).0);
    let metadata=snapshot!(child.view_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_suffix_root_terminal_drop(root,suffix,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*child_id));
    proof_assert!(child.view_accepts(*detached));
    let observed=chunk_api(&child).to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(child,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}

// AX: source-shaped, body-proved joins for the actual From<Box>, Buf::advance,
// chunk read, and normal raw Drop path.  This selected client excludes the
// empty-box branch; its precondition is the native From<Box> nonempty arm.
impl Bytes {
    #[ensures(result==self.len)]
    fn len(&self)->usize { self.len }
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[ensures(result@==scope.inner_logic().content())]
#[ensures(result@.len()==value.len@)]
fn raw_suffix_as_slice<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    let (bound,region)=ghost! {
        let raw=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>raw,
            _=>{proof_assert!(false);panic!()},
        };
        (&scope.view,&raw.physical)
    }.split();
    unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[ensures(result@==scope.inner_logic().content())]
fn chunk_raw_suffix<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    raw_suffix_as_slice(value,scope)
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[requires(amount<=value.len)]
#[ensures((^scope).root_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
unsafe fn inc_start_raw_suffix(value:&mut Bytes,amount:usize,mut scope:Ghost<&mut SuffixScope>) {
    debug_assert!(value.len>=amount,"internal: inc_start out of bounds");
    value.len-=amount;
    let (bound,lease)=ghost! {
        let raw=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>raw,
            _=>{proof_assert!(false);panic!()},
        };
        (&scope.view,crate::cursor_pointer::AdvanceLease::Live(&raw.physical))
    }.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,amount,bound,lease)};
    value.ptr=ptr;
    ghost! {scope.view=shifted.into_inner();};
}

#[requires(scope.inner_logic().root_valid(*value) && scope.inner_logic().scope.is_raw())]
#[requires(amount<=value.len)]
#[ensures((^scope).root_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
fn advance_raw_suffix(value:&mut Bytes,amount:usize,scope:Ghost<&mut SuffixScope>) {
    assert!(
        amount<=value.len(),
        "cannot advance past `remaining`: {:?} <= {:?}",
        amount,
        value.len(),
    );
    unsafe {inc_start_raw_suffix(value,amount,scope)}
}

type RawSuffixFreeInput=(RootDescriptor,raw_vec::BoundPtr,raw_vec::Recovery,raw_vec::PhysicalRegion);

/// The actual free_boxed_slice operation at a raw suffix view.  The consumed
/// capabilities describe the original full allocation; the native view may
/// start anywhere from zero through one-past, but its remaining length closes
/// the exact original capacity equation.
#[requires(input.inner_logic().0.valid())]
#[requires(input.inner_logic().1.invariant() && input.inner_logic().1@!=None)]
#[requires(input.inner_logic().1@.unwrap_logic().0==input.inner_logic().0.base@.unwrap_logic().0)]
#[requires(input.inner_logic().1@.unwrap_logic().1==input.inner_logic().0.capacity@)]
#[requires(input.inner_logic().1@.unwrap_logic().2+len@==input.inner_logic().0.capacity@)]
#[requires(pointer==input.inner_logic().1.raw_pointer() as *const u8)]
#[requires(base==input.inner_logic().0.base.raw_pointer())]
#[requires(!pointer.is_null_logic() && !base.is_null_logic())]
#[requires(input.inner_logic().1.current_address()==pointer.addr_logic()@)]
#[requires(input.inner_logic().0.base.current_address()==base.addr_logic()@)]
#[requires(input.inner_logic().1.current_address()==input.inner_logic().0.base.current_address()+input.inner_logic().1@.unwrap_logic().2)]
#[requires(input.inner_logic().2.invariant() && input.inner_logic().3.invariant())]
#[requires(input.inner_logic().2.namespace()==input.inner_logic().0.base@.unwrap_logic().0)]
#[requires(input.inner_logic().2.capacity()==input.inner_logic().0.capacity@)]
#[requires(input.inner_logic().3.namespace()==input.inner_logic().2.namespace())]
#[requires(input.inner_logic().3.resource_id()==input.inner_logic().2.namespace())]
#[requires(input.inner_logic().3.capacity()==input.inner_logic().0.capacity@)]
#[requires(input.inner_logic().3.lo()==0 && input.inner_logic().3.hi()==input.inner_logic().0.capacity@)]
#[requires(forall<i:Int> 0<=i && i<input.inner_logic().0.capacity@ ==>
    input.inner_logic().3.slot(i)==Some(Some((*input.inner_logic().0.expected)[i])))]
#[ensures(result.inner_logic().namespace()==input.inner_logic().0.base@.unwrap_logic().0)]
#[ensures(result.inner_logic().pointer()==base)]
#[ensures(result.inner_logic().size()==input.inner_logic().0.capacity@)]
#[ensures(result.inner_logic().align()==1 && result.inner_logic().allocated())]
unsafe fn free_raw_suffix_checked(base:*mut u8,pointer:*const u8,len:usize,
    input:Ghost<RawSuffixFreeInput>)->Ghost<physical_projection::FreeReceipt> {
    let (descriptor,view,recovery,physical)=input.split();
    let distance=unsafe {crate::suffix_pointer::distance(
        pointer,base,view.borrow(),ghost! {&descriptor.base},physical.borrow())};
    let capacity=distance as usize+len;
    proof_assert!(capacity==descriptor.capacity);
    unsafe {physical_projection::deallocate(
        base,capacity,ghost! {descriptor.base},ghost! {(recovery.into_inner(),physical.into_inner())})}
}

type RawSuffixDropInput<'a>=(SuffixScope,&'a mut Option<physical_projection::FreeReceipt>);
type RawSuffixDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<RawSuffixDropInput<'a>>);

#[requires(input.inner_logic().0.valid() && input.inner_logic().0.scope.is_raw())]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),
    even_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize==0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None)]
#[ensures((^input.inner_logic().1).unwrap_logic().namespace()==input.inner_logic().0.scope.descriptor.base@.unwrap_logic().0)]
#[ensures((^input.inner_logic().1).unwrap_logic().pointer()==input.inner_logic().0.scope.descriptor.base.raw_pointer())]
#[ensures((^input.inner_logic().1).unwrap_logic().size()==input.inner_logic().0.scope.descriptor.capacity@)]
#[ensures((^input.inner_logic().1).unwrap_logic().align()==1 && (^input.inner_logic().1).unwrap_logic().allocated())]
fn even_raw_suffix_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,
    input:Ghost<RawSuffixDropInput>) {
    let (mut suffix,mut output)=input.split();
    let (descriptor,view,mut raw)=ghost! {
        let mut suffix=suffix.into_inner();
        let descriptor=suffix.scope.descriptor;
        let view=suffix.view;
        let raw=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>raw,
            _=>{proof_assert!(false);panic!()},
        };
        (descriptor,view,raw)
    }.split();
    let (recovery,physical,own)=ghost! {
        let raw=raw.into_inner();
        (raw.recovery,raw.physical,raw.own)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    proof_assert!(word==descriptor.word());
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        // The native ARC release arm is impossible for this fresh raw owner,
        // whose affine history contains only a tagged promotable word.
        proof_assert!(false);
        unreachable!("fresh raw suffix cannot enter the ARC release branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let base=tag_specs::clear_low_bit(word,ghost! {&descriptor.base});
        proof_assert!(base==descriptor.base.raw_pointer());
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(receipt.into_inner());};
    }
}

#[requires(input.inner_logic().0.valid() && input.inner_logic().0.scope.is_raw())]
#[requires(input.inner_logic().0.matches(offset,len,pointer_event::pointer_model(data),
    odd_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize!=0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None)]
#[ensures((^input.inner_logic().1).unwrap_logic().namespace()==input.inner_logic().0.scope.descriptor.base@.unwrap_logic().0)]
#[ensures((^input.inner_logic().1).unwrap_logic().pointer()==input.inner_logic().0.scope.descriptor.base.raw_pointer())]
#[ensures((^input.inner_logic().1).unwrap_logic().size()==input.inner_logic().0.scope.descriptor.capacity@)]
#[ensures((^input.inner_logic().1).unwrap_logic().align()==1 && (^input.inner_logic().1).unwrap_logic().allocated())]
fn odd_raw_suffix_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,
    input:Ghost<RawSuffixDropInput>) {
    let (mut suffix,mut output)=input.split();
    let (descriptor,view,mut raw)=ghost! {
        let mut suffix=suffix.into_inner();
        let descriptor=suffix.scope.descriptor;
        let view=suffix.view;
        let raw=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>raw,
            _=>{proof_assert!(false);panic!()},
        };
        (descriptor,view,raw)
    }.split();
    let (recovery,physical,own)=ghost! {
        let raw=raw.into_inner();
        (raw.recovery,raw.physical,raw.own)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    proof_assert!(word==descriptor.word());
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        proof_assert!(false);
        unreachable!("fresh raw suffix cannot enter the ARC release branch")
    } else {
        debug_assert_eq!(kind,1usize);
        let base=word.cast::<u8>();
        proof_assert!(base==descriptor.base.raw_pointer());
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(receipt.into_inner());};
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RawSuffixDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==even_raw_suffix_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RawSuffixDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==even_raw_suffix_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_raw_suffix_drop_registration<'a>()->Ghost<RawSuffixDropSpec<'a>> {Ghost::conjure()}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RawSuffixDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==odd_raw_suffix_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RawSuffixDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==odd_raw_suffix_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_raw_suffix_drop_registration<'a>()->Ghost<RawSuffixDropSpec<'a>> {Ghost::conjure()}

#[requires(scope.inner_logic().root_valid(value) && scope.inner_logic().scope.is_raw())]
#[requires(*output.inner_logic()==None)]
#[ensures(^output!=None)]
#[ensures((^output).unwrap_logic().namespace()==scope.inner_logic().scope.descriptor.base@.unwrap_logic().0)]
#[ensures((^output).unwrap_logic().pointer()==scope.inner_logic().scope.descriptor.base.raw_pointer())]
#[ensures((^output).unwrap_logic().size()==scope.inner_logic().scope.descriptor.capacity@)]
#[ensures((^output).unwrap_logic().align()==1 && (^output).unwrap_logic().allocated())]
fn bytes_raw_suffix_terminal_drop(mut value:Bytes,scope:Ghost<SuffixScope>,
    mut output:Ghost<&mut Option<physical_projection::FreeReceipt>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {
        match value.original_shared.into_inner() {
            OriginalSharedProof::Root(descriptor)=>descriptor,
            _=>{proof_assert!(false);panic!()},
        }
    };
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address&1usize==0usize {
            even_raw_suffix_drop_registration().into_inner()
        } else {
            odd_raw_suffix_drop_registration().into_inner()
        }
    };
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(scope.into_inner(),&mut **output)},spec);
}

/// Exact normal-path composition: original nonempty Box, actual suffix
/// advance, branch-aware physical read, then normal raw Drop of the owner.
#[requires(input@.len()>0 && amount@<=input@.len())]
#[ensures(result@==input@.subsequence(amount@,input@.len()))]
pub(crate) fn raw_suffix_scope(input:Box<[u8]>,amount:usize)->Vec<u8> {
    let (mut value,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_raw_suffix(&mut value,amount,suffix.borrow_mut());
    let result=chunk_raw_suffix(&value,suffix.borrow()).to_vec();
    let mut receipt=ghost! {None::<physical_projection::FreeReceipt>};
    bytes_raw_suffix_terminal_drop(value,suffix,receipt.borrow_mut());
    proof_assert!(receipt.inner_logic()!=None && receipt.inner_logic().unwrap_logic().allocated());
    result
}

// AY: one affine Root view across raw and promoted phases.  All inherited
// suffix contracts remain unchanged; Shared windows need not close capacity.
impl SuffixScope {
    #[logic(prophetic)] fn root_view_matches(self,ptr:*const u8,len:usize,model:ModelAtomicPtr<()>,table:&'static Vtable)->bool {pearlite! {
        self.valid() && ptr==self.view.raw_pointer() as *const u8 &&
        0<=self.view@.unwrap_logic().2 && self.view@.unwrap_logic().2+len@<=self.scope.descriptor.capacity@ &&
        (self.scope.is_raw() ==> self.view@.unwrap_logic().2+len@==self.scope.descriptor.capacity@) &&
        model==*self.scope.descriptor.model && table==self.scope.descriptor.table()
    }}
    #[logic(prophetic)] fn root_view_valid(self,value:Bytes)->bool {pearlite! {
        self.root_view_matches(value.ptr,value.len,pointer_event::pointer_model(&value.data),value.vtable) &&
        match value.original_shared.inner_logic() {OriginalSharedProof::Root(d)=>d==self.scope.descriptor,_=>false}
    }}
    #[logic] fn root_view_content(self,len:usize)->Seq<u8> {pearlite! {
        (*self.scope.descriptor.expected).subsequence(self.view@.unwrap_logic().2,self.view@.unwrap_logic().2+len@)
    }}
}

#[requires(scope.inner_logic().root_view_valid(*value) && amount<=value.len)]
#[ensures((^scope).root_view_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
unsafe fn inc_start_root_view(value:&mut Bytes,amount:usize,mut scope:Ghost<&mut SuffixScope>) {
    debug_assert!(value.len>=amount,"internal: inc_start out of bounds");
    value.len-=amount;
    let (bound,lease)=ghost! {
        let region=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>&raw.physical,
            Phase::Shared(shared)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*shared.root.physical).to_ref();
                full.borrow(&shared.root.ticket.token)
            },
        };
        (&scope.view,crate::cursor_pointer::AdvanceLease::Live(region))
    }.split();
    let (ptr,shifted)=unsafe {crate::cursor_pointer::add(value.ptr,amount,bound,lease)};
    value.ptr=ptr;
    ghost! {scope.view=shifted.into_inner();};
}

#[requires(scope.inner_logic().root_view_valid(*value) && amount<=value.len)]
#[ensures((^scope).root_view_valid(^value) && (^scope).scope==scope.inner_logic().scope)]
#[ensures((^value).original_shared==value.original_shared && (^value).data==value.data && (^value).vtable==value.vtable)]
#[ensures((^value).len@==value.len@-amount@ && (^value).ptr.addr_logic()@==value.ptr.addr_logic()@+amount@)]
#[ensures((^scope).view@==Some((scope.inner_logic().view@.unwrap_logic().0,
    scope.inner_logic().view@.unwrap_logic().1,scope.inner_logic().view@.unwrap_logic().2+amount@)))]
fn advance_root_view(value:&mut Bytes,amount:usize,scope:Ghost<&mut SuffixScope>) {
    assert!(amount<=value.len(),"cannot advance past `remaining`: {:?} <= {:?}",amount,value.len());
    unsafe {inc_start_root_view(value,amount,scope)}
}

#[requires(scope.inner_logic().root_view_valid(*value))]
#[ensures(result@==scope.inner_logic().root_view_content(value.len))]
#[ensures(result@.len()==value.len@)]
fn root_view_as_slice<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    let (bound,region)=ghost! {
        let region=match scope.scope.phase.as_ref().unwrap() {
            Phase::Raw(raw)=>&raw.physical,
            Phase::Shared(shared)=>{
                let full:&FullBorrow<raw_vec::PhysicalRegion>=(*shared.root.physical).to_ref();
                full.borrow(&shared.root.ticket.token)
            },
        };
        (&scope.view,region)
    }.split();
    unsafe {physical_projection::borrow(value.ptr,value.len,bound,region)}
}

#[requires(scope.inner_logic().root_view_valid(*value))]
#[ensures(result@==scope.inner_logic().root_view_content(value.len))]
fn chunk_root_view<'a>(value:&'a Bytes,scope:Ghost<&'a SuffixScope>)->&'a [u8] {
    root_view_as_slice(value,scope)
}

enum RootDropEffect { Raw(physical_projection::FreeReceipt), Shared(DetachedScope,Completion) }
impl RootDropEffect {
    #[logic(prophetic)] fn valid_for(self,before:SuffixScope)->bool {pearlite! {match self {
        Self::Raw(receipt)=>before.scope.is_raw() &&
            receipt.namespace()==before.scope.descriptor.base@.unwrap_logic().0 &&
            receipt.pointer()==before.scope.descriptor.base.raw_pointer() &&
            receipt.size()==before.scope.descriptor.capacity@ && receipt.align()==1 && receipt.allocated(),
        Self::Shared(detached,completion)=>before.scope.is_shared() &&
            detached.model()==before.scope.cursor_model() && detached.public()==before.scope.cursor_public() &&
            (*detached.observation()).0==(*before.scope.observation()).0.remove(before.scope.root_id()) &&
            (*detached.observation()).1==(*before.scope.observation()).1 &&
            completion.valid(before.scope.root_metadata()) &&
            completion.reclaimed()==((*before.scope.observation()).0.len()==1) &&
            (completion.reclaimed() ==> (*detached.observation()).0.len()==0),
    }}}
    #[logic] fn reclaimed(self)->bool {match self {Self::Raw(_)=>true,Self::Shared(_,c)=>c.reclaimed()}}
}

// No resource getter: the callback consumes the existing phase and splits its
// owned atomic permission from the actual remaining affine capabilities.
enum RootDropRemainder {
    Raw(raw_vec::Recovery,raw_vec::PhysicalRegion),
    Shared(SharedCore,Cursor),
}
type RootViewDropInput<'a>=(SuffixScope,&'a mut Option<RootDropEffect>);
type RootViewDropSpec<'a>=fn(&'a mut AtomicPtr<()>,*const u8,usize,Ghost<RootViewDropInput<'a>>);

#[requires(scope.inner_logic().root_view_valid(value))]
#[requires(*output.inner_logic()==None)]
#[ensures(^output!=None && (^output).unwrap_logic().valid_for(scope.inner_logic()))]
fn bytes_root_view_terminal_drop(mut value:Bytes,scope:Ghost<SuffixScope>,mut output:Ghost<&mut Option<RootDropEffect>>) {
    let native=value.vtable.drop;
    let descriptor=ghost! {match value.original_shared.into_inner() {
        OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()},
    }};
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address&1usize==0usize {even_root_view_drop_registration().into_inner()}
        else {odd_root_view_drop_registration().into_inner()}
    };
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **output)},spec);
}

#[logic(prophetic)]
fn root_peer_drop_input(value:Bytes,scope:SuffixScope)->bool {pearlite! {
    scope.valid() && scope.scope.is_shared() && value.api_view_valid() && value.view_owned() &&
    value.view_id()!=scope.scope.root_id() && value.view_public()==scope.scope.cursor_public() &&
    match (value.original_shared.inner_logic(),scope.scope.phase) {
        (OriginalSharedProof::Child(p),Some(Phase::Shared(s)))|
        (OriginalSharedProof::View(p,_),Some(Phase::Shared(s)))=>p.core.accepts(s.cursor),
        _=>false,
    }
}}

#[requires(root_peer_drop_input(value,*scope.inner_logic()))]
#[requires(*output.inner_logic()==None)]
#[ensures((^scope).valid() && (^scope).scope.is_shared())]
#[ensures((^scope).view==scope.inner_logic().view && (^scope).scope.descriptor==scope.inner_logic().scope.descriptor)]
#[ensures((^scope).scope.same_root(scope.inner_logic().scope) && (^scope).scope.same_pointer_owner(scope.inner_logic().scope))]
#[ensures((^scope).scope.cursor_model()==scope.inner_logic().scope.cursor_model() && (^scope).scope.cursor_public()==scope.inner_logic().scope.cursor_public())]
#[ensures((*((^scope).scope.observation())).0==(*scope.inner_logic().scope.observation()).0.remove(value.view_id()))]
#[ensures((*((^scope).scope.observation())).1==(*scope.inner_logic().scope.observation()).1)]
#[ensures(^output!=None && !(^output).unwrap_logic().reclaimed() && !(^output).unwrap_logic().was_static())]
#[ensures((^output).unwrap_logic().valid(scope.inner_logic().scope.root_metadata()))]
fn bytes_root_peer_terminal_drop(mut value:Bytes,mut scope:Ghost<&mut SuffixScope>,mut output:Ghost<&mut Option<ViewEffect>>) {
    let native=value.vtable.drop;
    let spec=ghost! {cursor_shared_drop_registration().into_inner()};
    let cursor=ghost! {match scope.scope.phase.as_mut().unwrap() {
        Phase::Shared(shared)=>&mut shared.cursor,
        _=>{proof_assert!(false);panic!()},
    }};
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),
        ghost! {(value.original_shared.into_inner(),cursor.into_inner(),&mut **output)},spec);
}

/// Both native runtime branches join the same Root continuation. The peer is
/// consumed at its lexical block end, before the second public advance.
#[requires(input@.len()>0 && first@<=input@.len() && second@<=input@.len()-first@)]
#[ensures(result@==input@.subsequence(first@+second@,input@.len()))]
pub(crate) fn root_phase_scope(input:Box<[u8]>,first:usize,second:usize,promote:bool)->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut value,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_root_view(&mut value,first,suffix.borrow_mut());
    if promote {
        let peer=clone_suffix_root(&value,suffix.borrow_mut());
        let root_id=snapshot!(suffix.scope.root_id());
        let peer_id=snapshot!(peer.view_id());
        proof_assert!(*root_id!=*peer_id && (*suffix.scope.observation()).0.len()==2);
        let mut peer_receipt=ghost! {None::<ViewEffect>};
        bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut());
        proof_assert!(!peer_receipt.inner_logic().unwrap_logic().reclaimed());
        proof_assert!((*suffix.scope.observation()).0.len()==1);
        proof_assert!((*suffix.scope.observation()).0.contains(*root_id));
    }
    proof_assert!(suffix.root_view_valid(value));
    advance_root_view(&mut value,second,suffix.borrow_mut());
    let result=chunk_root_view(&value,suffix.borrow()).to_vec();
    let saved_return=result;
    let mut receipt=ghost! {None::<RootDropEffect>};
    bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut());
    proof_assert!(receipt.inner_logic()!=None && receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}

#[requires(input.inner_logic().0.root_view_matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize==0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None && (^input.inner_logic().1).unwrap_logic().valid_for(input.inner_logic().0))]
fn even_root_view_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewDropInput>) {
    let (suffix,mut output)=input.split();
    let before=snapshot!(*suffix);
    let (descriptor,view,own,remainder)=ghost! {
        let mut suffix=suffix.into_inner();
        let (own,remainder)=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>(raw.own,RootDropRemainder::Raw(raw.recovery,raw.physical)),
            Phase::Shared(shared)=>(shared.own,RootDropRemainder::Shared(shared.root,shared.cursor)),
        };
        (suffix.scope.descriptor,suffix.view,own,remainder)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        let (root,mut cursor)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Shared(root,cursor)=>(root,cursor),
            RootDropRemainder::Raw(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==root.shared as *mut ());
        let mut completion=ghost! {None::<Completion>};
        release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut());
        ghost! {**output=Some(RootDropEffect::Shared(DetachedScope {cursor:cursor.into_inner()},completion.into_inner().unwrap()));};
    } else {
        debug_assert_eq!(kind,1usize);
        let (recovery,physical)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Raw(recovery,physical)=>(recovery,physical),
            RootDropRemainder::Shared(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==descriptor.word());
        let base=tag_specs::clear_low_bit(word,ghost! {&descriptor.base});
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(RootDropEffect::Raw(receipt.into_inner()));};
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==even_root_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==even_root_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn even_root_view_drop_registration<'a>()->Ghost<RootViewDropSpec<'a>> {Ghost::conjure()}

#[requires(input.inner_logic().0.root_view_matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(input.inner_logic().0.scope.descriptor.base.raw_pointer().addr_logic()&1usize!=0usize)]
#[requires(*input.inner_logic().1==None)]
#[ensures(^input.inner_logic().1!=None && (^input.inner_logic().1).unwrap_logic().valid_for(input.inner_logic().0))]
fn odd_root_view_drop_checked(data:&mut AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewDropInput>) {
    let (suffix,mut output)=input.split();
    let before=snapshot!(*suffix);
    let (descriptor,view,own,remainder)=ghost! {
        let mut suffix=suffix.into_inner();
        let (own,remainder)=match suffix.scope.phase.take().unwrap() {
            Phase::Raw(raw)=>(raw.own,RootDropRemainder::Raw(raw.recovery,raw.physical)),
            Phase::Shared(shared)=>(shared.own,RootDropRemainder::Shared(shared.root,shared.cursor)),
        };
        (suffix.scope.descriptor,suffix.view,own,remainder)
    }.split();
    let (word,_terminal_timestamp)=owned_pointer::get_mut_finish(data,own);
    let kind=crate::provenance_specs::pointer_addr(word)&1usize;
    if kind==0usize {
        let (root,mut cursor)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Shared(root,cursor)=>(root,cursor),
            RootDropRemainder::Raw(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==root.shared as *mut ());
        let mut completion=ghost! {None::<Completion>};
        release_core(word.cast(),root,cursor.borrow_mut(),completion.borrow_mut());
        ghost! {**output=Some(RootDropEffect::Shared(DetachedScope {cursor:cursor.into_inner()},completion.into_inner().unwrap()));};
    } else {
        debug_assert_eq!(kind,1usize);
        let (recovery,physical)=ghost! {match remainder.into_inner() {
            RootDropRemainder::Raw(recovery,physical)=>(recovery,physical),
            RootDropRemainder::Shared(_,_)=>{proof_assert!(false);panic!()},
        }}.split();
        proof_assert!(word==descriptor.word());
        let base=word.cast::<u8>();
        let receipt=unsafe {free_raw_suffix_checked(base,offset,len,ghost! {
            (descriptor.into_inner(),view.into_inner(),recovery.into_inner(),physical.into_inner())
        })};
        ghost! {**output=Some(RootDropEffect::Raw(receipt.into_inner()));};
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().drop,result.inner_logic()))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().precondition((data,ptr,len,input))==odd_root_view_drop_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&mut AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewDropInput>>
    result.inner_logic().postcondition((data,ptr,len,input),())==odd_root_view_drop_checked.postcondition((data,ptr,len,input),()))]
fn odd_root_view_drop_registration<'a>()->Ghost<RootViewDropSpec<'a>> {Ghost::conjure()}

