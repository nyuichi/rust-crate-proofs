
/// Actual raw suffix advance precedes first promotion. Root drops before the
/// surviving suffix is read; zero-length suffixes still own the allocation.
#[requires(input@.len()>0 && amount@<=input@.len())]
#[ensures(result@==input@.subsequence(amount@,input@.len()))]
pub(crate) fn promotable_suffix_scope(input:Box<[u8]>,amount:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (mut root,scope)=from_box_scoped(input);
    let mut suffix=SuffixScope::new(scope);
    advance_suffix(&mut root,amount,suffix.borrow_mut());
    let child=clone_suffix_root(&root,suffix.borrow_mut());
    let root_id=snapshot!(suffix.scope.root_id());
    let child_id=snapshot!(child.view_id());
    let before=snapshot!((*suffix.scope.observation()).0);
    let metadata=snapshot!(child.view_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_suffix_root_terminal_drop(root,suffix,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1 && (*detached.observation()).0.contains(*child_id));
    proof_assert!(child.view_accepts(*detached));
    let observed=chunk_api(&child).to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<ViewEffect>};
    bytes_cursor_terminal_drop(child,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
