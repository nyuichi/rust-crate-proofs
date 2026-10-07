//! Borrowed and explicitly owned byte iteration.

use core::slice;
use creusot_std::prelude::*;

use super::exclusive::ExclusiveBytes;

impl ExclusiveBytes {
    /// Borrows the bytes through the standard slice iterator contract.
    #[ensures(result@@ == self@)]
    pub fn iter(&self) -> slice::Iter<'_, u8> {
        self.as_slice().iter()
    }
}

/// Copies through the borrowed iterator and explicitly releases the source.
#[ensures(result@ == source@)]
pub fn copy_iter_then_close(source: ExclusiveBytes) -> ExclusiveBytes {
    let copy = ExclusiveBytes::from_iter(source.iter().copied());
    source.close();
    copy
}

/// Proves the concrete sequence identity needed to advance an owned cursor.
#[check(ghost)]
#[requires(0 <= *start && *start < *end && *end <= (*source).len())]
#[ensures((*source).subsequence(*start, *end)
    .subsequence(1, (*source).subsequence(*start, *end).len())
    == (*source).subsequence(*start + 1, *end))]
fn sequence_drop_first(source: Snapshot<Seq<u8>>, start: Snapshot<Int>, end: Snapshot<Int>) {
    let range = snapshot!((*source).subsequence(*start, *end));
    let shifted = snapshot!((*source).subsequence(*start + 1, *end));
    let dropped = snapshot!((*source)
        .subsequence(*start, *end)
        .subsequence(1, (*source).subsequence(*start, *end).len()));

    proof_assert!((*range).len() == *end - *start);
    proof_assert!((*shifted).len() == (*range).len() - 1);
    proof_assert!((*dropped).len() == (*shifted).len());
    proof_assert!(forall<i: Int>
        0 <= i && i < (*shifted).len() ==>
            (*dropped)[i] == (*shifted)[i]);
    proof_assert!((*dropped).ext_eq(*shifted));
}

/// An owned cursor that keeps its byte allocation until `close` is called.
///
/// This cursor intentionally does not implement `Iterator` or `IntoIterator`:
/// generic iterator consumers can drop an exhausted iterator, which would
/// release the backing vector outside the explicit close path.
pub struct OwnedByteCursor {
    owner: ExclusiveBytes,
    index: usize,
}

impl Invariant for OwnedByteCursor {
    #[logic(open(self))]
    fn invariant(self) -> bool {
        pearlite! { self.index@ <= self.owner@.len() }
    }
}

impl View for OwnedByteCursor {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.owner@.subsequence(self.index@, self.owner@.len()) }
    }
}

impl OwnedByteCursor {
    /// Starts iteration at the first byte while retaining the owner.
    #[ensures(result@ == owner@)]
    pub fn new(owner: ExclusiveBytes) -> Self {
        Self { owner, index: 0 }
    }

    /// Returns the next byte, leaving the cursor unchanged at exhaustion.
    #[ensures(result == if self@.len() > 0 {
        Some(self@[0])
    } else {
        None
    })]
    #[ensures((^self).owner@ == self.owner@)]
    #[ensures((^self).index@ == if self.index@ < self.owner@.len() {
        self.index@ + 1
    } else {
        self.index@
    })]
    #[ensures((^self)@ == if self@.len() == 0 {
        self@
    } else {
        self@.subsequence(1, self@.len())
    })]
    pub fn next_byte(&mut self) -> Option<u8> {
        let owner = snapshot!(self.owner@);
        let index = snapshot!(self.index@);
        let end = snapshot!(self.owner@.len());
        if self.index < self.owner.len() {
            sequence_drop_first(owner, index, end);
        }

        match self.owner.get(self.index) {
            Some(byte) => {
                self.index += 1;
                Some(byte)
            }
            None => None,
        }
    }

    /// Returns the number of bytes that remain to be read.
    #[ensures(result@ == self@.len())]
    pub fn remaining_len(&self) -> usize {
        self.owner.len() - self.index
    }

    /// Releases the backing allocation through `ExclusiveBytes::close`.
    pub fn close(self) {
        self.owner.close();
    }
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn borrowed_iteration_copies_then_closes_the_source() {
        let source = ExclusiveBytes::copy_from_slice(b"iterate");
        let copy = copy_iter_then_close(source);
        assert_eq!(copy.as_slice(), b"iterate");
        copy.close();
    }

    #[test]
    fn owned_cursor_preserves_owner_until_explicit_close() {
        let owner = ExclusiveBytes::copy_from_slice(b"ab");
        let mut cursor = OwnedByteCursor::new(owner);
        assert_eq!(cursor.remaining_len(), 2);
        assert_eq!(cursor.next_byte(), Some(b'a'));
        assert_eq!(cursor.remaining_len(), 1);
        assert_eq!(cursor.next_byte(), Some(b'b'));
        assert_eq!(cursor.next_byte(), None);
        assert_eq!(cursor.remaining_len(), 0);
        cursor.close();
    }

    #[test]
    fn empty_owned_cursor_stays_exhausted() {
        let owner = ExclusiveBytes::copy_from_slice(b"");
        let mut cursor = OwnedByteCursor::new(owner);
        assert_eq!(cursor.next_byte(), None);
        assert_eq!(cursor.next_byte(), None);
        assert_eq!(cursor.remaining_len(), 0);
        cursor.close();
    }
}
