//! Exercise unique access without promotion to a Shared control block.
use bytes_sequential_bytesmut_split::unique_access;

fn fixture(kind: usize) -> Vec<u8> {
    match kind {
        0 => Vec::new(),
        1 => Vec::with_capacity(8),
        2 => Vec::from(Box::<[u8]>::from([1, 2, 3, 4])),
        _ => {
            let mut input = Vec::with_capacity(8);
            input.extend_from_slice(&[1, 2, 3, 4]);
            input
        }
    }
}

#[test]
fn unique_known_spare_and_empty_access() {
    for kind in 0..4 {
        let len = fixture(kind).len();
        for keep in [0, len / 2, len] {
            for value in [0, 255] {
                unique_access(fixture(kind), keep, value);
            }
        }
    }
}
