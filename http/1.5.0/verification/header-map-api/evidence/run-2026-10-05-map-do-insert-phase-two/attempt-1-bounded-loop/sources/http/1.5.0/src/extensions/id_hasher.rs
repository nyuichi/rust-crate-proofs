use creusot_std::prelude::*;
use std::hash::Hasher;

/// The integer hasher used for `TypeId` values in `Extensions`.
pub struct IdHasher(
    /// The last value supplied to [`Hasher::write_u64`].
    pub u64,
);

impl Default for IdHasher {
    #[ensures(result.0@ == 0)]
    fn default() -> Self {
        Self(0)
    }
}

impl Clone for IdHasher {
    #[ensures(result.0@ == self.0@)]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    #[ensures((^self).0@ == id@)]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    #[ensures(result@ == self.0@)]
    fn finish(&self) -> u64 {
        self.0
    }
}
