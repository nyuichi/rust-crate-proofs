//! Exclusive, Vec-backed byte storage for the modified verified API.
//!
//! This type owns one ordinary `Vec<u8>` and does not expose shared aliases.
//! Call [`ExclusiveBytes::close`] to release its allocation through the
//! explicit B1/B3 ownership path.

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use creusot_std::prelude::*;

use crate::ownership_proof::{bound_ptr, raw_vec};

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
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { bytes }
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

    /// Appends another owner and explicitly closes its allocation.
    #[ensures((^self)@ == self@.concat(other@))]
    pub fn append_owner(&mut self, other: Self) {
        self.extend_from_slice(other.as_slice());
        other.close();
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
    ///
    /// The current formal postcondition specifies byte-sequence preservation;
    /// the capacity guarantee follows the native `Vec::reserve` behavior and
    /// is outside this leaf's current formal contract.
    #[ensures((^self)@ == self@)]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional);
    }

    /// Requests capacity for at least `len() + additional` bytes, minimizing excess capacity.
    ///
    /// The current formal postcondition specifies byte-sequence preservation;
    /// the capacity guarantee follows the native `Vec::reserve_exact` behavior
    /// and is outside this leaf's current formal contract.
    #[ensures((^self)@ == self@)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.bytes.reserve_exact(additional);
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

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::ExclusiveBytes;
    use std::{ops::{Deref, DerefMut}, vec, vec::Vec};

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
}
