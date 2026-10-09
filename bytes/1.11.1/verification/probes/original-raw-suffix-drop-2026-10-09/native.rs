use bytes::{Buf, Bytes};

pub fn raw_suffix_scope(input: Box<[u8]>, amount: usize) -> Vec<u8> {
    let mut value = Bytes::from(input);
    value.advance(amount);
    value.chunk().to_vec()
}
