
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
