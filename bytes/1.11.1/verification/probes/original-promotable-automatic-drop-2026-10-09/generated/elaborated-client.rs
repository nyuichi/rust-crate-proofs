
/// Proof-only elaboration: inner-scope child Drop, root read, saved return,
/// then root Drop. The independent checker certifies the native normal edges.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_automatic_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let child=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(child.child_id());
    let live_before=snapshot!((*scope.observation()).0);
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(child,scope.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && !child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_before).remove(*child_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!(!(*scope.observation()).0.contains(*child_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}
