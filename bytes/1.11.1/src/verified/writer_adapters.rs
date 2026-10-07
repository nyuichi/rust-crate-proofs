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

#[cfg(all(test, not(creusot), feature = "std"))]
mod tests {
    use super::{LimitedWriter, ExclusiveBytes};
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
}
