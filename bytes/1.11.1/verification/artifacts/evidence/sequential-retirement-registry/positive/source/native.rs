use bytes_sequential_retirement_registry::{retire_left_then_right, retire_right_then_left};

#[test]
fn both_orders_at_all_split_positions() {
    for split in 0..=4 {
        retire_left_then_right(vec![1, 2, 3, 4], split);
        retire_right_then_left(vec![1, 2, 3, 4], split);
    }
}

#[test]
fn empty_allocations_and_spare_capacity() {
    retire_left_then_right(Vec::new(), 0);
    retire_right_then_left(Vec::new(), 0);
    retire_left_then_right(Vec::with_capacity(8), 0);
    retire_right_then_left(Vec::with_capacity(8), 0);
    let mut input = Vec::with_capacity(16);
    input.extend_from_slice(&[1, 2, 3, 4]);
    retire_left_then_right(input, 2);
    let mut input = Vec::with_capacity(16);
    input.extend_from_slice(&[1, 2, 3, 4]);
    retire_right_then_left(input, 2);
}
