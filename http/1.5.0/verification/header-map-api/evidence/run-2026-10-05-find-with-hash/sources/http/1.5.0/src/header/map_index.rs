use core::fmt::{self, Debug, Formatter};
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel, Int};

/// Type used for representing the size of a HeaderMap value.
///
/// 32,768 entries fit below the sentinel used by `Pos`.
pub(crate) type Size = u16;

/// This limit falls out from above.
pub(crate) const MAX_SIZE: usize = 1 << 15;

/// Hash values are limited to u16 as well. The index table never grows beyond
/// 32,768 slots, so the lower bits suffice for its slot selection.
#[derive(Copy, Eq)]
pub(crate) struct HashValue(pub u16);

impl Debug for HashValue {
    #[cfg_attr(creusot, ensures(creusot_std::std::fmt::formatter_extends(
        formatter.deep_model(), (^formatter).deep_model()
    )))]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("HashValue").field(&self.0).finish()
    }
}

impl Clone for HashValue {
    #[ensures(result.0@ == self.0@)]
    fn clone(&self) -> Self {
        HashValue(self.0)
    }
}

impl DeepModel for HashValue {
    type DeepModelTy = Int;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self.0@ }
    }
}

impl PartialEq for HashValue {
    #[ensures(result == (self.0@ == other.0@))]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// An entry in the hash table. This represents the full hash code for an entry
/// as well as the position of the entry in the `entries` vector.
#[derive(Copy)]
pub(crate) struct Pos {
    // Index in the `entries` vec. `u16::MAX` is the empty-slot sentinel.
    pub index: Size,
    // Full hash value for the entry.
    pub hash: HashValue,
}

impl Clone for Pos {
    #[ensures(result.index == self.index && result.hash == self.hash)]
    fn clone(&self) -> Self {
        Pos {
            index: self.index,
            hash: self.hash,
        }
    }
}

impl Pos {
    #[inline]
    #[requires(index@ < MAX_SIZE@)]
    #[ensures(result.index@ == index@)]
    #[ensures(result.hash == hash)]
    pub(crate) fn new(index: usize, hash: HashValue) -> Self {
        debug_assert!(index < MAX_SIZE);
        Pos {
            index: index as Size,
            hash,
        }
    }

    #[inline]
    #[ensures(result.index == u16::MAX && result.hash.0 == 0u16)]
    pub(crate) fn none() -> Self {
        Pos {
            index: u16::MAX,
            hash: HashValue(0),
        }
    }

    #[inline]
    #[ensures(result == (self.index != u16::MAX))]
    pub(crate) fn is_some(&self) -> bool {
        !self.is_none()
    }

    #[inline]
    #[ensures(result == (self.index == u16::MAX))]
    pub(crate) fn is_none(&self) -> bool {
        self.index == u16::MAX
    }

    #[inline]
    #[ensures(match result {
        Some((index, hash)) => self.index != u16::MAX
            && index@ == self.index@ && hash == self.hash,
        None => self.index == u16::MAX,
    })]
    pub(crate) fn resolve(&self) -> Option<(usize, HashValue)> {
        if self.is_some() {
            Some((self.index as usize, self.hash))
        } else {
            None
        }
    }
}

#[inline]
#[cfg_attr(creusot, creusot_std::prelude::bitwise_proof)]
#[ensures(result == (hash.0 & mask) as usize)]
#[ensures(result <= mask as usize)]
pub(crate) fn desired_pos(mask: Size, hash: HashValue) -> usize {
    (hash.0 & mask) as usize
}

/// The number of steps that `current` is forward of the desired position for hash.
#[inline]
#[requires(exists<k: Int> k >= 0 && mask@ + 1 == 2.pow(k))]
#[requires(mask@ < MAX_SIZE@)]
#[requires(current@ <= mask@)]
#[requires(current@ + mask@ + 1 <= usize::MAX@)]
#[ensures(result@ == if current@ >= (hash.0 & mask)@ {
    current@ - (hash.0 & mask)@
} else {
    current@ + mask@ + 1 - (hash.0 & mask)@
})]
pub(crate) fn probe_distance(mask: Size, hash: HashValue, current: usize) -> usize {
    let desired = desired_pos(mask, hash);
    if current >= desired {
        current - desired
    } else {
        current + mask as usize + 1 - desired
    }
}
