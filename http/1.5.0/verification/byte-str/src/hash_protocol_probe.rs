use core::hash::Hasher;
use creusot_std::prelude::*;

pub struct EvenHasher(pub u64);

impl Invariant for EvenHasher {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { self.0@ % 2 == 0 }
    }
}

impl Hasher for EvenHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

#[requires(inv(*state))]
#[ensures(inv(^state))]
pub(crate) fn hash_byte_str_with_even_hasher(
    value: &crate::byte_str::ByteStr,
    state: &mut EvenHasher,
) {
    core::hash::Hash::hash(value, state)
}

#[cfg(feature = "negative-hash-protocol")]
pub struct OddMakingHasher(pub u64);

#[cfg(feature = "negative-hash-protocol")]
impl Invariant for OddMakingHasher {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { self.0@ % 2 == 0 }
    }
}

#[cfg(feature = "negative-hash-protocol")]
impl Hasher for OddMakingHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        self.0 = 1;
    }
}
