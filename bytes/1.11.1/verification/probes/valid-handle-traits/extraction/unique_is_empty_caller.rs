#[cfg(any())]
fn proof_observe_is_empty(owner: &actual::BytesMut) {
    assert!(owner.is_empty() == (owner.len() == 0));
}