//! Native edge cases for advancing the two handles produced by `split_off`.
use bytes_sequential_bytesmut_split::advance_split_off;
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

fn push_unique_pair(
    pairs: &mut [(usize, usize); 6],
    count: &mut usize,
    pair: (usize, usize),
) {
    if !pairs[..*count].contains(&pair) {
        pairs[*count] = pair;
        *count += 1;
    }
}

fn exercise_fixture(
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

    let mut split_points = [0usize; 5];
    let mut split_count = 0usize;
    push_unique(&mut split_points, &mut split_count, 0);
    push_unique(&mut split_points, &mut split_count, len);
    if len > 1 {
        push_unique(&mut split_points, &mut split_count, len / 2);
    }
    if capacity - len > 1 {
        push_unique(
            &mut split_points,
            &mut split_count,
            len + (capacity - len) / 2,
        );
    }
    push_unique(&mut split_points, &mut split_count, capacity);

    for &at in &split_points[..split_count] {
        let left_len = len.min(at);
        let right_len = len.saturating_sub(at);
        let left_capacity = at;
        let right_capacity = capacity - at;

        let mut advances = [(0usize, 0usize); 6];
        let mut advance_count = 0usize;
        push_unique_pair(&mut advances, &mut advance_count, (0, 0));
        push_unique_pair(
            &mut advances,
            &mut advance_count,
            (left_len, right_len),
        );
        push_unique_pair(
            &mut advances,
            &mut advance_count,
            (left_capacity, right_capacity),
        );
        if left_capacity > 1 || right_capacity > 1 {
            push_unique_pair(
                &mut advances,
                &mut advance_count,
                (left_capacity / 2, right_capacity / 2),
            );
        }
        if left_capacity > left_len {
            push_unique_pair(
                &mut advances,
                &mut advance_count,
                (left_len + 1, 0),
            );
        }
        if right_capacity > right_len {
            push_unique_pair(
                &mut advances,
                &mut advance_count,
                (0, right_len + 1),
            );
        }

        for &(left_count, right_count) in &advances[..advance_count] {
            assert!(left_count <= left_capacity);
            assert!(right_count <= right_capacity);
            for right_first in [false, true] {
                // This helper checks post-advance geometry and visible bytes,
                // mutates only any remaining initialized prefix, then
                // explicitly releases both handles in the requested order.
                advance_split_off(make_input(), at, left_count, right_count, right_first);
            }
        }
    }
}

#[test]
fn split_off_advances_cover_visible_and_spare_capacity_in_both_orders() {
    exercise_fixture(empty_zero_capacity, 0, 0, true);
    exercise_fixture(empty_spare_capacity, 0, 8, false);
    exercise_fixture(initialized_full_capacity, 4, 4, true);
    exercise_fixture(initialized_spare_capacity, 4, 8, false);
}
