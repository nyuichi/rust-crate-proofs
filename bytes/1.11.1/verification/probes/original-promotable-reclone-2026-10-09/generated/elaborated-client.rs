
/// Native lexical second/first Drop, surviving root read, saved return and
/// final root Drop. Every ledger key comes from the actual returned ticket.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn promoted_reclone_scope(input:Box<[u8]>)->Vec<u8> {
    let (original,mut scope)=from_box_scoped(input);
    let first=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let first_id=snapshot!(first.child_id());
    let before_reclone=snapshot!((*scope.observation()).0);
    let second=reclone_root(&original,scope.borrow_mut());
    let second_id=snapshot!(second.child_id());
    let second_fraction=snapshot!(second.child_fraction());
    let live_three=snapshot!((*scope.observation()).0);
    proof_assert!(!(*before_reclone).contains(*second_id));
    proof_assert!(*live_three==(*before_reclone).insert(*second_id,Excl(*second_fraction)));
    proof_assert!((*live_three).len()==3);
    let mut second_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(second,scope.borrow_mut(),second_receipt.borrow_mut());
    proof_assert!(second_receipt.inner_logic()!=None && !second_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_three).remove(*second_id));
    proof_assert!(!(*scope.observation()).0.contains(*second_id));
    proof_assert!((*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==2);
    let live_two=snapshot!((*scope.observation()).0);
    let mut first_receipt=ghost! {None::<Completion>};
    bytes_child_terminal_drop(first,scope.borrow_mut(),first_receipt.borrow_mut());
    proof_assert!(first_receipt.inner_logic()!=None && !first_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!((*scope.observation()).0==(*live_two).remove(*first_id));
    proof_assert!(!(*scope.observation()).0.contains(*first_id));
    proof_assert!((*scope.observation()).0.contains(*root_id));
    proof_assert!((*scope.observation()).0.len()==1);
    let borrowed=read_root(&original,scope.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_terminal_drop(original,scope.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(scope.phase==None && root_receipt.inner_logic()!=None && root_receipt.inner_logic().unwrap_logic().reclaimed());
    saved_return
}
