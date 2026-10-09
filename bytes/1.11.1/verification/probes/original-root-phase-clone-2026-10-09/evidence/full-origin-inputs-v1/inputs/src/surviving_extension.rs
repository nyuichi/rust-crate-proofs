
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
