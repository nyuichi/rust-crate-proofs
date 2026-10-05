pub(crate) fn reserve_caller(input: Vec<u8>, cut: usize, additional: usize, value: u8) -> usize {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let offset = cmp::min(cut, right.len());
    let left = right.split_to(offset);
    let original = snapshot!(right);
    let mut context = right.proof_take_coordinator();
    assert!(!left.proof_carrier_release(context.borrow_mut()));
    if additional > (isize::MAX as usize) - offset - right.len {
        assert!(right.proof_carrier_release(context.borrow_mut()));
        return 0;
    }
    let registration = ghost! { right.shared_registration.take().unwrap() };
    let identity = ghost! { registration.control.identity.into_inner() };
    let control = sequential_shared_control::ControlPtr { pointer: right.data, identity };
    let lease = shared_reclaim::acquire(control, context, registration);
    let (pointer, capacity, lease) = shared_reserve::reserve_lease(control, lease, right.ptr, offset, right.len, additional);
    let end = snapshot!(pointer@.unwrap_logic().2 + capacity@);
    let (mut context, registration) = shared_reserve::reactivate_view(control, lease, end).split();
    right.ptr = pointer;
    right.cap = capacity;
    right.shared_registration = ghost! { Some(registration.into_inner()) };
    proof_assert!(forall<index: Int> 0 <= index && index < right.len@ ==>
        right.proof_view_slot(index) == original.proof_view_slot(index));
    proof_assert!(right.proof_owned_valid());
    proof_assert!(forall<i: Int> 0 <= i && i < right.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(right.proof_view_slot(i)));
    proof_assert!(right.proof_initialized());
    assert!(right.capacity() >= right.len() + additional);
    if right.len() > 0 {
        right.as_slice_mut()[0] = value;
        assert!(right.as_slice()[0] == value);
    }
    assert!(right.proof_carrier_release(context.borrow_mut()));
    capacity
}

#[cfg(feature = "negative_reserved_view")]
#[requires(input@.len() >= 1)]
pub(crate) fn negative_shrunken_reactivation(input: Vec<u8>) {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let left = right.split_to(0);
    let mut context = right.proof_take_coordinator();
    assert!(!left.proof_carrier_release(context.borrow_mut()));
    let registration = ghost! { right.shared_registration.take().unwrap() };
    let identity = ghost! { registration.control.identity.into_inner() };
    let control = sequential_shared_control::ControlPtr { pointer: right.data, identity };
    let lease = shared_reclaim::acquire(control, context, registration);
    let (context, registration) = shared_reserve::reactivate_view(control, lease, snapshot!(0int)).split();
    right.cap = 0;
    right.shared_registration = ghost! { Some(registration.into_inner()) };
    // A registration for [0,0) cannot authorize the old nonempty view.
    let _ = right.as_slice();
    mem::forget(right);
    let _ = context;
}
