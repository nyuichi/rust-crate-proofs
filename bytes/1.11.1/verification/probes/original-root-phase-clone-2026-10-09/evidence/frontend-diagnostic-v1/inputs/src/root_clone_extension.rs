// AZ: one Root Clone across raw suffix promotion and Shared bounded windows.
// Ledger membership remains explicit caller knowledge; accepts is not a getter.
#[logic(prophetic)]
fn root_clone_result(before:SuffixScope,after:SuffixScope,ptr:*const u8,len:usize,result:Bytes)->bool {pearlite! {
    after.valid() && after.scope.is_shared() && after.scope.descriptor==before.scope.descriptor && after.view==before.view &&
    result.api_view_valid() && result.view_owned() && result.ptr==ptr && result.len==len && result.view_bound()==before.view &&
    result.view_content()==before.root_view_content(len) && result.view_public()==after.scope.cursor_public() &&
    result.view_public().3.1==before.scope.descriptor.base@.unwrap_logic().0 &&
    result.view_public().3.2==before.scope.descriptor.capacity@ &&
    result.view_public().3.3==*before.scope.descriptor.expected &&
    result.view_public().3.6==before.scope.descriptor.base.raw_pointer() &&
    match result.original_shared.inner_logic() {OriginalSharedProof::View(p,_)=>
        match after.scope.phase {Some(Phase::Shared(s))=>p.core.accepts(s.cursor) && result.has_allocation(s.root),_=>false},_=>false} &&
    if before.scope.is_raw() {
        after.scope.root_id()!=result.view_id() &&
        (*after.scope.observation()).0==FMap::singleton(after.scope.root_id(),Excl(after.scope.root_fraction())).insert(
            result.view_id(),Excl(result.view_fraction())) && (*after.scope.observation()).1==2
    } else {
        before.scope.is_shared() && after.scope.same_root(before.scope) && after.scope.pointer_history_progresses(before.scope) &&
        after.scope.cursor_model()==before.scope.cursor_model() && after.scope.cursor_public()==before.scope.cursor_public() &&
        !(*before.scope.observation()).0.contains(result.view_id()) &&
        (*after.scope.observation()).0==(*before.scope.observation()).0.insert(result.view_id(),Excl(result.view_fraction())) &&
        (*after.scope.observation()).1==(*before.scope.observation()).1+1
    }
}}

type RootViewCloneInput<'a>=(&'a RootDescriptor,raw_vec::BoundPtr,&'a mut SuffixScope);
type RootViewCloneSpec<'a>=fn(&'a AtomicPtr<()>,*const u8,usize,Ghost<RootViewCloneInput<'a>>)->Bytes;

#[requires(scope.inner_logic().root_view_valid(*source))]
#[ensures((^scope).root_view_valid(*source))]
#[ensures(root_clone_result(*scope.inner_logic(),^scope,source.ptr,source.len,result))]
fn clone_root_view(source:&Bytes,mut scope:Ghost<&mut SuffixScope>)->Bytes {
    let descriptor=ghost! {match &*source.original_shared {
        OriginalSharedProof::Root(d)=>d,_=>{proof_assert!(false);panic!()},
    }};
    let spec=ghost! {
        let base=snapshot!(descriptor.base.raw_pointer()).into_ghost().into_inner();
        let address=crate::provenance_specs::pointer_addr(base);
        if address&1usize==0usize {even_root_view_clone_registration().into_inner()}
        else {odd_root_view_clone_registration().into_inner()}
    };
    let native=source.vtable.clone;
    erased_call::invoke3(native,(&source.data,source.ptr,source.len),
        ghost! {(*descriptor,scope.view,&mut **scope)},spec)
}

#[requires(input@.len()>0)]
#[ensures(result@==input@.subsequence(consumed(steps@,steps@.len(),input@.len()),input@.len()))]
pub(crate) fn root_clone_steps(input:Box<[u8]>,steps:&[usize])->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut value,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    let descriptor=snapshot!(suffix.scope.descriptor);
    let mut index=0usize;
    #[invariant(index@<=steps@.len())]
    #[invariant(suffix.root_view_valid(value))]
    #[invariant(suffix.scope.is_raw()==(index==0usize))]
    #[invariant(suffix.scope.descriptor==*descriptor && *suffix.scope.descriptor.expected==*expected)]
    #[invariant(suffix.view@.unwrap_logic().2==consumed(steps@,index@,expected.len()))]
    #[invariant(value.len@==expected.len()-consumed(steps@,index@,expected.len()))]
    #[invariant(suffix.scope.is_raw() || (suffix.scope.is_shared() &&
        (*suffix.scope.observation()).0==FMap::singleton(suffix.scope.root_id(),Excl(suffix.scope.root_fraction()))))]
    #[variant(steps@.len()-index@)]
    while index<steps.len() {
        let amount=core::cmp::min(steps[index],value.len());
        advance_root_view(&mut value,amount,suffix.borrow_mut());
        {
            let peer=clone_root_view(&value,suffix.borrow_mut());
            let root_id=snapshot!(suffix.scope.root_id());
            let root_fraction=snapshot!(suffix.scope.root_fraction());
            let peer_id=snapshot!(peer.view_id());
            proof_assert!(*root_id!=*peer_id);
            proof_assert!((*suffix.scope.observation()).0==FMap::singleton(*root_id,Excl(*root_fraction)).insert(
                *peer_id,Excl(peer.view_fraction())));
            let mut peer_receipt=ghost! {None::<ViewEffect>};
            bytes_root_peer_terminal_drop(peer,suffix.borrow_mut(),peer_receipt.borrow_mut());
            proof_assert!(!peer_receipt.inner_logic().unwrap_logic().reclaimed());
            proof_assert!({
                let remaining=FMap::singleton(*root_id,Excl(*root_fraction));
                (*suffix.scope.observation()).0.ext_eq(remaining)
            });
            proof_assert!((*suffix.scope.observation()).0==FMap::singleton(*root_id,Excl(*root_fraction)));
        }
        index+=1;
    }
    let result=chunk_root_view(&value,suffix.borrow()).to_vec();
    let saved_return=result;
    let mut receipt=ghost! {None::<RootDropEffect>};
    bytes_root_view_terminal_drop(value,suffix,receipt.borrow_mut());
    proof_assert!(receipt.inner_logic()!=None && receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(match receipt.inner_logic().unwrap_logic() {
        RootDropEffect::Raw(_)=>steps@.len()==0,
        RootDropEffect::Shared(_,_)=>steps@.len()>0,
    });
    saved_return
}

#[requires(input.inner_logic().2.root_view_matches(offset,len,pointer_event::pointer_model(data),even_table()))]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic()&1usize==0usize)]
#[ensures(root_clone_result(*input.inner_logic().2,^input.inner_logic().2,offset,len,result))]
fn even_root_view_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewCloneInput>)->Bytes {
    let (descriptor,view,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                Phase::Shared(shared)=>{c.shoot_load(&shared.own,&mut shared.current);},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored)&1usize;
    if kind==0usize {
        let mut shared=ghost! {match scope.scope.phase.as_mut().unwrap() {
            Phase::Shared(shared)=>shared,
            Phase::Raw(_)=>{proof_assert!(false);panic!()},
        }};
        shallow_clone_owned_view_checked(stored.cast(),offset,len,ghost! {
            (&shared.root,view.into_inner(),&mut shared.cursor)
        })
    } else {
        debug_assert_eq!(kind,1usize);
        proof_assert!(scope.scope.is_raw());
        let base=tag_specs::clear_low_bit(stored,ghost! {&descriptor.base});
        shallow_clone_root_suffix_checked(data,stored,base,offset,len,ghost! {
            (*descriptor,view.into_inner(),&mut **scope)
        })
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(even_table().clone,result.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==even_root_view_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==even_root_view_clone_checked.postcondition((data,ptr,len,input),output))]
fn even_root_view_clone_registration<'a>()->Ghost<RootViewCloneSpec<'a>> {Ghost::conjure()}

#[requires(input.inner_logic().2.root_view_matches(offset,len,pointer_event::pointer_model(data),odd_table()))]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().0.base.raw_pointer().addr_logic()&1usize!=0usize)]
#[ensures(root_clone_result(*input.inner_logic().2,^input.inner_logic().2,offset,len,result))]
fn odd_root_view_clone_checked(data:&AtomicPtr<()>,offset:*const u8,len:usize,input:Ghost<RootViewCloneInput>)->Bytes {
    let (descriptor,view,mut scope)=input.split();
    let stored=owned_pointer::load_acquire(data,ghost! {
        |c:&Committer<ModelAtomicPtr<()>,*mut (),Acquire,NoStore>| {
            match scope.scope.phase.as_mut().unwrap() {
                Phase::Raw(raw)=>{c.shoot_load(&raw.own,&mut raw.current);},
                Phase::Shared(shared)=>{c.shoot_load(&shared.own,&mut shared.current);},
            }
        }
    });
    let kind=crate::provenance_specs::pointer_addr(stored)&1usize;
    if kind==0usize {
        let mut shared=ghost! {match scope.scope.phase.as_mut().unwrap() {
            Phase::Shared(shared)=>shared,
            Phase::Raw(_)=>{proof_assert!(false);panic!()},
        }};
        shallow_clone_owned_view_checked(stored.cast(),offset,len,ghost! {
            (&shared.root,view.into_inner(),&mut shared.cursor)
        })
    } else {
        debug_assert_eq!(kind,1usize);
        proof_assert!(scope.scope.is_raw());
        let base=stored.cast::<u8>();
        shallow_clone_root_suffix_checked(data,stored,base,offset,len,ghost! {
            (*descriptor,view.into_inner(),&mut **scope)
        })
    }
}

#[trusted]
#[check(ghost)]
#[ensures(erased_call::registered3(odd_table().clone,result.inner_logic()))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewCloneInput>>
    result.inner_logic().precondition((data,ptr,len,input))==odd_root_view_clone_checked.precondition((data,ptr,len,input)))]
#[ensures(forall<data:&AtomicPtr<()>,ptr:*const u8,len:usize,input:Ghost<RootViewCloneInput>,output:Bytes>
    result.inner_logic().postcondition((data,ptr,len,input),output)==odd_root_view_clone_checked.postcondition((data,ptr,len,input),output))]
fn odd_root_view_clone_registration<'a>()->Ghost<RootViewCloneSpec<'a>> {Ghost::conjure()}

// Full AV operation retained; AZ exports allocation identity and initial counter.
#[requires(input.inner_logic().2.valid() && input.inner_logic().2.scope.is_raw())]
#[requires(*input.inner_logic().0==input.inner_logic().2.scope.descriptor && input.inner_logic().1==input.inner_logic().2.view)]
#[requires(input.inner_logic().2.matches(offset,len,pointer_event::pointer_model(data),input.inner_logic().0.table()))]
#[requires(expected==input.inner_logic().0.word() && buf==input.inner_logic().0.base.raw_pointer())]
#[ensures(suffix_clone_result(*input.inner_logic().0,input.inner_logic().1,^input.inner_logic().2,result))]
#[ensures(root_clone_result(*input.inner_logic().2,^input.inner_logic().2,offset,len,result))]
fn shallow_clone_root_suffix_checked(data:&AtomicPtr<()>,expected:*mut (),buf:*mut u8,offset:*const u8,len:usize,input:Ghost<SuffixCloneInput>)->Bytes {
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

