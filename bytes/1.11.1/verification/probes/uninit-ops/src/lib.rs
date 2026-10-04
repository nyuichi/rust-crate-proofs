//! Proof probe for safe initialization of `MaybeUninit<u8>` prefixes.
#![allow(unexpected_cfgs)]

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

#[path = "../../../../src/uninit_ops.rs"]
mod uninit_ops;

/// Copies source bytes into a bounded prefix and returns initialized slots.
#[requires(source@.len() <= input@.len())]
#[ensures(result@.len() == source@.len())]
#[ensures(forall<i> 0 <= i && i < source@.len() ==> result@[i]@ == Some(source@[i]))]
#[ensures((^input)@ == input@[source@.len()..])]
pub fn copy_prefix<'a>(
    input: &mut &'a mut [MaybeUninit<u8>],
    source: &[u8],
) -> &'a mut [MaybeUninit<u8>] {
    uninit_ops::initialize_prefix(input, source)
}

/// Fills a bounded prefix and returns initialized slots.
#[requires(count@ <= input@.len())]
#[ensures(result@.len() == count@)]
#[ensures(forall<i> 0 <= i && i < count@ ==> result@[i]@ == Some(value))]
#[ensures((^input)@ == input@[count@..])]
pub fn fill_prefix<'a>(
    input: &mut &'a mut [MaybeUninit<u8>],
    count: usize,
    value: u8,
) -> &'a mut [MaybeUninit<u8>] {
    uninit_ops::fill_prefix(input, count, value)
}

#[cfg(feature = "wrong_init")]
#[requires(source@.len() > 0)]
#[requires(source@.len() <= input@.len())]
#[requires(source@[0]@ != 0)]
#[ensures(result@[0]@ == Some(0u8))]
pub fn wrong_init<'a>(
    input: &mut &'a mut [MaybeUninit<u8>],
    source: &[u8],
) -> &'a mut [MaybeUninit<u8>] {
    uninit_ops::initialize_prefix(input, source)
}

#[cfg(feature = "uninitialized_recovery")]
pub unsafe fn uninitialized_recovery() -> u8 {
    let value = MaybeUninit::uninit();
    value.assume_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_into_initialized_slots_and_preserves_the_remaining_suffix() {
        let mut storage = [MaybeUninit::new(0xee); 4];
        let mut input = &mut storage[..];
        let prefix = copy_prefix(&mut input, &[1, 2]);

        assert_eq!(unsafe { prefix[0].assume_init() }, 1);
        assert_eq!(unsafe { prefix[1].assume_init() }, 2);
        assert_eq!(input.len(), 2);
        assert_eq!(unsafe { input[0].assume_init() }, 0xee);
        assert_eq!(unsafe { input[1].assume_init() }, 0xee);
    }

    #[test]
    fn fills_uninitialized_slots_and_returns_the_remaining_suffix() {
        let mut storage = [MaybeUninit::uninit(); 4];
        let mut input = &mut storage[..];
        let prefix = fill_prefix(&mut input, 3, 0x5a);

        for slot in prefix {
            assert_eq!(unsafe { slot.assume_init() }, 0x5a);
        }
        assert_eq!(input.len(), 1);
    }
}
