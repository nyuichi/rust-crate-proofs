//! Native edge cases for two consecutive in-capacity advances of a unique
//! `BytesMut` handle. The probe entry checks view geometry and recovery.

use bytes_sequential_bytesmut_split::unique_advance;
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

fn push_unique(points: &mut [usize], count: &mut usize, value: usize) {
    if !points[..*count].contains(&value) {
        points[*count] = value;
        *count += 1;
    }
}

fn advance_counts(capacity: usize, visible_len: usize) -> ([usize; 5], usize) {
    let mut points = [0usize; 5];
    let mut count = 0usize;
    push_unique(&mut points, &mut count, 0);
    if capacity > 0 {
        push_unique(&mut points, &mut count, 1);
    }
    push_unique(&mut points, &mut count, visible_len / 2);
    push_unique(&mut points, &mut count, visible_len);
    push_unique(&mut points, &mut count, capacity);
    assert!(count <= points.len());
    (points, count)
}

fn exercise_fixture(make_input: fn() -> Vec<u8>, expected_len: usize, min_capacity: usize) {
    let sample = make_input();
    let len = sample.len();
    let capacity = sample.capacity();
    assert_eq!(len, expected_len);
    assert!(capacity >= min_capacity);
    assert!(capacity <= 8);
    drop(sample);

    let (first_counts, first_count) = advance_counts(capacity, len);
    for &first in &first_counts[..first_count] {
        assert!(first <= capacity);
        let remaining_capacity = capacity - first;
        let remaining_len = len.saturating_sub(first);
        let (second_counts, second_count) = advance_counts(remaining_capacity, remaining_len);

        for &second in &second_counts[..second_count] {
            assert!(second <= remaining_capacity);
            for value in [0, u8::MAX] {
                // Every request is valid without clamping. The extracted
                // entry checks the twice-advanced view and current contents,
                // then recovers the original allocation.
                unique_advance(make_input(), first, second, value);
            }
        }
    }
}

#[test]
fn unique_advances_cover_zero_partial_visible_and_capacity_counts() {
    exercise_fixture(empty_zero_capacity, 0, 0);
    exercise_fixture(empty_spare_capacity, 0, 8);
    exercise_fixture(initialized_full_capacity, 4, 4);
    exercise_fixture(initialized_spare_capacity, 4, 8);
}
