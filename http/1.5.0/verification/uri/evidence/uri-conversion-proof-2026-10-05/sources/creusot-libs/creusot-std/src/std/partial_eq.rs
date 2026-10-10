//! Logical equality relations between potentially different model types.
//!
//! Rust's `PartialEq<Rhs>` permits heterogeneous comparisons. Deep models need
//! not have the same Rust type for those comparisons (for example, a byte
//! sequence compared with a string's character sequence), so this relation
//! connects the two model types without changing their runtime APIs.

use crate::prelude::*;

/// The logical relation corresponding to a heterogeneous `PartialEq` pair.
pub trait PartialEqModel<Rhs> {
    /// Compare two model values using the semantics of their source values.
    #[logic]
    fn eq_model(self, rhs: Rhs) -> bool;
}

/// Values with the same model type use ordinary logical equality.
impl<T> PartialEqModel<T> for T {
    #[logic(open)]
    fn eq_model(self, rhs: T) -> bool {
        pearlite! { self == rhs }
    }
}

/// A byte model equals a string model exactly when it is the string's UTF-8
/// encoding. This relation is directional only in its Rust types; the reverse
/// implementation below has the same Boolean definition.
impl PartialEqModel<Seq<char>> for Seq<u8> {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { rhs.to_bytes() == self }
    }
}

impl PartialEqModel<Seq<u8>> for Seq<char> {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<u8>) -> bool {
        pearlite! { self.to_bytes() == rhs }
    }
}

/// A runtime byte slice has `Seq<Int>` as its deep model, whereas `Bytes`
/// exposes `Seq<u8>`. Their equality is pointwise equality of the byte values.
impl PartialEqModel<Seq<Int>> for Seq<u8> {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<Int>) -> bool {
        pearlite! {
            self.len() == rhs.len()
            && forall<i> 0 <= i && i < self.len() ==> self[i]@ == rhs[i]
        }
    }
}

impl PartialEqModel<Seq<u8>> for Seq<Int> {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<u8>) -> bool {
        pearlite! {
            self.len() == rhs.len()
            && forall<i> 0 <= i && i < self.len() ==> self[i] == rhs[i]@
        }
    }
}

/// Relate the byte-to-integer equality model used by slice deep models to
/// ordinary equality between byte sequences.
///
/// A slice of `u8` has `Seq<Int>` as its deep model. This bridge says that the
/// pointwise numeric relation is exactly equality with the same bytes viewed
/// as `Seq<u8>`; it does not impose any relationship between the left and
/// right sequence lengths.
#[logic(opaque)]
#[requires(right_bytes.len() == right_integers.len())]
#[requires(forall<i> 0 <= i && i < right_bytes.len() ==> right_bytes[i]@ == right_integers[i])]
#[ensures(result)]
#[ensures(result == (left_bytes.eq_model(right_integers) == (left_bytes == right_bytes)))]
pub fn seq_eq_u8_int_view_transport(
    left_bytes: Seq<u8>,
    right_bytes: Seq<u8>,
    right_integers: Seq<Int>,
) -> bool {
    proof_assert! {
        left_bytes.eq_model(right_integers) == (left_bytes == right_bytes)
    };
    true
}
