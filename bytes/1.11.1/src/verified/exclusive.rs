//! Exclusive, Vec-backed byte storage for the modified verified API.
//!
//! This type owns one ordinary `Vec<u8>` and does not expose shared aliases.
//! Call [`ExclusiveBytes::close`] to release its allocation through the
//! explicit B1/B3 ownership path.

use alloc::vec::Vec;
use core::{borrow::BorrowMut, ops::{Deref, DerefMut}};

use creusot_std::prelude::*;

use crate::ownership_proof::{bound_ptr, raw_vec};

/// Rejoins the two exact pieces named by a checked split index.
#[check(ghost)]
#[requires(0 <= *index && *index <= (*source).len())]
#[ensures((*source).subsequence(0, *index)
    .concat((*source).subsequence(*index, (*source).len())) == *source)]
fn split_rejoins(source: Snapshot<Seq<u8>>, index: Snapshot<Int>) {
    let prefix = snapshot!((*source).subsequence(0, *index));
    let suffix = snapshot!((*source).subsequence(*index, (*source).len()));
    let joined = snapshot!((*prefix).concat(*suffix));

    proof_assert!((*prefix).len() == *index);
    proof_assert!((*suffix).len() == (*source).len() - *index);
    proof_assert!((*joined).len() == (*source).len());
    proof_assert!(forall<i: Int> 0 <= i && i < *index ==>
        (*joined)[i] == (*source)[i]);
    proof_assert!(forall<i: Int> *index <= i && i < (*source).len() ==>
        (*joined)[i] == (*source)[i]);
    proof_assert!((*joined).ext_eq(*source));
}

/// A byte sequence with one exclusive, Vec-backed owner.
pub struct ExclusiveBytes {
    bytes: Vec<u8>,
}

impl View for ExclusiveBytes {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.bytes@ }
    }
}

impl ExclusiveBytes {
    /// Takes exclusive ownership of an existing byte vector.
    #[ensures(result@ == bytes@)]
    #[check(terminates)]
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Creates an empty byte sequence with room for at least `capacity` bytes.
    #[ensures(result@.len() == 0)]
    #[cfg_attr(creusot, ensures(
        creusot_std::std::vec::capacity_model(result.bytes) >= capacity@
    ))]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bytes: Vec::with_capacity(capacity) }
    }

    /// Returns the number of bytes the allocation can hold without growing.
    #[cfg_attr(creusot, ensures(
        result@ == creusot_std::std::vec::capacity_model(self.bytes)
    ))]
    #[ensures(result@ >= self@.len())]
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Creates `len` initialized zero bytes.
    #[ensures(result@.len() == len@)]
    #[ensures(forall<i: Int> 0 <= i && i < result@.len() ==> result@[i] == 0u8)]
    pub fn zeroed(len: usize) -> Self {
        let mut bytes = Self::from_vec(Vec::new());
        bytes.resize(len, 0);
        bytes
    }

    /// Returns the owned vector and its byte sequence.
    #[ensures(result@ == self@)]
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns the number of initialized bytes.
    #[ensures(result@ == self@.len())]
    #[check(terminates)]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether the sequence contains no bytes.
    #[ensures(result == (self@.len() == 0))]
    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }

    /// Borrows the complete sequence as an immutable slice.
    #[ensures(result@ == self@)]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Appends an initialized slice using Vec's sequence-preserving extension contract.
    #[ensures((^self)@ == self@.concat(source@))]
    pub(super) fn append_initialized_slice(&mut self, source: &[u8]) {
        self.bytes.extend(source.iter().copied());
    }

    /// Copies a byte at `index`, or returns `None` when it is out of bounds.
    #[ensures(result == if index@ < self@.len() { Some(self@[index@]) } else { None })]
    #[check(terminates)]
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.bytes.len() {
            Some(self.bytes[index])
        } else {
            None
        }
    }

    /// Replaces the byte at `index`, returning whether the index was in range.
    #[ensures(result == (index@ < self@.len()))]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures(forall<i: Int> 0 <= i && i < self@.len() ==>
        (^self)@[i] == if i == index@ { value } else { self@[i] })]
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        if index < self.bytes.len() {
            self.bytes[index] = value;
            true
        } else {
            false
        }
    }

    /// Appends one byte.
    #[ensures((^self)@ == self@.push_back(value))]
    pub fn push(&mut self, value: u8) {
        self.bytes.push(value);
    }

    /// Removes and returns the last byte, if present.
    #[ensures(match result {
        Some(value) => self@ == (^self)@.push_back(value),
        None => self@.len() == 0 && (^self)@ == self@,
    })]
    pub fn pop(&mut self) -> Option<u8> {
        self.bytes.pop()
    }

    /// Keeps the first `len` bytes, or leaves the sequence unchanged if it is shorter.
    #[ensures((^self)@ == self@.subsequence(0,
        if len@ < self@.len() { len@ } else { self@.len() }))]
    pub fn truncate(&mut self, len: usize) {
        let old = snapshot!(self@);
        let target = if len < self.bytes.len() {
            len
        } else {
            self.bytes.len()
        };

        #[invariant(target@ == if len@ < old.len() { len@ } else { old.len() })]
        #[invariant(target@ <= self@.len() && self@.len() <= old.len())]
        #[invariant(self@ == old.subsequence(0, self@.len()))]
        #[variant(self@.len() - target@)]
        while self.bytes.len() > target {
            self.bytes.pop();
        }
    }

    /// Removes all initialized bytes while retaining the allocation.
    #[ensures((^self)@ == self@.subsequence(0, 0))]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Copies the suffix starting at `index`, retaining the prefix in `self`.
    ///
    /// The returned owner has its own Vec allocation. Both owners must be
    /// explicitly closed by their callers.
    #[ensures(match result {
        Some(suffix) => index@ <= self@.len() &&
            (^self)@ == self@.subsequence(0, index@) &&
            suffix@ == self@.subsequence(index@, self@.len()) &&
            (^self)@.concat(suffix@) == self@,
        None => index@ > self@.len() && (^self)@ == self@,
    })]
    pub fn try_split_off_copy(&mut self, index: usize) -> Option<Self> {
        let old = snapshot!(self@);
        if index > self.bytes.len() {
            None
        } else {
            let suffix = Self::copy_from_slice(&self.bytes[index..]);
            self.truncate(index);
            ghost! { split_rejoins(old, snapshot!(index@)); };
            Some(suffix)
        }
    }

    /// Copies the prefix before `index`, retaining the suffix in `self`.
    ///
    /// The returned owner has its own Vec allocation. Both owners must be
    /// explicitly closed by their callers.
    #[ensures(match result {
        Some(prefix) => index@ <= self@.len() &&
            prefix@ == self@.subsequence(0, index@) &&
            (^self)@ == self@.subsequence(index@, self@.len()) &&
            prefix@.concat((^self)@) == self@,
        None => index@ > self@.len() && (^self)@ == self@,
    })]
    pub fn try_split_to_copy(&mut self, index: usize) -> Option<Self> {
        let old = snapshot!(self@);
        if index > self.bytes.len() {
            None
        } else {
            let prefix = Self::copy_from_slice(&self.bytes[..index]);
            let suffix = Self::copy_from_slice(&self.bytes[index..]);
            self.clear();
            self.append_owner(suffix);
            ghost! { split_rejoins(old, snapshot!(index@)); };
            Some(prefix)
        }
    }

    /// Appends another owner and explicitly closes its allocation.
    #[ensures((^self)@ == self@.concat(other@))]
    pub fn append_owner(&mut self, other: Self) {
        self.extend_from_slice(other.as_slice());
        other.close();
    }

    /// Appends `count` copies of `byte`, returning false without mutation on overflow.
    #[ensures(result == (count@ <= usize::MAX@ - self@.len()))]
    #[ensures(if result {
        (^self)@.len() == self@.len() + count@ &&
        forall<i: Int> 0 <= i && i < self@.len() ==> (^self)@[i] == self@[i] &&
        forall<i: Int> 0 <= i && i < count@ ==> (^self)@[self@.len() + i] == byte
    } else {
        (^self)@ == self@
    })]
    pub fn append_repeated(&mut self, byte: u8, count: usize) -> bool {
        match self.bytes.len().checked_add(count) {
            Some(new_len) => {
                self.resize(new_len, byte);
                true
            }
            None => false,
        }
    }

    /// Resizes to `new_len`, filling newly added bytes with `value`.
    #[ensures((^self)@.len() == new_len@)]
    #[ensures(forall<i: Int> 0 <= i && i < self@.len() && i < new_len@ ==>
        (^self)@[i] == self@[i])]
    #[ensures(forall<i: Int> self@.len() <= i && i < new_len@ ==>
        (^self)@[i] == value)]
    pub fn resize(&mut self, new_len: usize, value: u8) {
        let old = snapshot!(self@);
        if new_len <= self.bytes.len() {
            self.truncate(new_len);
        } else {
            #[invariant(old.len() <= self@.len())]
            #[invariant(self@.len() <= new_len@)]
            #[invariant(forall<i: Int> 0 <= i && i < old.len() ==> self@[i] == old[i])]
            #[invariant(forall<i: Int> old.len() <= i && i < self@.len() ==> self@[i] == value)]
            #[variant(new_len@ - self@.len())]
            while self.bytes.len() < new_len {
                self.bytes.push(value);
            }
        }
    }

    /// Requests capacity for at least `len() + additional` bytes.
    #[ensures((^self)@ == self@)]
    #[cfg_attr(creusot, ensures(
        creusot_std::std::vec::capacity_model((^self).bytes) >=
            creusot_std::std::vec::capacity_model(self.bytes)
    ))]
    #[cfg_attr(creusot, ensures(
        creusot_std::std::vec::capacity_model((^self).bytes) >=
            self@.len() + additional@
    ))]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional);
    }

    /// Requests capacity for at least `len() + additional` bytes, minimizing excess capacity.
    #[ensures((^self)@ == self@)]
    #[cfg_attr(creusot, ensures(
        creusot_std::std::vec::capacity_model((^self).bytes) >=
            creusot_std::std::vec::capacity_model(self.bytes)
    ))]
    #[cfg_attr(creusot, ensures(
        creusot_std::std::vec::capacity_model((^self).bytes) >=
            self@.len() + additional@
    ))]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.bytes.reserve_exact(additional);
    }

    /// Tries to reserve capacity for at least `len() + additional` bytes.
    ///
    /// The byte sequence is unchanged whether reservation succeeds or returns
    /// an allocation or capacity-overflow error.
    #[ensures((^self)@ == self@)]
    #[cfg_attr(creusot, ensures(match result {
        Ok(()) =>
            creusot_std::std::vec::capacity_model((^self).bytes) >=
                creusot_std::std::vec::capacity_model(self.bytes) &&
            creusot_std::std::vec::capacity_model((^self).bytes) >=
                self@.len() + additional@,
        Err(_) =>
            creusot_std::std::vec::capacity_model((^self).bytes) ==
                creusot_std::std::vec::capacity_model(self.bytes),
    }))]
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.bytes.try_reserve(additional)
    }

    /// Tries to reserve the minimum capacity for at least `len() + additional` bytes.
    #[ensures((^self)@ == self@)]
    #[cfg_attr(creusot, ensures(match result {
        Ok(()) =>
            creusot_std::std::vec::capacity_model((^self).bytes) >=
                creusot_std::std::vec::capacity_model(self.bytes) &&
            creusot_std::std::vec::capacity_model((^self).bytes) >=
                self@.len() + additional@,
        Err(_) =>
            creusot_std::std::vec::capacity_model((^self).bytes) ==
                creusot_std::std::vec::capacity_model(self.bytes),
    }))]
    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.bytes.try_reserve_exact(additional)
    }

    /// Releases this allocation through the explicit B1 detach and B3 deallocate path.
    ///
    /// The consumed `Vec` is not left for the wrapper's implicit destructor.
    pub fn close(self) {
        let (bound, _len, capacity, capabilities) =
            bound_ptr::detach_bound_vec(self.bytes);
        // SAFETY: B1 returned the descriptor, capacity, and matching full
        // Recovery/PhysicalRegion capabilities for this exact Vec allocation.
        unsafe { raw_vec::deallocate_bound_vec(bound, capacity, capabilities) };
    }
}

/// A basic verified caller for reserve framing followed by explicit cleanup.
///
/// This checks that the sequence is preserved at the reserve boundary and that
/// the observed capacity meets the requested lower bound before closing the owner.
#[cfg(creusot)]
#[ensures(result@ >= owner@.len() + additional@)]
pub(crate) fn reserve_then_close(mut owner: ExclusiveBytes, additional: usize) -> usize {
    let original_contents = snapshot!(owner@);
    owner.reserve(additional);
    proof_assert!(owner@ == *original_contents);

    let capacity = owner.capacity();
    proof_assert!(capacity@ >= original_contents.len() + additional@);
    owner.close();
    capacity
}

/// Splits at `index` and explicitly closes both independently owned results.
#[ensures(result.0 == (index@ <= input@.len()))]
#[ensures(result.1@ == if index@ <= input@.len() { index@ } else { input@.len() })]
#[ensures(result.2@ == if index@ <= input@.len() {
    input@.len() - index@
} else { 0 })]
pub(crate) fn split_off_copy_then_close(
    input: Vec<u8>,
    index: usize,
) -> (bool, usize, usize) {
    let mut owner = ExclusiveBytes::from_vec(input);
    match owner.try_split_off_copy(index) {
        Some(suffix) => {
            let prefix_len = owner.len();
            let suffix_len = suffix.len();
            owner.close();
            suffix.close();
            (true, prefix_len, suffix_len)
        }
        None => {
            let retained_len = owner.len();
            owner.close();
            (false, retained_len, 0)
        }
    }
}

/// Splits at `index` in the opposite direction and explicitly closes both owners.
#[ensures(result.0 == (index@ <= input@.len()))]
#[ensures(result.1@ == if index@ <= input@.len() { index@ } else { 0 })]
#[ensures(result.2@ == if index@ <= input@.len() {
    input@.len() - index@
} else { input@.len() })]
pub(crate) fn split_to_copy_then_close(
    input: Vec<u8>,
    index: usize,
) -> (bool, usize, usize) {
    let mut owner = ExclusiveBytes::from_vec(input);
    match owner.try_split_to_copy(index) {
        Some(prefix) => {
            let prefix_len = prefix.len();
            let suffix_len = owner.len();
            prefix.close();
            owner.close();
            (true, prefix_len, suffix_len)
        }
        None => {
            let retained_len = owner.len();
            owner.close();
            (false, 0, retained_len)
        }
    }
}

/// Exercises the repeated append body and closes the owner on either result.
#[ensures(result == (count@ <= usize::MAX@ - owner@.len()))]
pub(crate) fn append_repeated_then_close(
    mut owner: ExclusiveBytes,
    byte: u8,
    count: usize,
) -> bool {
    let appended = owner.append_repeated(byte, count);
    owner.close();
    appended
}

impl AsRef<[u8]> for ExclusiveBytes {
    #[check(ghost)]
    #[ensures(result@ == self@)]
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Deref for ExclusiveBytes {
    type Target = [u8];

    #[check(ghost)]
    #[ensures(result@ == self@)]
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for ExclusiveBytes {
    #[check(ghost)]
    #[ensures(result@ == self@)]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures((^result)@ == (^self)@)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl AsMut<[u8]> for ExclusiveBytes {
    #[check(ghost)]
    #[ensures(result@ == self@)]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures((^result)@ == (^self)@)]
    fn as_mut(&mut self) -> &mut [u8] {
        DerefMut::deref_mut(self)
    }
}

impl BorrowMut<[u8]> for ExclusiveBytes {
    #[check(ghost)]
    #[ensures(result@ == self@)]
    #[ensures((^self)@.len() == self@.len())]
    #[ensures((^result)@ == (^self)@)]
    fn borrow_mut(&mut self) -> &mut [u8] {
        <Self as AsMut<[u8]>>::as_mut(self)
    }
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::ExclusiveBytes;
    use std::{borrow::BorrowMut, convert::AsMut, ops::{Deref, DerefMut}, vec, vec::Vec};

    #[test]
    fn mutable_slice_traits_preserve_owner_sequence_and_close() {
        let mut bytes = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        AsMut::<[u8]>::as_mut(&mut bytes)[0] = 10;
        BorrowMut::<[u8]>::borrow_mut(&mut bytes)[2] = 30;
        assert_eq!(bytes.as_slice(), &[10, 2, 30]);
        bytes.close();
    }

    #[test]
    fn exclusive_sequence_operations() {
        let mut bytes = ExclusiveBytes::from_vec(vec![10, 20, 30]);
        assert_eq!(bytes.len(), 3);
        assert!(!bytes.is_empty());
        assert_eq!(bytes.as_slice(), &[10, 20, 30]);
        assert_eq!(bytes.get(1), Some(20));
        assert_eq!(bytes.get(3), None);
        assert_eq!(AsRef::<[u8]>::as_ref(&bytes), &[10, 20, 30]);
        assert_eq!(Deref::deref(&bytes), &[10, 20, 30]);

        DerefMut::deref_mut(&mut bytes)[1] = 22;
        assert_eq!(bytes.as_slice(), &[10, 22, 30]);

        assert!(bytes.set(1, 21));
        assert!(!bytes.set(3, 99));
        assert_eq!(bytes.as_slice(), &[10, 21, 30]);

        bytes.push(40);
        assert_eq!(bytes.pop(), Some(40));
        assert_eq!(bytes.pop(), Some(30));
        assert_eq!(bytes.pop(), Some(21));
        assert_eq!(bytes.pop(), Some(10));
        assert_eq!(bytes.pop(), None);
        assert!(bytes.is_empty());

        bytes.push(1);
        bytes.push(2);
        bytes.push(3);
        bytes.truncate(2);
        assert_eq!(bytes.as_slice(), &[1, 2]);
        bytes.truncate(8);
        assert_eq!(bytes.as_slice(), &[1, 2]);

        bytes.resize(5, 7);
        assert_eq!(bytes.as_slice(), &[1, 2, 7, 7, 7]);
        bytes.resize(1, 9);
        assert_eq!(bytes.as_slice(), &[1]);

        bytes.reserve(12);
        bytes.reserve_exact(4);
        assert_eq!(bytes.into_vec(), vec![1]);
    }

    #[test]
    fn try_reserve_overflow_preserves_contents_and_capacity() {
        let mut bytes = ExclusiveBytes::from_vec(vec![17]);
        let old_capacity = bytes.capacity();
        let result = bytes.try_reserve(usize::MAX);
        assert!(result.is_err());
        assert_eq!(bytes.as_slice(), &[17]);
        assert_eq!(bytes.capacity(), old_capacity);
        bytes.close();
    }

    #[test]
    fn try_reserve_exact_overflow_preserves_contents_and_capacity() {
        let mut bytes = ExclusiveBytes::from_vec(vec![23]);
        let old_capacity = bytes.capacity();
        let result = bytes.try_reserve_exact(usize::MAX);
        assert!(result.is_err());
        assert_eq!(bytes.as_slice(), &[23]);
        assert_eq!(bytes.capacity(), old_capacity);
        bytes.close();
    }

    #[test]
    fn zero_and_spare_capacity_reservations_preserve_capacity() {
        let mut empty = ExclusiveBytes::with_capacity(0);
        let empty_capacity = empty.capacity();
        assert_eq!(empty.try_reserve(0), Ok(()));
        assert_eq!(empty.capacity(), empty_capacity);
        empty.close();

        let mut spare = ExclusiveBytes::from_vec(Vec::with_capacity(8));
        spare.push(31);
        let spare_capacity = spare.capacity();
        assert_eq!(spare.try_reserve_exact(0), Ok(()));
        assert_eq!(spare.capacity(), spare_capacity);
        assert_eq!(spare.as_slice(), &[31]);
        spare.close();
    }

    #[test]
    fn successful_reservations_preserve_contents_and_meet_lower_bound() {
        let original = vec![3, 5, 8, 13];
        let additional = 32;

        let mut amortized = ExclusiveBytes::from_vec(original.clone());
        assert_eq!(amortized.try_reserve(additional), Ok(()));
        assert_eq!(amortized.as_slice(), original.as_slice());
        assert!(amortized.capacity() >= original.len() + additional);
        amortized.close();

        let mut exact = ExclusiveBytes::from_vec(original.clone());
        assert_eq!(exact.try_reserve_exact(additional), Ok(()));
        assert_eq!(exact.as_slice(), original.as_slice());
        assert!(exact.capacity() >= original.len() + additional);
        exact.close();
    }

    #[test]
    fn reserve_then_close_preserves_contents_and_capacity_lower_bound() {
        let original = vec![2, 4, 6, 8];
        let additional = 24;
        let mut owner = ExclusiveBytes::from_vec(original.clone());
        owner.reserve(additional);
        assert_eq!(owner.as_slice(), original.as_slice());
        let capacity = owner.capacity();
        assert!(capacity >= original.len() + additional);
        owner.close();
    }

    #[test]
    fn explicit_close_releases_empty_and_nonempty_allocations() {
        ExclusiveBytes::from_vec(Vec::new()).close();
        ExclusiveBytes::from_vec(Vec::with_capacity(8)).close();
        ExclusiveBytes::from_vec(vec![1, 2, 3]).close();
    }

    #[test]
    fn closed_convenience_mutations_preserve_exact_sequences() {
        let mut bytes = ExclusiveBytes::from_vec(vec![1, 2]);
        bytes.clear();
        assert_eq!(bytes.as_slice(), &[]);
        bytes.append_owner(ExclusiveBytes::from_vec(vec![3, 4]));
        assert_eq!(bytes.as_slice(), &[3, 4]);
        bytes.close();

        let zeroes = ExclusiveBytes::zeroed(5);
        assert_eq!(zeroes.as_slice(), &[0, 0, 0, 0, 0]);
        zeroes.close();
        ExclusiveBytes::zeroed(0).close();
    }

    #[test]
    fn checked_copy_splits_preserve_sequences_and_close_both_owners() {
        let mut empty_with_spare = ExclusiveBytes::from_vec(Vec::with_capacity(8));
        let empty_suffix = empty_with_spare.try_split_off_copy(0).unwrap();
        assert_eq!(empty_with_spare.as_slice(), &[]);
        assert_eq!(empty_suffix.as_slice(), &[]);
        empty_suffix.close();
        assert!(empty_with_spare.try_split_to_copy(1).is_none());
        assert_eq!(empty_with_spare.as_slice(), &[]);

        empty_with_spare.extend_from_slice(&[10, 20, 30, 40]);
        let prefix = empty_with_spare.try_split_to_copy(2).unwrap();
        assert_eq!(prefix.as_slice(), &[10, 20]);
        assert_eq!(empty_with_spare.as_slice(), &[30, 40]);
        prefix.close();
        empty_with_spare.close();

        let mut at_zero = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        let all_bytes = at_zero.try_split_off_copy(0).unwrap();
        assert_eq!(at_zero.as_slice(), &[]);
        assert_eq!(all_bytes.as_slice(), &[1, 2, 3]);
        all_bytes.close();
        at_zero.close();

        let mut split_off_middle = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        let suffix = split_off_middle.try_split_off_copy(2).unwrap();
        assert_eq!(split_off_middle.as_slice(), &[1, 2]);
        assert_eq!(suffix.as_slice(), &[3]);
        suffix.close();
        split_off_middle.close();

        let mut at_end = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        let empty = at_end.try_split_off_copy(3).unwrap();
        assert_eq!(at_end.as_slice(), &[1, 2, 3]);
        assert_eq!(empty.as_slice(), &[]);
        empty.close();
        at_end.close();

        let mut split_to_zero = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        let empty = split_to_zero.try_split_to_copy(0).unwrap();
        assert_eq!(empty.as_slice(), &[]);
        assert_eq!(split_to_zero.as_slice(), &[1, 2, 3]);
        empty.close();
        split_to_zero.close();

        let mut split_to_end = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        let all_bytes = split_to_end.try_split_to_copy(3).unwrap();
        assert_eq!(all_bytes.as_slice(), &[1, 2, 3]);
        assert_eq!(split_to_end.as_slice(), &[]);
        all_bytes.close();
        split_to_end.close();

        let mut invalid_off = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        assert!(invalid_off.try_split_off_copy(4).is_none());
        assert_eq!(invalid_off.as_slice(), &[1, 2, 3]);
        invalid_off.close();

        let mut invalid_to = ExclusiveBytes::from_vec(vec![1, 2, 3]);
        assert!(invalid_to.try_split_to_copy(4).is_none());
        assert_eq!(invalid_to.as_slice(), &[1, 2, 3]);
        invalid_to.close();
    }

    #[test]
    fn split_cleanup_callers_cover_empty_boundaries_and_invalid_indices() {
        assert_eq!(
            super::split_off_copy_then_close(vec![1, 2, 3], 0),
            (true, 0, 3),
        );
        assert_eq!(
            super::split_off_copy_then_close(vec![1, 2, 3], 3),
            (true, 3, 0),
        );
        assert_eq!(
            super::split_off_copy_then_close(vec![], 0),
            (true, 0, 0),
        );
        assert_eq!(
            super::split_off_copy_then_close(vec![1, 2, 3], 4),
            (false, 3, 0),
        );
        assert_eq!(
            super::split_to_copy_then_close(vec![1, 2, 3], 0),
            (true, 0, 3),
        );
        assert_eq!(
            super::split_to_copy_then_close(vec![1, 2, 3], 3),
            (true, 3, 0),
        );
        assert_eq!(
            super::split_to_copy_then_close(vec![], 0),
            (true, 0, 0),
        );
        assert_eq!(
            super::split_to_copy_then_close(vec![1, 2, 3], 4),
            (false, 0, 3),
        );
    }

    #[test]
    fn repeated_append_handles_zero_success_and_checked_overflow() {
        let mut bytes = ExclusiveBytes::from_vec(vec![4, 5]);
        assert!(bytes.append_repeated(9, 0));
        assert_eq!(bytes.as_slice(), &[4, 5]);
        assert!(bytes.append_repeated(9, 3));
        assert_eq!(bytes.as_slice(), &[4, 5, 9, 9, 9]);
        bytes.close();

        assert!(!super::append_repeated_then_close(
            ExclusiveBytes::from_vec(vec![7]),
            0xaa,
            usize::MAX,
        ));
    }
}
