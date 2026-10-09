
/// Runtime count creates an arbitrary finite inventory. Every popped peer is
/// retired by its actual lexical Drop; empty Vec Drop precedes final survivor.
#[requires(input@.len()>0)]
#[ensures(result@==input@)]
pub(crate) fn finite_shared_scope(input:Box<[u8]>,count:usize)->Vec<u8> {
    let expected=snapshot!(input@);
    let (original,mut scope)=from_box_scoped(input);
    let survivor=clone_root(&original,scope.borrow_mut());
    let root_id=snapshot!(scope.root_id());
    let child_id=snapshot!(survivor.child_id());
    let before=snapshot!((*scope.observation()).0);
    let metadata=snapshot!(survivor.child_public().3);
    let mut detached_output=ghost! {None::<DetachedScope>};
    let mut root_receipt=ghost! {None::<Completion>};
    bytes_root_detaching_terminal_drop(original,scope,detached_output.borrow_mut(),root_receipt.borrow_mut());
    proof_assert!(root_receipt.inner_logic()!=None && !root_receipt.inner_logic().unwrap_logic().reclaimed());
    let mut detached=ghost! {detached_output.into_inner().unwrap()};
    proof_assert!((*detached.observation()).0==(*before).remove(*root_id));
    proof_assert!((*detached.observation()).0.len()==1);
    let mut owners:Vec<Bytes>=Vec::new();
    let mut made=0usize;
    #[invariant(made<=count)]
    #[invariant(owners@.len()==made@)]
    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(count@-made@)]
    while made<count {
        let old_owners=snapshot!(owners@);
        let old_scope=snapshot!(detached.inner_logic());
        let next=clone_surviving_child(&survivor,detached.borrow_mut());
        ghost! {prove_inventory_push(old_owners,snapshot!(survivor),snapshot!(next),
            old_scope,snapshot!(detached.inner_logic()));};
        owners.push(next);
        made+=1;
    }
    #[invariant(finite_inventory(owners@,survivor,detached.inner_logic()))]
    #[invariant(survivor.child_content()==*expected)]
    #[invariant(survivor.child_public().3==*metadata)]
    #[variant(owners@.len())]
    while let Some(peer)=owners.pop() {
        proof_assert!(finite_inventory(owners@.push_back(peer),survivor,detached.inner_logic()));
        let mut peer_receipt=ghost! {None::<Completion>};
        bytes_detached_child_terminal_drop(peer,detached.borrow_mut(),peer_receipt.borrow_mut());
        proof_assert!(peer_receipt.inner_logic()!=None && !peer_receipt.inner_logic().unwrap_logic().reclaimed());
        proof_assert!(peer_receipt.inner_logic().unwrap_logic().valid(*metadata));
    }
    proof_assert!(owners@.len()==0 && (*detached.observation()).0.len()==1);
    let borrowed=read_surviving_child(&survivor,detached.borrow());
    let observed=borrowed.to_vec();
    let saved_return=observed;
    empty_vec_terminal_drop(owners);
    let mut child_receipt=ghost! {None::<Completion>};
    bytes_detached_child_terminal_drop(survivor,detached.borrow_mut(),child_receipt.borrow_mut());
    proof_assert!(child_receipt.inner_logic()!=None && child_receipt.inner_logic().unwrap_logic().reclaimed());
    proof_assert!(child_receipt.inner_logic().unwrap_logic().valid(*metadata));
    proof_assert!((*detached.observation()).0.len()==0);
    saved_return
}
