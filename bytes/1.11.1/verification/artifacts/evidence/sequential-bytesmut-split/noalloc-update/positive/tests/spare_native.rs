//! Native checks for initializing spare slots and reinitializing a prefix.
use bytes_sequential_bytesmut_split::{initialize_split_spare, reinitialize_split_prefix};
use std::vec::Vec;

fn empty_zero_capacity() -> Vec<u8> {
    Vec::new()
}

fn empty_spare_capacity() -> Vec<u8> {
    Vec::with_capacity(8)
}

fn initialized_full_capacity() -> Vec<u8> {
    let input: Box<[u8]> = Box::new([1, 2, 3, 4]);
    Vec::from(input)
}

fn initialized_spare_capacity() -> Vec<u8> {
    let mut input = Vec::with_capacity(8);
    input.extend_from_slice(&[1, 2, 3, 4]);
    input
}

fn push_unique(points: &mut [usize; 5], count: &mut usize, at: usize) {
    if !points[..*count].contains(&at) {
        points[*count] = at;
        *count += 1;
    }
}

fn exercise_spare_fixture(
    make_input: fn() -> Vec<u8>,
    expected_len: usize,
    minimum_capacity: usize,
    require_full: bool,
) {
    let sample = make_input();
    let len = sample.len();
    let capacity = sample.capacity();
    assert_eq!(len, expected_len);
    assert!(capacity >= minimum_capacity);
    if require_full {
        assert_eq!(capacity, len);
    } else if len == 0 && minimum_capacity == 0 {
        assert_eq!(capacity, 0);
    } else {
        assert!(capacity > len);
    }
    drop(sample);

    let mut points = [0usize; 5];
    let mut count = 0usize;
    push_unique(&mut points, &mut count, 0);
    push_unique(&mut points, &mut count, len);
    if len > 1 {
        push_unique(&mut points, &mut count, len / 2);
    }
    if capacity - len > 1 {
        push_unique(
            &mut points,
            &mut count,
            len + (capacity - len) / 2,
        );
    }
    push_unique(&mut points, &mut count, capacity);

    for &at in &points[..count] {
        for right_first in [false, true] {
            // The helper initializes only available spare slots, publishes
            // each slot with set_len after writing it, then frees both handles.
            initialize_split_spare(make_input(), at, right_first);
        }
    }
}

#[test]
fn split_initializes_spare_slots_at_allocation_edges_in_both_orders() {
    exercise_spare_fixture(empty_zero_capacity, 0, 0, true);
    exercise_spare_fixture(empty_spare_capacity, 0, 8, false);
    exercise_spare_fixture(initialized_full_capacity, 4, 4, true);
    exercise_spare_fixture(initialized_spare_capacity, 4, 8, false);
}

#[test]
fn nonempty_prefix_can_be_reinitialized_after_becoming_unknown() {
    let fixtures: [fn() -> Vec<u8>; 2] = [
        initialized_full_capacity,
        initialized_spare_capacity,
    ];
    for make_input in fixtures {
        for right_first in [false, true] {
            reinitialize_split_prefix(make_input(), right_first);
        }
    }
}
