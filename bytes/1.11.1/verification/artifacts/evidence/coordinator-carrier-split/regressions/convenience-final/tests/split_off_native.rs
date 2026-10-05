//! Native edge cases for the extracted `BytesMut::split_off` path.
use bytes_sequential_bytesmut_split::split_off_both;
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

    let mut points = [0usize; 5];
    let mut point_count = 0usize;
    push_unique(&mut points, &mut point_count, 0);
    push_unique(&mut points, &mut point_count, len);
    if len > 1 {
        push_unique(&mut points, &mut point_count, len / 2);
    }
    if capacity - len > 1 {
        push_unique(
            &mut points,
            &mut point_count,
            len + (capacity - len) / 2,
        );
    }
    push_unique(&mut points, &mut point_count, capacity);

    for &at in &points[..point_count] {
        for right_first in [false, true] {
            // The extracted helper checks geometry, reads/mutates only bytes
            // inside each handle's logical length, and explicitly releases
            // both registrations in the requested order.
            split_off_both(make_input(), at, right_first);
        }
    }
}

#[test]
fn split_off_edge_matrix_preserves_geometry_and_releases_in_both_orders() {
    // Includes at=0/capacity for the zero-allocation input and at=0/len/
    // interior/capacity across full and spare-capacity initialized buffers.
    exercise_fixture(empty_zero_capacity, 0, 0, true);
    exercise_fixture(empty_spare_capacity, 0, 8, false);
    exercise_fixture(initialized_full_capacity, 4, 4, true);
    exercise_fixture(initialized_spare_capacity, 4, 8, false);
}
