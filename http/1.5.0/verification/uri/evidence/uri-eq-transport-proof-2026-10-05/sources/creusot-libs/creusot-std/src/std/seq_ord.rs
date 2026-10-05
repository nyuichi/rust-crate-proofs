//! Recursive lexicographic order for logical sequences.
//!
//! The helper proofs use only the element type's `OrdLogic` interface. They
//! never call `Seq` ordering, so the later `OrdLogic for Seq<T>` implementation
//! does not assume its own laws.

use crate::prelude::*;
use core::cmp::Ordering;

/// Lexicographically compare two finite sequences.
#[logic(open)]
#[variant(left.len() + right.len())]
pub fn seq_cmp<T: OrdLogic>(left: Seq<T>, right: Seq<T>) -> Ordering {
    pearlite! {
        if left.len() == 0 {
            if right.len() == 0 { Ordering::Equal } else { Ordering::Less }
        } else if right.len() == 0 {
            Ordering::Greater
        } else {
            match left[0].cmp_log(right[0]) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => seq_cmp(left.tail(), right.tail()),
            }
        }
    }
}

/// Equality of mathematical values makes the scalar `char` and `u8` orders
/// agree. This bridges their distinct Rust types without using an ordering
/// axiom or a trusted conversion.
#[logic(open)]
#[requires(left_char@ == left_byte@)]
#[requires(right_char@ == right_byte@)]
#[ensures(left_char.cmp_log(right_char) == left_byte.cmp_log(right_byte))]
pub fn char_u8_cmp_transport(
    left_char: char,
    right_char: char,
    left_byte: u8,
    right_byte: u8,
) {
    proof_assert! {
        (left_char < right_char) == (left_byte < right_byte)
    };
    proof_assert! {
        (left_char == right_char) == (left_byte == right_byte)
    };
}

/// Lexicographic sequence comparison is preserved when corresponding `char`
/// and `u8` elements have the same mathematical values.
#[logic(open)]
#[variant(left_chars.len() + right_chars.len())]
#[requires(left_chars.len() == left_bytes.len())]
#[requires(right_chars.len() == right_bytes.len())]
#[requires(forall<i> 0 <= i && i < left_chars.len()
    ==> left_chars[i]@ == left_bytes[i]@)]
#[requires(forall<i> 0 <= i && i < right_chars.len()
    ==> right_chars[i]@ == right_bytes[i]@)]
#[ensures(seq_cmp(left_chars, right_chars) == seq_cmp(left_bytes, right_bytes))]
pub fn seq_cmp_char_u8_transport(
    left_chars: Seq<char>,
    right_chars: Seq<char>,
    left_bytes: Seq<u8>,
    right_bytes: Seq<u8>,
) {
    if left_chars.len() > 0 && right_chars.len() > 0 {
        let left_char = left_chars[0];
        let right_char = right_chars[0];
        let left_byte = left_bytes[0];
        let right_byte = right_bytes[0];
        char_u8_cmp_transport(left_char, right_char, left_byte, right_byte);

        if left_char.cmp_log(right_char) == Ordering::Equal {
            proof_assert! {
                left_chars.tail().len() == left_bytes.tail().len()
            };
            proof_assert! {
                right_chars.tail().len() == right_bytes.tail().len()
            };
            proof_assert! {
                forall<i> 0 <= i && i < left_chars.tail().len()
                    ==> left_chars.tail()[i]@ == left_bytes.tail()[i]@
            };
            proof_assert! {
                forall<i> 0 <= i && i < right_chars.tail().len()
                    ==> right_chars.tail()[i]@ == right_bytes.tail()[i]@
            };
            seq_cmp_char_u8_transport(
                left_chars.tail(),
                right_chars.tail(),
                left_bytes.tail(),
                right_bytes.tail(),
            );
        }
    }
}

#[logic(open)]
pub fn reverse_ordering(order: Ordering) -> Ordering {
    match order {
        Ordering::Less => Ordering::Greater,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Less,
    }
}

/// Comparator equality is exactly sequence equality.
#[logic(open)]
#[ensures((seq_cmp(left, right) == Ordering::Equal) == (left == right))]
#[variant(left.len() + right.len())]
pub fn seq_eq_cmp<T: OrdLogic>(left: Seq<T>, right: Seq<T>) {
    if pearlite! {
        left.len() > 0 && right.len() > 0
            && left[0].cmp_log(right[0]) == Ordering::Equal
    } {
        proof_assert!(left[0] == right[0]);
        proof_assert!(left == left.tail().push_front(left[0]));
        proof_assert!(right == right.tail().push_front(right[0]));
        seq_eq_cmp(left.tail(), right.tail());
    }
}

/// Reversing the operands reverses the comparison result.
#[logic(open)]
#[ensures(seq_cmp(right, left) == reverse_ordering(seq_cmp(left, right)))]
#[variant(left.len() + right.len())]
pub fn seq_reverse_cmp<T: OrdLogic>(left: Seq<T>, right: Seq<T>) {
    if pearlite! { left.len() > 0 && right.len() > 0 } {
        match left[0].cmp_log(right[0]) {
            Ordering::Less => <T as OrdLogic>::antisym1(left[0], right[0]),
            Ordering::Equal => {
                proof_assert!(left[0] == right[0]);
                seq_reverse_cmp(left.tail(), right.tail());
            }
            Ordering::Greater => <T as OrdLogic>::antisym2(left[0], right[0]),
        }
    }
}

/// Transitivity for comparisons with the same result.
#[logic(open)]
#[requires(seq_cmp(first, second) == order)]
#[requires(seq_cmp(second, third) == order)]
#[ensures(seq_cmp(first, third) == order)]
#[variant(first.len() + second.len() + third.len())]
pub fn seq_cmp_trans<T: OrdLogic>(first: Seq<T>, second: Seq<T>, third: Seq<T>, order: Ordering) {
    if pearlite! {
        first.len() > 0 && second.len() > 0 && third.len() > 0
            && first[0].cmp_log(second[0]) == Ordering::Equal
            && second[0].cmp_log(third[0]) == Ordering::Equal
    } {
        proof_assert!(first[0] == second[0]);
        proof_assert!(second[0] == third[0]);
        seq_cmp_trans(first.tail(), second.tail(), third.tail(), order);
    }
}
