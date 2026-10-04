use bytes_sequential_bytesmut_split::{left_then_right, right_then_left};
#[test]
fn actual_shared_control_both_orders() {
    for split in 0..=4 {
        left_then_right(vec![1, 2, 3, 4], split);
        right_then_left(vec![1, 2, 3, 4], split);
    }
    left_then_right(Vec::new(), 0);
    right_then_left(Vec::new(), 0);
    left_then_right(Vec::with_capacity(16), 0);
    right_then_left(Vec::with_capacity(16), 0);
    let mut input = Vec::with_capacity(32);
    input.extend_from_slice(&[1, 2, 3, 4]);
    left_then_right(input, 2);
}

#[test]
fn simultaneous_mutations_and_empty_views() {
    use bytes_sequential_bytesmut_split::{access_both, mutate_both};
    for right_first in [false, true] {
        for split in 1..4 {
            let mut input = Vec::with_capacity(32);
            input.extend_from_slice(&[1, 2, 3, 4]);
            mutate_both(input, split, 71, 93, right_first);
        }
        for split in 0..=4 { access_both(vec![1, 2, 3, 4], split, right_first); }
        access_both(Vec::new(), 0, right_first);
        access_both(Vec::with_capacity(32), 0, right_first);
    }
}
