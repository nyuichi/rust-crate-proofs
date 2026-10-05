#[cfg(not(creusot))]
pub fn unique_is_empty(input: Vec<u8>) -> bool {
    let owner = actual::BytesMut::from_vec(input);
    let result = owner.is_empty();
    owner.proof_release_unique_at_zero();
    result
}