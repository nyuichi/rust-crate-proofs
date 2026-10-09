
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
