//! Exercise the actual unique-handle trait implementations across repeated
//! advances, including requests into spare capacity that the entry clamps.
use bytes_valid_handle_traits::traits_unique_advance;

fn fixture(kind: usize) -> Vec<u8> {
    match kind {
        // Empty allocation.
        0 => Vec::new(),
        // Empty visible view with spare allocation.
        1 => Vec::with_capacity(8),
        // Full allocation with an initialized prefix.
        2 => Vec::from(Box::<[u8]>::from([1, 2, 3, 4])),
        // Initialized prefix followed by spare capacity.
        _ => {
            let mut bytes = Vec::with_capacity(8);
            bytes.extend_from_slice(&[1, 2, 3, 4]);
            bytes
        }
    }
}

fn points(len: usize, capacity: usize) -> Vec<usize> {
    let candidates = [
        0,
        1,
        len / 2,
        len,
        capacity,
        capacity.saturating_add(1),
        usize::MAX,
    ];
    let mut result = Vec::new();
    for point in candidates {
        if !result.contains(&point) {
            result.push(point);
        }
    }
    result
}

#[test]
fn unique_trait_calls_cover_repeated_advance_bounds_and_cleanup() {
    for kind in 0..4 {
        let sample = fixture(kind);
        let len = sample.len();
        let capacity = sample.capacity();
        let counts = points(len, capacity);

        // The entry clamps each request to the current handle bounds. Pairing
        // every first and second request covers repeated advances from zero,
        // through the visible prefix, and into spare capacity up to the end.
        for &first in &counts {
            for &second in &counts {
                for value in [0, u8::MAX] {
                    traits_unique_advance(fixture(kind), first, second, value);
                }
            }
        }
    }
}
