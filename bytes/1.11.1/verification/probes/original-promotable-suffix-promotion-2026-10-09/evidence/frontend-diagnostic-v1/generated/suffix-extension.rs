
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
    #[logic(prophetic)] fn matches(self,ptr:*const u8,len:usize,model:ModelAtomicPtr<()>,table:&'static Vtable)->bool {
        self.valid() && ptr==self.view.raw_pointer() as *const u8 &&
        self.view@.unwrap_logic().2+len@==self.scope.descriptor.capacity@ &&
        model==*self.scope.descriptor.model && table==self.scope.descriptor.table()
    }
    #[logic(prophetic)] fn root_valid(self,value:Bytes)->bool {
        self.matches(value.ptr,value.len,pointer_event::pointer_model(&value.data),value.vtable) &&
        match value.original_shared.inner_logic() {OriginalSharedProof::Root(d)=>d==self.scope.descriptor,_=>false}
    }
    #[logic] fn content(self)->Seq<u8> {
        (*self.scope.descriptor.expected).subsequence(self.view@.unwrap_logic().2,self.scope.descriptor.capacity@)
    }
    #[check(ghost)]
    #[requires(scope.inner_logic().valid() && scope.inner_logic().is_raw())]
    #[ensures(result.inner_logic().valid() && result.inner_logic().scope==scope.inner_logic())]
    #[ensures(result.inner_logic().view==scope.inner_logic().descriptor.base)]
    fn new(scope:Ghost<PromotionScope>)->Ghost<Self> {
        ghost! {
            let _base=scope.descriptor.base.as_ptr();
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
        let base=descriptor.base.as_ptr();
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
        let base=descriptor.base.as_ptr();
        let address=crate::provenance_specs::pointer_addr(base);
        if address & 1usize==0usize {even_suffix_drop_registration().into_inner()}
        else {odd_suffix_drop_registration().into_inner()}
    };
    erased_call::invoke3(native,(&mut value.data,value.ptr,value.len),ghost! {(scope.into_inner(),&mut **detached,&mut **output)},spec);
}
