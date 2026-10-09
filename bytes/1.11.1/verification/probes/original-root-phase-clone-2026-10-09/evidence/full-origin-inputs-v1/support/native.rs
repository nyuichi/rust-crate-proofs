use bytes::{Buf, Bytes};

pub fn root_clone_steps(input: Box<[u8]>, steps: &[usize]) -> Vec<u8> {
    let mut value = Bytes::from(input);
    let mut index = 0usize;
    while index < steps.len() {
        let amount = core::cmp::min(steps[index], value.len());
        value.advance(amount);
        {
            let _peer = value.clone();
        }
        index += 1;
    }
    value.chunk().to_vec()
}
