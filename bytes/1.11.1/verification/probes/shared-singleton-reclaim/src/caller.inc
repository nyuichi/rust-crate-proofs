pub(crate) fn singleton_reclaim_caller(input: Vec<u8>, cut: usize, value: u8) {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let left = right.split_to(cmp::min(cut, right.len()));
    let original = snapshot!(right);
    let mut context = right.proof_take_coordinator();
    assert!(!left.proof_carrier_release(context.borrow_mut()));
    let registration = ghost! { right.shared_registration.take().unwrap() };
    let identity = ghost! { registration.control.identity.into_inner() };
    let control = sequential_shared_control::ControlPtr { pointer: right.data, identity };
    assert!(shared_reclaim::is_unique(control, context.borrow()));
    let lease = shared_reclaim::acquire(control, context, registration);
    let restored = shared_reclaim::reactivate(control, lease);
    let (mut context, registration) = restored.split();
    right.shared_registration = ghost! { Some(registration.into_inner()) };
    proof_assert!(right.proof_initialized());
    proof_assert!(forall<index: Int> 0 <= index && index < right.len@ ==>
        right.proof_view_slot(index) == original.proof_view_slot(index));
    if right.len() > 0 {
        right.as_slice_mut()[0] = value;
        assert!(right.as_slice()[0] == value);
    }
    assert!(right.proof_carrier_release(context.borrow_mut()));
}

#[cfg(feature = "negative_not_singleton")]
pub(crate) fn singleton_negative() {
    let mut right = BytesMut::from_vec(Vec::new());
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let _left = right.split_to(0);
    let context = right.proof_take_coordinator();
    let registration = ghost! { right.shared_registration.take().unwrap() };
    let identity = ghost! { registration.control.identity.into_inner() };
    let control = sequential_shared_control::ControlPtr { pointer: right.data, identity };
    assert!(!shared_reclaim::is_unique(control, context.borrow()));
    proof_assert!(registration.matches(context.inner_logic()));
    proof_assert!((*context.status.pending).len() == 2);
    // Both regions are empty; the outstanding sibling registration still
    // prevents taking the allocation and resetting the native count-two state.
    let _lease = shared_reclaim::acquire(control, context, registration);
}
