//! Native edge cases for in-capacity resize/copy operations on unique and
//! split `BytesMut` handles. The probe entries check values and geometry.

use bytes_sequential_bytesmut_split::{noalloc_split, noalloc_unique};
use std::vec::Vec;

const EXTENSION: [u8; 8] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];

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

fn exercise_unique(make_input: fn() -> Vec<u8>, expected_len: usize, min_capacity: usize) {
    let sample = make_input();
    let len = sample.len();
    let capacity = sample.capacity();
    assert_eq!(len, expected_len);
    assert!(capacity >= min_capacity);
    assert!(capacity <= EXTENSION.len());
    drop(sample);

    let mut new_lengths = [0usize; 5];
    let mut new_length_count = 0usize;
    push_unique(&mut new_lengths, &mut new_length_count, 0);
    push_unique(&mut new_lengths, &mut new_length_count, len / 2);
    push_unique(&mut new_lengths, &mut new_length_count, len);
    push_unique(&mut new_lengths, &mut new_length_count, capacity);

    for &new_len in &new_lengths[..new_length_count] {
        let spare = capacity - new_len;
        let mut extension_lengths = [0usize; 3];
        let mut extension_count = 0usize;
        push_unique(&mut extension_lengths, &mut extension_count, 0);
        if spare > 0 {
            push_unique(&mut extension_lengths, &mut extension_count, 1);
        }
        push_unique(&mut extension_lengths, &mut extension_count, spare);

        for &extension_len in &extension_lengths[..extension_count] {
            assert!(extension_len <= spare);
            for value in [0, u8::MAX] {
                // Requests are valid as given; the probe entry checks that
                // resize and append stay within the original allocation.
                noalloc_unique(make_input(), new_len, value, &EXTENSION[..extension_len]);
            }
        }
    }
}

#[test]
fn unique_resize_and_append_cover_empty_full_and_spare_capacity() {
    exercise_unique(empty_zero_capacity, 0, 0);
    exercise_unique(empty_spare_capacity, 0, 8);
    exercise_unique(initialized_full_capacity, 4, 4);
    exercise_unique(initialized_spare_capacity, 4, 8);
}

fn exercise_split(make_input: fn() -> Vec<u8>, expected_len: usize, min_capacity: usize) {
    let sample = make_input();
    let len = sample.len();
    let capacity = sample.capacity();
    assert_eq!(len, expected_len);
    assert!(capacity >= min_capacity);
    assert!(capacity <= EXTENSION.len());
    drop(sample);

    let mut split_points = [0usize; 3];
    let mut split_count = 0usize;
    push_unique(&mut split_points, &mut split_count, 0);
    push_unique(&mut split_points, &mut split_count, len / 2);
    push_unique(&mut split_points, &mut split_count, len);

    for &split in &split_points[..split_count] {
        let right_len = len - split;
        let right_capacity = capacity - split;
        let mut new_lengths = [0usize; 4];
        let mut new_length_count = 0usize;
        push_unique(&mut new_lengths, &mut new_length_count, 0);
        push_unique(&mut new_lengths, &mut new_length_count, right_len / 2);
        push_unique(&mut new_lengths, &mut new_length_count, right_len);
        push_unique(&mut new_lengths, &mut new_length_count, right_capacity);

        for &new_len in &new_lengths[..new_length_count] {
            assert!(new_len <= right_capacity);
            let spare = right_capacity - new_len;
            let mut extension_lengths = [0usize; 3];
            let mut extension_count = 0usize;
            push_unique(&mut extension_lengths, &mut extension_count, 0);
            if spare > 0 {
                push_unique(&mut extension_lengths, &mut extension_count, 1);
            }
            push_unique(&mut extension_lengths, &mut extension_count, spare);

            for &extension_len in &extension_lengths[..extension_count] {
                assert!(extension_len <= spare);
                for value in [0, u8::MAX] {
                    for left_first in [false, true] {
                        // `noalloc_split` uses split_to: it resizes/appends to
                        // the retained right handle and checks the returned
                        // left handle remains unchanged.
                        noalloc_split(
                            make_input(),
                            split,
                            new_len,
                            value,
                            &EXTENSION[..extension_len],
                            left_first,
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn split_resize_and_append_cover_both_release_orders() {
    exercise_split(empty_zero_capacity, 0, 0);
    exercise_split(empty_spare_capacity, 0, 8);
    exercise_split(initialized_full_capacity, 4, 4);
    exercise_split(initialized_spare_capacity, 4, 8);
}
