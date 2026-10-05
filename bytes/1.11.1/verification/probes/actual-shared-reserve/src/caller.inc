pub(crate) fn reserve_caller(input: Vec<u8>, cut: usize, additional: usize, value: u8) -> usize {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let offset = cmp::min(cut, right.len());
    let left = right.split_to(offset);
    let mut context = right.proof_take_coordinator();
    assert!(!left.proof_carrier_release(context.borrow_mut()));
    if additional > (isize::MAX as usize) - offset - right.len {
        assert!(right.proof_carrier_release(context.borrow_mut()));
        return 0;
    }
    let mut coordinator = ghost! { Some(context.into_inner()) };
    right.reserve(additional, coordinator.borrow_mut());
    assert!(right.capacity() >= right.len() + additional);
    if right.len() > 0 {
        right.as_slice_mut()[0] = value;
        assert!(right.as_slice()[0] == value);
    }
    let capacity = right.cap;
    let mut context = ghost! { coordinator.take().unwrap() };
    assert!(right.proof_carrier_release(context.borrow_mut()));
    capacity
}

pub(crate) fn reserve_with_sibling(input: Vec<u8>, additional: usize) -> usize {
    let mut right = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(right)); };
    let left = right.split_to(0);
    let context = right.proof_take_coordinator();
    if additional > (isize::MAX as usize) - right.len || additional <= right.cap - right.len {
        let mut context = context;
        assert!(!left.proof_carrier_release(context.borrow_mut()));
        assert!(right.proof_carrier_release(context.borrow_mut()));
        return 0;
    }
    let mut coordinator = ghost! { Some(context.into_inner()) };
    right.reserve(additional, coordinator.borrow_mut());
    let mut context = ghost! { coordinator.take().unwrap() };
    assert!(left.proof_carrier_release(context.borrow_mut()));
    let capacity = right.cap;
    let pointer = right.ptr;
    let caps = ghost! { right.unique_at_zero.take().unwrap() };
    mem::forget(right);
    unsafe { release_unique_storage(pointer, capacity, 0, caps); }
    capacity
}
