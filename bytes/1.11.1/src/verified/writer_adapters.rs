//! Bounded writers over concrete initialized byte owners.

use super::exclusive::ExclusiveBytes;
use creusot_std::prelude::*;

/// Appends at most `limit` bytes to one exclusively borrowed owner.
pub struct LimitedWriter<'a> {
    owner: &'a mut ExclusiveBytes,
    remaining: usize,
}

impl<'a> LimitedWriter<'a> {
    #[logic]
    pub fn owner_view(self) -> Seq<u8> {
        pearlite! { (*self.owner)@ }
    }

    #[logic(prophetic)]
    pub fn owner_final_view(self) -> Seq<u8> {
        pearlite! { (^self.owner)@ }
    }

    #[logic]
    pub fn remaining_view(self) -> Int {
        pearlite! { self.remaining@ }
    }

    /// Borrows `owner` and creates a writer with the given append budget.
    #[ensures(result.owner_view() == owner@)]
    #[ensures(result.owner_final_view() == (^owner)@)]
    #[ensures(result.remaining_view() == limit@)]
    pub fn new(owner: &'a mut ExclusiveBytes, limit: usize) -> Self {
        Self { owner, remaining: limit }
    }

    /// Returns the number of bytes this writer may still append.
    #[ensures(result@ == self.remaining_view())]
    pub fn remaining(&self) -> usize {
        self.remaining
    }

    /// Appends `source` only when it fits both the budget and the owner's length domain.
    /// Failure leaves the owner's bytes and the remaining budget unchanged.
    #[ensures(result == (
        source@.len() <= self.remaining_view() &&
        source@.len() <= usize::MAX@ - self.owner_view().len()
    ))]
    #[ensures(if result {
        (^self).owner_view() == self.owner_view().concat(source@) &&
        (^self).remaining_view() == self.remaining_view() - source@.len()
    } else {
        (^self).owner_view() == self.owner_view() &&
        (^self).remaining_view() == self.remaining_view()
    })]
    #[ensures((^self).owner_final_view() == self.owner_final_view())]
    pub fn write_slice(&mut self, source: &[u8]) -> bool {
        let count = source.len();
        let available_length = usize::MAX - self.owner.len();
        if count > self.remaining || count > available_length {
            false
        } else {
            self.owner.extend_from_slice(source);
            self.remaining -= count;
            true
        }
    }

    /// Returns the borrowed owner with all successful appends applied.
    #[ensures(result@ == self.owner_view())]
    #[ensures((^result)@ == self.owner_final_view())]
    pub fn into_inner(self) -> &'a mut ExclusiveBytes {
        self.owner
    }
}

/// Writes through a limited adapter and returns the owner's final mutable borrow.
#[ensures(result == (
    source@.len() <= limit@ && source@.len() <= usize::MAX@ - owner@.len()
))]
#[ensures((^owner)@ == if result { owner@.concat(source@) } else { owner@ })]
pub fn append_through_limited_writer(
    owner: &mut ExclusiveBytes,
    source: &[u8],
    limit: usize,
) -> bool {
    #[cfg(creusot)]
    let original = snapshot!(owner@);
    #[cfg(creusot)]
    let source_view = snapshot!(source@);
    let mut writer = LimitedWriter::new(owner, limit);
    let written = writer.write_slice(source);
    proof_assert!(writer.owner_view() == if written {
        (*original).concat(*source_view)
    } else {
        *original
    });
    let returned_owner = writer.into_inner();
    proof_assert!(returned_owner@ == if written {
        (*original).concat(*source_view)
    } else {
        *original
    });
    written
}

/// Exercises construction, append, borrow release, observation, and explicit close.
#[ensures(result == (
    source@.len() <= limit@ && source@.len() <= usize::MAX@ - owner@.len()
))]
pub fn limited_write_then_close(
    mut owner: ExclusiveBytes,
    source: &[u8],
    limit: usize,
) -> bool {
    #[cfg(creusot)]
    let original = snapshot!(owner@);
    #[cfg(creusot)]
    let source_view = snapshot!(source@);
    let written = append_through_limited_writer(&mut owner, source, limit);
    proof_assert!(owner@ == if written {
        (*original).concat(*source_view)
    } else {
        *original
    });
    owner.close();
    written
}

#[logic]
fn left_chunk_len(source_len: Int, left_budget: Int) -> Int {
    pearlite! { if source_len < left_budget { source_len } else { left_budget } }
}

/// Appends initialized bytes across two separately owned buffers with fixed budgets.
pub struct ChainedWriter<'a> {
    left: &'a mut ExclusiveBytes,
    right: &'a mut ExclusiveBytes,
    left_remaining: usize,
    right_remaining: usize,
}

impl<'a> ChainedWriter<'a> {
    #[logic]
    pub fn left_view(self) -> Seq<u8> {
        pearlite! { (*self.left)@ }
    }

    #[logic]
    pub fn right_view(self) -> Seq<u8> {
        pearlite! { (*self.right)@ }
    }

    #[logic(prophetic)]
    pub fn left_final_view(self) -> Seq<u8> {
        pearlite! { (^self.left)@ }
    }

    #[logic(prophetic)]
    pub fn right_final_view(self) -> Seq<u8> {
        pearlite! { (^self.right)@ }
    }

    #[logic]
    pub fn left_remaining_view(self) -> Int {
        pearlite! { self.left_remaining@ }
    }

    #[logic]
    pub fn right_remaining_view(self) -> Int {
        pearlite! { self.right_remaining@ }
    }

    /// Borrows both owners and records their append budgets.
    #[ensures(result.left_view() == left@)]
    #[ensures(result.right_view() == right@)]
    #[ensures(result.left_final_view() == (^left)@)]
    #[ensures(result.right_final_view() == (^right)@)]
    #[ensures(result.left_remaining_view() == left_limit@)]
    #[ensures(result.right_remaining_view() == right_limit@)]
    pub fn new(
        left: &'a mut ExclusiveBytes,
        right: &'a mut ExclusiveBytes,
        left_limit: usize,
        right_limit: usize,
    ) -> Self {
        Self {
            left,
            right,
            left_remaining: left_limit,
            right_remaining: right_limit,
        }
    }

    /// Writes a source into the left owner first, then the right owner. Failure changes nothing.
    #[ensures(result == (
        source@.len() - left_chunk_len(source@.len(), self.left_remaining_view())
            <= self.right_remaining_view() &&
        left_chunk_len(source@.len(), self.left_remaining_view())
            <= usize::MAX@ - self.left_view().len() &&
        source@.len() - left_chunk_len(source@.len(), self.left_remaining_view())
            <= usize::MAX@ - self.right_view().len()
    ))]
    #[ensures(if result {
        (^self).left_view() == self.left_view().concat(
            source@[0..left_chunk_len(source@.len(), self.left_remaining_view())]) &&
        (^self).right_view() == self.right_view().concat(
            source@[left_chunk_len(source@.len(), self.left_remaining_view())..]) &&
        (^self).left_remaining_view() == self.left_remaining_view()
            - left_chunk_len(source@.len(), self.left_remaining_view()) &&
        (^self).right_remaining_view() == self.right_remaining_view()
            - (source@.len() - left_chunk_len(source@.len(), self.left_remaining_view()))
    } else {
        (^self).left_view() == self.left_view() &&
        (^self).right_view() == self.right_view() &&
        (^self).left_remaining_view() == self.left_remaining_view() &&
        (^self).right_remaining_view() == self.right_remaining_view()
    })]
    #[ensures((^self).left_final_view() == self.left_final_view())]
    #[ensures((^self).right_final_view() == self.right_final_view())]
    pub fn write_slice(&mut self, source: &[u8]) -> bool {
        let count = source.len();
        let left_count = if count < self.left_remaining { count } else { self.left_remaining };
        let right_count = count - left_count;
        let left_available_length = usize::MAX - self.left.len();
        let right_available_length = usize::MAX - self.right.len();
        if right_count > self.right_remaining
            || left_count > left_available_length
            || right_count > right_available_length
        {
            false
        } else {
            let left_part = &source[..left_count];
            let right_part = &source[left_count..];
            self.left.append_initialized_slice(left_part);
            self.right.append_initialized_slice(right_part);
            self.left_remaining -= left_count;
            self.right_remaining -= right_count;
            true
        }
    }

    /// Returns both borrowed owners after all successful writes.
    #[ensures(result.0@ == self.left_view())]
    #[ensures(result.1@ == self.right_view())]
    #[ensures((^result.0)@ == self.left_final_view())]
    #[ensures((^result.1)@ == self.right_final_view())]
    pub fn into_inner(self) -> (&'a mut ExclusiveBytes, &'a mut ExclusiveBytes) {
        (self.left, self.right)
    }
}

/// Writes across two owners, releases both borrows, and exposes their final byte sequences.
#[ensures(result == (
    source@.len() - left_chunk_len(source@.len(), left_limit@)
        <= right_limit@ &&
    left_chunk_len(source@.len(), left_limit@) <= usize::MAX@ - left@.len() &&
    source@.len() - left_chunk_len(source@.len(), left_limit@)
        <= usize::MAX@ - right@.len()
))]
#[ensures((^left)@ == if result {
    left@.concat(source@[0..left_chunk_len(source@.len(), left_limit@)])
} else { left@ })]
#[ensures((^right)@ == if result {
    right@.concat(source@[left_chunk_len(source@.len(), left_limit@)..])
} else { right@ })]
pub fn append_through_chained_writer(
    left: &mut ExclusiveBytes,
    right: &mut ExclusiveBytes,
    source: &[u8],
    left_limit: usize,
    right_limit: usize,
) -> bool {
    #[cfg(creusot)]
    let old_left = snapshot!(left@);
    #[cfg(creusot)]
    let old_right = snapshot!(right@);
    #[cfg(creusot)]
    let source_view = snapshot!(source@);
    let mut writer = ChainedWriter::new(left, right, left_limit, right_limit);
    let written = writer.write_slice(source);
    proof_assert!(writer.left_view() == if written {
        (*old_left).concat((*source_view)[0..left_chunk_len(source@.len(), left_limit@)])
    } else {
        *old_left
    });
    proof_assert!(writer.right_view() == if written {
        (*old_right).concat((*source_view)[left_chunk_len(source@.len(), left_limit@)..])
    } else {
        *old_right
    });
    let (returned_left, returned_right) = writer.into_inner();
    proof_assert!(returned_left@ == if written {
        (*old_left).concat((*source_view)[0..left_chunk_len(source@.len(), left_limit@)])
    } else {
        *old_left
    });
    proof_assert!(returned_right@ == if written {
        (*old_right).concat((*source_view)[left_chunk_len(source@.len(), left_limit@)..])
    } else {
        *old_right
    });
    written
}

/// Exercises the chained write, releases both borrows, observes both owners, and closes them.
#[ensures(result == (
    source@.len() - left_chunk_len(source@.len(), left_limit@)
        <= right_limit@ &&
    left_chunk_len(source@.len(), left_limit@) <= usize::MAX@ - left@.len() &&
    source@.len() - left_chunk_len(source@.len(), left_limit@)
        <= usize::MAX@ - right@.len()
))]
pub fn chained_write_then_close(
    mut left: ExclusiveBytes,
    mut right: ExclusiveBytes,
    source: &[u8],
    left_limit: usize,
    right_limit: usize,
) -> bool {
    #[cfg(creusot)]
    let old_left = snapshot!(left@);
    #[cfg(creusot)]
    let old_right = snapshot!(right@);
    #[cfg(creusot)]
    let source_view = snapshot!(source@);
    let written = append_through_chained_writer(
        &mut left,
        &mut right,
        source,
        left_limit,
        right_limit,
    );
    proof_assert!(left@ == if written {
        (*old_left).concat((*source_view)[0..left_chunk_len(source@.len(), left_limit@)])
    } else {
        *old_left
    });
    proof_assert!(right@ == if written {
        (*old_right).concat((*source_view)[left_chunk_len(source@.len(), left_limit@)..])
    } else {
        *old_right
    });
    left.close();
    right.close();
    written
}

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::{ChainedWriter, ExclusiveBytes, LimitedWriter};
    use alloc::vec;

    #[test]
    fn limited_writer_accepts_empty_write_with_zero_budget() {
        let mut owner = ExclusiveBytes::from_vec(vec![1, 2]);
        {
            let mut writer = LimitedWriter::new(&mut owner, 0);
            assert!(writer.write_slice(&[]));
            assert!(!writer.write_slice(&[3]));
            assert_eq!(writer.remaining(), 0);
            let returned_owner = writer.into_inner();
            assert_eq!(returned_owner.as_slice(), &[1, 2]);
        }
        owner.close();
    }

    #[test]
    fn limited_writer_accepts_exact_budget_and_rejects_later_write() {
        let mut owner = ExclusiveBytes::from_vec(vec![1, 2]);
        {
            let mut writer = LimitedWriter::new(&mut owner, 2);
            assert!(writer.write_slice(&[3, 4]));
            assert_eq!(writer.remaining(), 0);
            assert!(!writer.write_slice(&[5]));
            assert_eq!(writer.remaining(), 0);
            let returned_owner = writer.into_inner();
            assert_eq!(returned_owner.as_slice(), &[1, 2, 3, 4]);
        }
        owner.close();
    }

    #[test]
    fn limited_writer_rejects_oversized_write_without_mutation() {
        let mut owner = ExclusiveBytes::from_vec(vec![1, 2]);
        {
            let mut writer = LimitedWriter::new(&mut owner, 1);
            assert!(!writer.write_slice(&[3, 4]));
            assert_eq!(writer.remaining(), 1);
            let returned_owner = writer.into_inner();
            assert_eq!(returned_owner.as_slice(), &[1, 2]);
        }
        owner.close();
    }

    #[test]
    fn chained_writer_splits_across_left_then_right() {
        let mut left = ExclusiveBytes::from_vec(vec![1]);
        let mut right = ExclusiveBytes::from_vec(vec![9]);
        {
            let mut writer = ChainedWriter::new(&mut left, &mut right, 2, 2);
            assert!(writer.write_slice(&[2, 3, 4]));
            let (returned_left, returned_right) = writer.into_inner();
            assert_eq!(returned_left.as_slice(), &[1, 2, 3]);
            assert_eq!(returned_right.as_slice(), &[9, 4]);
        }
        left.close();
        right.close();
    }

    #[test]
    fn chained_writer_rejects_insufficient_total_budget_without_mutation() {
        let mut left = ExclusiveBytes::from_vec(vec![1]);
        let mut right = ExclusiveBytes::from_vec(vec![9]);
        {
            let mut writer = ChainedWriter::new(&mut left, &mut right, 1, 1);
            assert!(!writer.write_slice(&[2, 3, 4]));
            let (returned_left, returned_right) = writer.into_inner();
            assert_eq!(returned_left.as_slice(), &[1]);
            assert_eq!(returned_right.as_slice(), &[9]);
        }
        left.close();
        right.close();
    }

    #[test]
    fn chained_writer_accepts_empty_write_with_zero_budgets() {
        let mut left = ExclusiveBytes::from_vec(vec![1]);
        let mut right = ExclusiveBytes::from_vec(vec![9]);
        {
            let mut writer = ChainedWriter::new(&mut left, &mut right, 0, 0);
            assert!(writer.write_slice(&[]));
            let (returned_left, returned_right) = writer.into_inner();
            assert_eq!(returned_left.as_slice(), &[1]);
            assert_eq!(returned_right.as_slice(), &[9]);
        }
        left.close();
        right.close();
    }
}
