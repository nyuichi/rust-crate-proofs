use bytes::{Buf, Bytes};

pub fn cursor_scope(input: Box<[u8]>, a: usize, b: usize, steps: &[usize]) -> Vec<u8> {
    let mut value = {
        let owner = {
            let original = Bytes::from(input);
            original.clone()
        };
        owner.slice(a..b)
    };
    let mut i = 0;
    while i < steps.len() {
        let by = core::cmp::min(steps[i], value.remaining());
        value.advance(by);
        i += 1;
    }
    let observed = value.chunk().to_vec();
    let rest = value.remaining();
    value.advance(rest);
    assert_eq!(value.remaining(), 0);
    assert!(value.chunk().is_empty());
    observed
}
