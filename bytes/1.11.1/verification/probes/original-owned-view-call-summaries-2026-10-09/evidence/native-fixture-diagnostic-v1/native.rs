use bytes::{Buf, Bytes};

pub fn clone_suffix(source: &Bytes, amount: usize) -> Bytes {
    let mut result = source.clone();
    result.advance(amount);
    result
}

pub fn owned_view_call_scope(input: Box<[u8]>, a: usize, b: usize, steps: &[usize]) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    let mut i = 0;
    while i < steps.len() {
        let amount = core::cmp::min(steps[i], value.remaining());
        let next = clone_suffix(&value, amount);
        value = next;
        i += 1;
    }
    value.chunk().to_vec()
}
