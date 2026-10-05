use crate::prelude::*;
use core::hash::{Hash, Hasher};
use super::hash_word::isize_to_usize_word;

// The default Hasher helpers only forward to the required `write` callback.
// Their exact bodies matter because user overrides must preserve the mutable
// hasher invariant through this callback protocol.
extern_spec! {
    impl usize {
        #[check(ghost)]
        fn to_ne_bytes(self) -> [u8; core::mem::size_of::<usize>()];
    }

    impl u16 {
        #[check(ghost)]
        fn to_ne_bytes(self) -> [u8; 2];
    }
}

// These specs follow rustc 1.95.0-nightly commit
// 6a979b3e32522049d0acb4a47f7ae44b7c8abfd5. They describe callback
// preservation only: this model does not specify hash output bytes or
// Hash/Eq coherence.
extern_spec! {
    mod core {
        mod hash {
            trait Hasher {
                #[ensures(inv(^self))]
                fn write(&mut self, bytes: &[u8]);

                #[ensures(inv(^self))]
                fn write_usize(&mut self, value: usize) {
                    self.write(&value.to_ne_bytes())
                }

                #[ensures(inv(^self))]
                fn write_u8(&mut self, value: u8) {
                    self.write(&[value])
                }

                #[ensures(inv(^self))]
                fn write_u16(&mut self, value: u16) {
                    self.write(&value.to_ne_bytes())
                }

                // Pinned core implements this as `write_usize(value as usize)`.
                // Spell the same two's-complement word value without a
                // wrapping signed-to-unsigned cast so Creusot can verify the
                // callback invariant. In the negative branch, value + 1 and
                // its negation are both representable, including for MIN.
                #[ensures(inv(^self))]
                fn write_isize(&mut self, value: isize) {
                    let unsigned = isize_to_usize_word(value);
                    self.write_usize(unsigned)
                }

                #[ensures(inv(^self))]
                fn write_length_prefix(&mut self, len: usize) {
                    self.write_usize(len)
                }
            }

            trait Hash {
                #[ensures(inv(^state))]
                fn hash<H: Hasher>(&self, state: &mut H);

                // The default implementation is the pinned standard-library
                // element loop. Primitive implementations may override it.
                #[ensures(inv(^state))]
                fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
                where
                    Self: Sized,
                {
                    for element in data {
                        element.hash(state);
                    }
                }
            }

        }
    }
}

extern_spec! {
    impl Hash for u8 {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u8(*self)
        }

        // The u8 specialization writes the entire slice once, which matches
        // the pinned primitive byte-slice implementation.
        #[ensures(inv(^state))]
        fn hash_slice<H: Hasher>(data: &[u8], state: &mut H) {
            state.write(data)
        }
    }

    impl Hash for u16 {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u16(*self)
        }
    }

    impl Hash for usize {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_usize(*self)
        }
    }

    impl Hash for isize {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_isize(*self)
        }
    }

    impl<T: Hash> Hash for [T] {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_length_prefix(self.len());
            T::hash_slice(self, state)
        }
    }

    impl<T: Hash, const N: usize> Hash for [T; N] {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            Hash::hash(self.as_slice(), state)
        }
    }

    impl<T: ?Sized + Hash, A: core::alloc::Allocator> Hash for alloc::boxed::Box<T, A> {
        #[ensures(inv(^state))]
        fn hash<H: Hasher>(&self, state: &mut H) {
            (**self).hash(state)
        }
    }
}

extern_spec! {
    impl<T: crate::std::num::NonZeroPrimitiveModel + core::hash::Hash>
        core::hash::Hash for core::num::NonZero<T>
    {
        #[ensures(inv(^state))]
        fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
            self.get().hash(state)
        }
    }
}
