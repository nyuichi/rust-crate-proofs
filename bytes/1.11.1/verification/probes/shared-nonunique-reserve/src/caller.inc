pub(crate) fn reserve_caller(input: Vec<u8>, cut: usize, additional: usize, value: u8) -> usize {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let left = right.split_to(cmp::min(cut, right.len()));
    let mut context = right.proof_take_coordinator();
    if additional > (isize::MAX as usize) - right.len {
        assert!(!left.proof_carrier_release(context.borrow_mut()));
        assert!(right.proof_carrier_release(context.borrow_mut()));
        return 0;
    }
    let mut unique = shared_copy::reserve_copy(right, additional, context.borrow_mut());
    if unique.len() > 0 {
        unique.as_slice_mut()[0] = value;
        assert!(unique.as_slice()[0] == value);
    }
    assert!(left.proof_carrier_release(context.borrow_mut()));
    let capacity = unique.cap;
    let pointer = unique.ptr;
    let caps = ghost! { unique.unique_at_zero.take().unwrap() };
    mem::forget(unique);
    unsafe { release_unique_storage(pointer, capacity, 0, caps); }
    capacity
}

#[cfg(feature = "negative_copy_singleton")]
pub(crate) fn negative_singleton_copy(input: Vec<u8>) {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let left = right.split_to(0);
    let mut context = right.proof_take_coordinator();
    assert!(!left.proof_carrier_release(context.borrow_mut()));
    // This path has one remaining registration: the non-singleton copy
    // adapter must reject it even when every visible byte is initialized.
    let unique = shared_copy::reserve_copy(right, 0, context.borrow_mut());
    mem::forget(unique);
}
