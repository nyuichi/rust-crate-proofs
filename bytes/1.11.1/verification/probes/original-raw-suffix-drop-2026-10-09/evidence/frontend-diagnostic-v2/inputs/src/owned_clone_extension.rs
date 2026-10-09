
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
