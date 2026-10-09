
/// Inner original Drop publishes recovery; the surviving child's final Drop
/// recovers it. No original owner or root permission survives the handoff.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_surviving_child_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    proof_assert!((*before).len()==2 && *root_id!=*child_id);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(root_receipt.inner_logic().unwrap_logic().valid(*metadata));
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!(!(*detached.observation()).0.contains(*root_id));
    proof_assert!((*detached.observation()).0.contains(*child_id));
    proof_assert!((*detached.observation()).0.len()==1);
    proof_assert!(detached.accepts(survivor));
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
