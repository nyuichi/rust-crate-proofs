
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
