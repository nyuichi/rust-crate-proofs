#[cfg(creusot)]
fn proof_observe_is_empty(owner: &BytesMut) {
    assert!(owner.is_empty() == (owner.len() == 0));
}