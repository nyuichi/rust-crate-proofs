
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
