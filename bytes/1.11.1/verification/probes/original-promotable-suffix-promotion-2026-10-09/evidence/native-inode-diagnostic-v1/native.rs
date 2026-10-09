use bytes::{Buf, Bytes};

pub fn promotable_suffix_scope(input: Box<[u8]>, amount: usize) -> Vec<u8> {
    let child = {
        let mut root = Bytes::from(input);
        root.advance(amount);
        root.clone()
    };
    child.chunk().to_vec()
}
