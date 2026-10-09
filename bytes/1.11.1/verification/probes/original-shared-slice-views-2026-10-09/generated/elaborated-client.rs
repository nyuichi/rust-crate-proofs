
/// All valid concrete ranges, including nested and one-past empty endpoints.
#[requires(input@.len()>0)]
#[requires(a<=b && b@<=input@.len() && c<=d && d@<=b@-a@)]
#[ensures(result@==input@.subsequence(a@+c@,a@+d@))]
pub(crate) fn nested_slice_scope(input:Box<[u8]>,a:usize,b:usize,c:usize,d:usize)->Vec<u8> {
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
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!((*detached.observation()).0.contains(*owner_id));
    let first=slice_view(&owner,a..b,detached.borrow_mut());
    let first_id=snapshot!(first.view_id());
    proof_assert!(first.view_owned() ==> *first_id!=*owner_id);
    proof_assert!(first.view_content()==(*expected).subsequence(a@,b@));
    let selected=slice_view(&first,c..d,detached.borrow_mut());
    let selected_id=snapshot!(selected.view_id());
    proof_assert!(selected.view_owned() ==> *selected_id!=*owner_id && *selected_id!=*first_id);
    proof_assert!(selected.view_content()==(*expected).subsequence(a@+c@,a@+d@));
    proof_assert!((*detached.observation()).0.len()==1+(if a<b {1int}else{0int})+(if c<d {1int}else{0int}));
    let mut first_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(first,detached.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(first_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==1+(if c<d {1int}else{0int}));
    let mut owner_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(owner,detached.borrow_mut(),owner_receipt.borrow_mut());
    proof_assert!(owner_receipt.inner_logic()!=None && owner_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()==(c==d));
    proof_assert!((*detached.observation()).0.len()==(if c<d {1int}else{0int}));
    let borrowed=read_view(&selected,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut selected_receipt=ghost! {None::<ViewEffect>};
    bytes_view_terminal_drop(selected,detached.borrow_mut(),selected_receipt.borrow_mut());
    proof_assert!(selected_receipt.inner_logic()!=None && selected_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!(selected_receipt.inner_logic().unwrap_logic().reclaimed()==(c<d));
    proof_assert!(owner_receipt.inner_logic().unwrap_logic().reclaimed()!=selected_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
