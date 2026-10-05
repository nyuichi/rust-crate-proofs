//! Actual two-split lifecycle in every explicit retirement order.
use bytes_coordinator_carrier_split::{carrier_split, carrier_split_off};

type CarrierAction = fn(Vec<u8>, usize, usize, u8, u8);
const CARRIER_ACTIONS: [CarrierAction; 2] = [carrier_split, carrier_split_off];

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

fn run_boundary_matrix(action: CarrierAction) {
    for kind in 0..4 {
        let sample = input(kind);
        let points = [
            0,
            1,
            sample.len() / 2,
            sample.len(),
            sample.len().saturating_add(1),
            sample.capacity(),
            sample.capacity().saturating_add(1),
            usize::MAX,
        ];
        for first in points {
            for second in points {
                for order in 0..6 {
                    for value in [0, 255] {
                        action(input(kind), first, second, order, value);
                    }
                }
            }
        }
    }
}

#[test]
fn actual_two_splits_mutation_and_all_six_retirement_orders() {
    run_boundary_matrix(CARRIER_ACTIONS[0]);
}

#[test]
fn actual_split_off_second_split_mutation_and_all_six_retirement_orders() {
    run_boundary_matrix(CARRIER_ACTIONS[1]);
}
