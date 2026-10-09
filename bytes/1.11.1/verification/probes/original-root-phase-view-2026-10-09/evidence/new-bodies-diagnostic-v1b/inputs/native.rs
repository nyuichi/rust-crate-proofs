use bytes::{Buf, Bytes};

pub fn root_phase_scope(input: Box<[u8]>, first: usize, second: usize, promote: bool) -> Vec<u8> {
    let mut value = Bytes::from(input);
    value.advance(first);
    if promote {
        let _peer = value.clone();
    }
    value.advance(second);
    value.chunk().to_vec()
}
