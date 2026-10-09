use bytes::{Buf, Bytes};

pub fn owned_view_clone_scope(
    input: Box<[u8]>,
    a: usize,
    b: usize,
    advance_by: usize,
    rounds: usize,
) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    value.advance(advance_by);
    let mut i = 0;
    while i < rounds {
        let next = value.clone();
        value = next;
        i += 1;
    }
    value.chunk().to_vec()
}
