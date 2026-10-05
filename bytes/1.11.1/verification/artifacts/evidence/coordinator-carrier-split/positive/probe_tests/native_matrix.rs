//! Actual two-split lifecycle in every explicit retirement order.
use bytes_coordinator_carrier_split::carrier_split;

fn input(kind: usize) -> Vec<u8> {
    match kind {
        0 => Vec::new(),
        1 => Vec::with_capacity(12),
        2 => Vec::from(Box::<[u8]>::from([1, 2, 3, 4, 5, 6])),
        _ => {
            let mut bytes = Vec::with_capacity(12);
            bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
            bytes
        }
    }
}

#[test]
fn actual_two_splits_mutation_and_all_six_retirement_orders() {
    for kind in 0..4 {
        let sample = input(kind);
        let points = [0, 1, sample.len()/2, sample.len(), sample.capacity(),
            sample.capacity().saturating_add(1), usize::MAX];
        for first in points {
            for second in points {
                for order in 0..6 {
                    for value in [0, 255] {
                        carrier_split(input(kind), first, second, order, value);
                    }
                }
            }
        }
    }
}
