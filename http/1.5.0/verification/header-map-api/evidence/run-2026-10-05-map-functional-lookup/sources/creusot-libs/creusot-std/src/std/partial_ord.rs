//! Logical comparison relations between potentially different model types.

use crate::prelude::*;
use crate::std::seq_ord::{reverse_ordering, seq_cmp, seq_reverse_cmp};
use core::cmp::Ordering;

/// The logical comparison corresponding to a heterogeneous `PartialOrd` pair.
pub trait PartialOrdModel<Rhs> {
    #[logic]
    fn partial_cmp_model(self, rhs: Rhs) -> Ordering;
}

impl<T: OrdLogic> PartialOrdModel<T> for T {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: T) -> Ordering {
        self.cmp_log(rhs)
    }
}

/// Compare byte and integer sequences by their corresponding numeric values.
#[logic(open)]
#[variant(bytes.len() + integers.len())]
pub fn seq_cmp_u8_int(bytes: Seq<u8>, integers: Seq<Int>) -> Ordering {
    pearlite! {
        if bytes.len() == 0 {
            if integers.len() == 0 { Ordering::Equal } else { Ordering::Less }
        } else if integers.len() == 0 {
            Ordering::Greater
        } else {
            match bytes[0]@.cmp_log(integers[0]) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => seq_cmp_u8_int(bytes.tail(), integers.tail()),
            }
        }
    }
}

/// A byte's logical integer view preserves its scalar comparison result.
#[logic(open)]
#[requires(integer == other_byte@)]
#[ensures(left_byte@.cmp_log(integer) == left_byte.cmp_log(other_byte))]
pub fn u8_int_cmp_transport(left_byte: u8, other_byte: u8, integer: Int) {
    proof_assert!(left_byte@.cmp_log(integer) == left_byte@.cmp_log(other_byte@));
    proof_assert!(left_byte@.cmp_log(other_byte@) == left_byte.cmp_log(other_byte));
}

/// The heterogeneous comparator agrees with ordinary byte-sequence order
/// when the integer sequence is the pointwise model of the byte sequence.
#[logic(open)]
#[variant(bytes.len() + integers.len())]
#[requires(other_bytes.len() == integers.len())]
#[requires(forall<i> 0 <= i && i < integers.len()
    ==> other_bytes[i]@ == integers[i])]
#[ensures(seq_cmp_u8_int(bytes, integers) == seq_cmp(bytes, other_bytes))]
pub fn seq_cmp_u8_int_transport(
    bytes: Seq<u8>,
    other_bytes: Seq<u8>,
    integers: Seq<Int>,
) {
    if bytes.len() > 0 && integers.len() > 0 {
        let left_byte = bytes[0];
        let other_byte = other_bytes[0];
        let integer = integers[0];
        u8_int_cmp_transport(left_byte, other_byte, integer);

        if pearlite! { left_byte@.cmp_log(integer) == Ordering::Equal } {
            proof_assert!(left_byte.cmp_log(other_byte) == Ordering::Equal);
            proof_assert!(other_bytes.tail().len() == integers.tail().len());
            proof_assert!(forall<i> 0 <= i && i < integers.tail().len()
                ==> other_bytes.tail()[i]@ == integers.tail()[i]);
            seq_cmp_u8_int_transport(bytes.tail(), other_bytes.tail(), integers.tail());
        }
    }
}

/// Lexicographic order is preserved when each byte sequence is paired with
/// its pointwise integer model. The compared sequences may have different
/// lengths; only the two views of each operand must have equal lengths.
#[logic(open)]
#[variant(left_bytes.len() + left_integers.len() + right_bytes.len() + right_integers.len())]
#[requires(left_bytes.len() == left_integers.len())]
#[requires(right_bytes.len() == right_integers.len())]
#[requires(forall<i> 0 <= i && i < left_bytes.len()
    ==> left_bytes[i]@ == left_integers[i])]
#[requires(forall<i> 0 <= i && i < right_bytes.len()
    ==> right_bytes[i]@ == right_integers[i])]
#[ensures(seq_cmp(left_integers, right_integers) == seq_cmp(left_bytes, right_bytes))]
pub fn seq_cmp_u8_int_pair_transport(
    left_bytes: Seq<u8>,
    left_integers: Seq<Int>,
    right_bytes: Seq<u8>,
    right_integers: Seq<Int>,
) {
    if left_bytes.len() > 0 && right_bytes.len() > 0 {
        let left_byte = left_bytes[0];
        let left_integer = left_integers[0];
        let right_byte = right_bytes[0];
        let right_integer = right_integers[0];

        proof_assert!(left_byte@ == left_integer);
        proof_assert!(right_byte@ == right_integer);
        u8_int_cmp_transport(left_byte, right_byte, right_integer);
        proof_assert!(
            left_integer.cmp_log(right_integer) == left_byte.cmp_log(right_byte)
        );

        if left_integer.cmp_log(right_integer) == Ordering::Equal {
            proof_assert!(left_bytes.tail().len() == left_integers.tail().len());
            proof_assert!(right_bytes.tail().len() == right_integers.tail().len());
            proof_assert!(forall<i> 0 <= i && i < left_bytes.tail().len()
                ==> left_bytes.tail()[i]@ == left_integers.tail()[i]);
            proof_assert!(forall<i> 0 <= i && i < right_bytes.tail().len()
                ==> right_bytes.tail()[i]@ == right_integers.tail()[i]);
            seq_cmp_u8_int_pair_transport(
                left_bytes.tail(),
                left_integers.tail(),
                right_bytes.tail(),
                right_integers.tail(),
            );
        }
    }
}

/// Opaque caller certificate for the two-sided sequence transport.
#[logic(opaque)]
#[requires(left_bytes.len() == left_integers.len())]
#[requires(right_bytes.len() == right_integers.len())]
#[requires(forall<i> 0 <= i && i < left_bytes.len()
    ==> left_bytes[i]@ == left_integers[i])]
#[requires(forall<i> 0 <= i && i < right_bytes.len()
    ==> right_bytes[i]@ == right_integers[i])]
#[ensures(result)]
#[ensures(result == (
    seq_cmp(left_integers, right_integers) == seq_cmp(left_bytes, right_bytes)
))]
pub fn seq_cmp_u8_int_pair_transport_preserved(
    left_bytes: Seq<u8>,
    left_integers: Seq<Int>,
    right_bytes: Seq<u8>,
    right_integers: Seq<Int>,
) -> bool {
    seq_cmp_u8_int_pair_transport(
        left_bytes,
        left_integers,
        right_bytes,
        right_integers,
    );
    proof_assert! {
        seq_cmp(left_integers, right_integers) == seq_cmp(left_bytes, right_bytes)
    };
    true
}

/// Lexicographic order is preserved when the left byte sequence is replaced
/// by its pointwise integer model. The right operand may have any length.
#[logic(open)]
#[variant(bytes.len() + other_integers.len())]
#[requires(bytes.len() == integers.len())]
#[requires(forall<i> 0 <= i && i < bytes.len()
    ==> bytes[i]@ == integers[i])]
#[ensures(seq_cmp(integers, other_integers) == seq_cmp_u8_int(bytes, other_integers))]
pub fn seq_cmp_u8_int_left_transport(
    bytes: Seq<u8>,
    integers: Seq<Int>,
    other_integers: Seq<Int>,
) {
    if bytes.len() > 0 && other_integers.len() > 0 {
        let byte = bytes[0];
        let integer = integers[0];
        let other_integer = other_integers[0];
        proof_assert!(byte@ == integer);
        proof_assert!(byte@.cmp_log(other_integer) == integer.cmp_log(other_integer));

        if integer.cmp_log(other_integer) == Ordering::Equal {
            proof_assert!(bytes.tail().len() == integers.tail().len());
            proof_assert!(forall<i> 0 <= i && i < bytes.tail().len()
                ==> bytes.tail()[i]@ == integers.tail()[i]);
            seq_cmp_u8_int_left_transport(
                bytes.tail(),
                integers.tail(),
                other_integers.tail(),
            );
        }
    }
}

/// Opaque caller certificate for left-operand byte-to-integer transport.
#[logic(opaque)]
#[requires(bytes.len() == integers.len())]
#[requires(forall<i> 0 <= i && i < bytes.len()
    ==> bytes[i]@ == integers[i])]
#[ensures(result)]
#[ensures(result == (
    seq_cmp(integers, other_integers) == seq_cmp_u8_int(bytes, other_integers)
))]
pub fn seq_cmp_u8_int_left_transport_preserved(
    bytes: Seq<u8>,
    integers: Seq<Int>,
    other_integers: Seq<Int>,
) -> bool {
    seq_cmp_u8_int_left_transport(bytes, integers, other_integers);
    proof_assert! {
        seq_cmp(integers, other_integers) == seq_cmp_u8_int(bytes, other_integers)
    };
    true
}

/// Opaque caller certificate for comparing a byte slice against an integer
/// model on the right while preserving the source's reversed comparison.
#[logic(opaque)]
#[requires(bytes.len() == integers.len())]
#[requires(forall<i> 0 <= i && i < bytes.len()
    ==> bytes[i]@ == integers[i])]
#[ensures(result)]
#[ensures(result == (
    seq_cmp(other_integers, integers)
        == reverse_ordering(seq_cmp_u8_int(bytes, other_integers))
))]
pub fn seq_cmp_u8_int_reverse_transport_preserved(
    bytes: Seq<u8>,
    integers: Seq<Int>,
    other_integers: Seq<Int>,
) -> bool {
    seq_cmp_u8_int_left_transport(bytes, integers, other_integers);
    seq_reverse_cmp(integers, other_integers);
    proof_assert! {
        seq_cmp(other_integers, integers)
            == reverse_ordering(seq_cmp_u8_int(bytes, other_integers))
    };
    true
}

impl PartialOrdModel<Seq<Int>> for Seq<u8> {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<Int>) -> Ordering {
        seq_cmp_u8_int(self, rhs)
    }
}

impl PartialOrdModel<Seq<u8>> for Seq<Int> {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<u8>) -> Ordering {
        reverse_ordering(seq_cmp_u8_int(rhs, self))
    }
}

/// Compare string and byte models using the string's UTF-8 encoding, matching
/// the source APIs that compare header content as raw bytes.
impl PartialOrdModel<Seq<char>> for Seq<u8> {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<char>) -> Ordering {
        self.cmp_log(rhs.to_bytes())
    }
}

impl PartialOrdModel<Seq<u8>> for Seq<char> {
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<u8>) -> Ordering {
        self.to_bytes().cmp_log(rhs)
    }
}
