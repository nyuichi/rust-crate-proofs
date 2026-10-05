pub(crate) fn unsplit_caller(input: Vec<u8>, first: usize, second: usize, reverse: bool, value: u8) {
    let mut tail = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(tail)); };
    let mut left = tail.split_to(cmp::min(first, tail.len()));
    let middle = tail.split_to(cmp::min(second, tail.len()));
    let mut context = tail.proof_take_coordinator();
    left.unsplit(middle, context.borrow_mut());
    if left.len() > 0 {
        let last = left.len() - 1;
        left.as_slice_mut()[last] = value;
        assert!(left.as_slice()[last] == value);
    }
    if reverse {
        assert!(!tail.proof_carrier_release(context.borrow_mut()));
        assert!(left.proof_carrier_release(context.borrow_mut()));
    } else {
        assert!(!left.proof_carrier_release(context.borrow_mut()));
        assert!(tail.proof_carrier_release(context.borrow_mut()));
    }
}

#[cfg(feature = "negative_unsplit_gap")]
#[requires(input@.len() >= 2)]
pub(crate) fn negative_gap_merge(input: Vec<u8>) {
    let mut tail = BytesMut::from_vec(input);
    ghost! { BytesMut::proof_initialization_coordinates(snapshot!(tail)); };
    let mut left = tail.split_to(1);
    let middle = tail.split_to(1);
    let mut context = tail.proof_take_coordinator();
    // The middle registration still owns the byte separating these views.
    left.unsplit(tail, context.borrow_mut());
    mem::forget(left);
    mem::forget(middle);
}
