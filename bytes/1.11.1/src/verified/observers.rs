//! Equality observers backed by the verified standard slice comparison contract.

use alloc::vec::Vec;
use creusot_std::prelude::*;
use core::{borrow::Borrow, cmp::Ordering};

use super::exclusive::ExclusiveBytes;

impl DeepModel for ExclusiveBytes {
    type DeepModelTy = Seq<Int>;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@.map(|byte: u8| byte@) }
    }
}

impl Borrow<[u8]> for ExclusiveBytes {
    #[ensures(result.deep_model() == self.deep_model())]
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl PartialEq for ExclusiveBytes {
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &Self) -> bool {
        <Self as AsRef<[u8]>>::as_ref(self) == <Self as AsRef<[u8]>>::as_ref(other)
    }
}

impl Eq for ExclusiveBytes {}

impl PartialEq<[u8]> for ExclusiveBytes {
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialOrd<[u8]> for ExclusiveBytes {
    #[ensures(result == self.deep_model().partial_cmp_log(other.deep_model()))]
    fn partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
        Some(self.as_slice().cmp(other))
    }
}

impl PartialEq<ExclusiveBytes> for [u8] {
    #[ensures(result == (self.deep_model() == other.deep_model()))]
    fn eq(&self, other: &ExclusiveBytes) -> bool {
        self == other.as_slice()
    }
}

impl PartialOrd<ExclusiveBytes> for [u8] {
    #[ensures(result == self.deep_model().partial_cmp_log(other.deep_model()))]
    fn partial_cmp(&self, other: &ExclusiveBytes) -> Option<Ordering> {
        Some(self.cmp(other.as_slice()))
    }
}

impl ExclusiveBytes {
    /// Compares this owner's bytes with a string's UTF-8 bytes using a temporary copy.
    #[ensures(result == (self.deep_model() == other@.to_bytes().map(|byte: u8| byte@)))]
    pub fn eq_str_copy(&self, other: &str) -> bool {
        let copy = Self::copy_from_str(other);
        let equal = self.as_slice() == copy.as_slice();
        copy.close();
        equal
    }

    /// Compares this owner's bytes with a string's UTF-8 bytes using a temporary copy.
    #[ensures(result == self.deep_model().partial_cmp_log(
        other@.to_bytes().map(|byte: u8| byte@)))]
    pub fn cmp_str_copy(&self, other: &str) -> Option<Ordering> {
        let copy = Self::copy_from_str(other);
        let ordering = self.as_slice().cmp(copy.as_slice());
        copy.close();
        Some(ordering)
    }

    /// Compares this owner's bytes with a String's UTF-8 bytes using a temporary copy.
    #[ensures(result == (self.deep_model() == other@.to_bytes().map(|byte: u8| byte@)))]
    pub fn eq_string_copy(&self, other: &alloc::string::String) -> bool {
        self.eq_str_copy(other)
    }

    /// Compares this owner's bytes with a String's UTF-8 bytes using a temporary copy.
    #[ensures(result == self.deep_model().partial_cmp_log(
        other@.to_bytes().map(|byte: u8| byte@)))]
    pub fn cmp_string_copy(&self, other: &alloc::string::String) -> Option<Ordering> {
        self.cmp_str_copy(other)
    }
}

impl PartialOrd for ExclusiveBytes {
    #[ensures(result == (*self).deep_model().partial_cmp_log((*other).deep_model()))]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(<Self as Ord>::cmp(self, other))
    }
}

impl Ord for ExclusiveBytes {
    #[ensures(result == (*self).deep_model().cmp_log((*other).deep_model()))]
    fn cmp(&self, other: &Self) -> Ordering {
        <Self as AsRef<[u8]>>::as_ref(self).cmp(<Self as AsRef<[u8]>>::as_ref(other))
    }
}

/// Establishes that a prefix ending at the sequence length is the full sequence.
#[check(ghost)]
#[ensures((*source).subsequence(0, (*source).len()) == *source)]
fn full_prefix(source: Snapshot<Seq<Int>>) {
    let prefix = snapshot!((*source).subsequence(0, (*source).len()));
    proof_assert!((*prefix).len() == (*source).len());
    proof_assert!(forall<i: Int> 0 <= i && i < (*source).len() ==>
        (*prefix)[i] == (*source)[i]);
    proof_assert!((*prefix).ext_eq(*source));
}

/// Rejoins a prefix and its next element using explicit sequence extensionality.
#[check(ghost)]
#[requires(0 <= *index && *index < (*source).len())]
#[ensures((*source).subsequence(0, *index + 1)
    == (*source).subsequence(0, *index).push_back((*source)[*index]))]
fn prefix_snoc(source: Snapshot<Seq<Int>>, index: Snapshot<Int>) {
    let prefix = snapshot!((*source).subsequence(0, *index));
    let longer = snapshot!((*source).subsequence(0, *index + 1));
    let appended = snapshot!((*prefix).push_back((*source)[*index]));

    proof_assert!((*prefix).len() == *index);
    proof_assert!((*longer).len() == *index + 1);
    proof_assert!((*appended).len() == *index + 1);
    proof_assert!(forall<i: Int> 0 <= i && i < (*longer).len() ==>
        if i < *index {
            (*longer)[i] == (*prefix)[i]
        } else {
            (*longer)[i] == (*source)[*index]
        });
    proof_assert!((*longer).ext_eq(*appended));
}

/// Compares two exclusive owners, then releases both allocations explicitly.
#[ensures(result == (left.deep_model() == right.deep_model()))]
pub fn equal_then_close(left: ExclusiveBytes, right: ExclusiveBytes) -> bool {
    let equal = left == right;
    left.close();
    right.close();
    equal
}

/// Consumes the checked `Borrow<[u8]>` relation through a generic caller.
#[ensures(result == (source.deep_model() == expected.deep_model()))]
fn borrowed_matches<T: Borrow<[u8]> + DeepModel<DeepModelTy = Seq<Int>>>(
    source: &T,
    expected: &[u8],
) -> bool {
    <T as Borrow<[u8]>>::borrow(source) == expected
}

/// Exercises generic borrowing, then explicitly closes the byte owner.
#[ensures(result == (owner.deep_model() == expected.deep_model()))]
pub(crate) fn borrow_matches_then_close(owner: ExclusiveBytes, expected: &[u8]) -> bool {
    let equal = borrowed_matches(&owner, expected);
    owner.close();
    equal
}

/// Orders two exclusive owners, then explicitly releases both allocations.
#[ensures(result == left.deep_model().cmp_log(right.deep_model()))]
pub fn cmp_then_close(left: ExclusiveBytes, right: ExclusiveBytes) -> Ordering {
    let ordering = left.cmp(&right);
    left.close();
    right.close();
    ordering
}

/// Stable 64-bit polynomial digest over bytes, processed from left to right.
///
/// This is a deterministic summary function, not Rust's `Hash` protocol and
/// not a cryptographic digest.
#[logic(open)]
pub fn stable_digest_step_model(prefix: Int, byte: Int) -> Int {
    pearlite! {
        ((prefix * 257) % 18446744073709551616 + byte) % 18446744073709551616
    }
}

#[logic(open)]
#[variant(bytes.len())]
pub fn stable_digest_model(bytes: Seq<Int>) -> Int {
    pearlite! {
        if bytes.len() == 0 {
            0
        } else {
            stable_digest_step_model(
                stable_digest_model(bytes.subsequence(0, bytes.len() - 1)),
                bytes[bytes.len() - 1],
            )
        }
    }
}

/// Establishes the one-byte recurrence of the mathematical digest model.
#[check(ghost)]
#[ensures(stable_digest_model((*prefix).push_back(*byte))
    == stable_digest_step_model(stable_digest_model(*prefix), *byte))]
fn stable_digest_model_snoc(prefix: Snapshot<Seq<Int>>, byte: Snapshot<Int>) {
    let appended = snapshot!((*prefix).push_back(*byte));
    proof_assert!((*appended).len() == (*prefix).len() + 1);
    proof_assert!((*appended).subsequence(0, (*appended).len() - 1).ext_eq(*prefix));
    proof_assert!((*appended)[(*appended).len() - 1] == *byte);
}

/// Executes one wrapping digest step and relates it to the integer model.
#[ensures(result@ == stable_digest_step_model(prefix@, byte@))]
fn wrapping_digest_step(prefix: u64, byte: u8) -> u64 {
    prefix.wrapping_mul(257u64).wrapping_add(byte as u64)
}

/// A lowercase ASCII hex digit. The precondition keeps the addition in range.
#[logic(open)]
pub fn hex_digit_model(nibble: Int) -> Int {
    pearlite! {
        if nibble < 10 { 48 + nibble } else { 87 + nibble }
    }
}

#[requires(nibble@ <= 15)]
#[ensures(result@ == hex_digit_model(nibble@))]
fn encode_hex_digit(nibble: u8) -> u8 {
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    }
}

/// The two lowercase hexadecimal digits for one byte.
#[logic(open)]
pub fn hex_byte_model(byte: Int) -> Seq<Int> {
    pearlite! {
        Seq::singleton(hex_digit_model(byte / 16))
            .push_back(hex_digit_model(byte % 16))
    }
}

/// The concatenated lowercase hexadecimal representation of a byte sequence.
#[logic(open)]
#[variant(bytes.len())]
pub fn hex_bytes_model(bytes: Seq<Int>) -> Seq<Int> {
    pearlite! {
        if bytes.len() == 0 {
            Seq::empty()
        } else {
            hex_bytes_model(bytes.subsequence(0, bytes.len() - 1))
                .concat(hex_byte_model(bytes[bytes.len() - 1]))
        }
    }
}

/// Extends the hex model by the exact two-digit sequence for one byte.
#[check(ghost)]
#[ensures(hex_bytes_model((*prefix).push_back(*byte))
    == hex_bytes_model(*prefix).concat(hex_byte_model(*byte)))]
fn hex_bytes_model_snoc(prefix: Snapshot<Seq<Int>>, byte: Snapshot<Int>) {
    let bytes = snapshot!((*prefix).push_back(*byte));
    let prior = snapshot!((*bytes).subsequence(0, (*bytes).len() - 1));
    let extended = snapshot!(hex_bytes_model(*bytes));
    let expected = snapshot!(hex_bytes_model(*prefix).concat(hex_byte_model(*byte)));
    proof_assert!((*bytes).len() == (*prefix).len() + 1);
    proof_assert!((*prior).len() == (*prefix).len());
    proof_assert!((*prior).ext_eq(*prefix));
    proof_assert!((*bytes)[(*bytes).len() - 1] == *byte);
    proof_assert!((*extended).ext_eq(*expected));
}

impl ExclusiveBytes {
    /// Computes the specified 64-bit polynomial digest over this byte sequence.
    #[ensures(result@ == stable_digest_model(self.deep_model()))]
    pub fn stable_digest(&self) -> u64 {
        let input = self.as_slice();
        let mut digest = 0u64;
        let mut index = 0usize;

        #[invariant(index@ <= input@.len())]
        #[invariant(digest@ == stable_digest_model(self.deep_model().subsequence(0, index@)))]
        #[variant(input@.len() - index@)]
        while index < input.len() {
            let source = snapshot!(self.deep_model());
            let position = snapshot!(index@);
            let prefix = snapshot!((*source).subsequence(0, *position));
            let byte = snapshot!((*source)[*position]);
            ghost! {
                prefix_snoc(source, position);
                stable_digest_model_snoc(prefix, byte);
            };
            digest = wrapping_digest_step(digest, input[index]);
            index += 1;
        }

        ghost! { full_prefix(snapshot!(self.deep_model())); };

        digest
    }

    /// Creates a new owner containing two lowercase hex digits per input byte.
    ///
    /// The returned owner has the exact byte model in `hex_bytes_model`; callers
    /// release it explicitly with [`ExclusiveBytes::close`].
    #[ensures(result.deep_model() == hex_bytes_model(self.deep_model()))]
    pub fn hex_bytes(&self) -> ExclusiveBytes {
        let input = self.as_slice();
        let mut output = ExclusiveBytes::from_vec(Vec::new());
        let mut index = 0usize;

        #[invariant(index@ <= input@.len())]
        #[invariant(output.deep_model() ==
            hex_bytes_model(self.deep_model().subsequence(0, index@)))]
        #[variant(input@.len() - index@)]
        while index < input.len() {
            let byte = input[index];
            let pair = [encode_hex_digit(byte / 16), encode_hex_digit(byte % 16)];
            let source = snapshot!(self.deep_model());
            let position = snapshot!(index@);
            let prefix = snapshot!((*source).subsequence(0, *position));
            let model_byte = snapshot!((*source)[*position]);
            ghost! {
                prefix_snoc(source, position);
                hex_bytes_model_snoc(prefix, model_byte);
            };
            output.append_initialized_slice(&pair);
            index += 1;
        }

        ghost! { full_prefix(snapshot!(self.deep_model())); };

        output
    }
}

/// Computes both summaries, then explicitly closes the source owner.
#[ensures(result.0@ == stable_digest_model(source.deep_model()))]
#[ensures(result.1.deep_model() == hex_bytes_model(source.deep_model()))]
pub fn digest_and_hex_close(source: ExclusiveBytes) -> (u64, ExclusiveBytes) {
    let digest = source.stable_digest();
    let encoded = source.hex_bytes();
    source.close();
    (digest, encoded)
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn ordering_matches_lexicographic_slice_order_and_closes() {
        let left = ExclusiveBytes::copy_from_slice(b"byte");
        let right = ExclusiveBytes::copy_from_slice(b"bytes");
        assert_eq!(cmp_then_close(left, right), Ordering::Less);
    }

    #[test]
    fn borrow_trait_returns_the_same_bytes_and_closes_explicitly() {
        assert!(super::borrow_matches_then_close(
            ExclusiveBytes::copy_from_slice(b"borrow"),
            b"borrow",
        ));
        assert!(!super::borrow_matches_then_close(
            ExclusiveBytes::copy_from_slice(b"borrow"),
            b"Borrow",
        ));
    }

    #[test]
    fn cross_slice_comparisons_match_slice_equality_and_order() {
        let owner = ExclusiveBytes::copy_from_slice(b"byte");
        assert!(owner == b"byte"[..]);
        assert!(b"byte"[..] == owner);
        assert!(owner < b"bytes"[..]);
        assert!(b"byt"[..] < owner);
        assert!(owner != b"bytes"[..]);
        owner.close();
    }

    #[test]
    fn explicit_string_comparison_uses_utf8_bytes_and_closes_temporary_copies() {
        let owner = ExclusiveBytes::copy_from_slice("café".as_bytes());
        assert!(owner.eq_str_copy("café"));
        assert!(!owner.eq_str_copy("cafe"));
        assert_eq!(owner.cmp_str_copy("cafê"), Some(Ordering::Less));

        let equal = alloc::string::String::from("café");
        let after = alloc::string::String::from("cafê");
        assert!(owner.eq_string_copy(&equal));
        assert_eq!(owner.cmp_string_copy(&after), Some(Ordering::Less));
        owner.close();
    }
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod deterministic_tests {
    use super::{digest_and_hex_close, ExclusiveBytes};
    use std::vec;

    #[test]
    fn deterministic_digest_and_hex_model_match_and_close() {
        let (digest, encoded) = digest_and_hex_close(
            ExclusiveBytes::from_vec(vec![0x00, 0x0a, 0x10, 0xff]),
        );
        let expected_digest = (((0u64.wrapping_mul(257).wrapping_add(0x00))
            .wrapping_mul(257).wrapping_add(0x0a))
            .wrapping_mul(257).wrapping_add(0x10))
            .wrapping_mul(257).wrapping_add(0xff);
        assert_eq!(digest, expected_digest);
        assert_eq!(encoded.as_slice(), b"000a10ff");
        encoded.close();
    }

    #[test]
    fn empty_deterministic_summaries_close() {
        let (digest, encoded) = digest_and_hex_close(ExclusiveBytes::from_vec(vec![]));
        assert_eq!(digest, 0);
        assert!(encoded.is_empty());
        encoded.close();
    }
}
