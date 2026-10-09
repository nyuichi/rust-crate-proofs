
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
