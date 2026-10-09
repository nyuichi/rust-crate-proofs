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
enum OriginalSharedProof { Root(RootDescriptor), Child(ChildProof) }

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
