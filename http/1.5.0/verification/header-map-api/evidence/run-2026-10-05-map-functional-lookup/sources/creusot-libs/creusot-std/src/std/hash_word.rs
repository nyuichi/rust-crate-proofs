use crate::prelude::*;

// Return the unsigned machine word with the same two's-complement bit pattern
// as `value as usize`, without asking Creusot to model that wrapping cast.
#[ensures(
    result@ == if value@ >= 0 {
        value@
    } else {
        usize::MAX@ + 1 + value@
    }
)]
pub(crate) fn isize_to_usize_word(value: isize) -> usize {
    if value >= 0 {
        value as usize
    } else {
        usize::MAX - ((-(value + 1)) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::isize_to_usize_word;
    use core::hash::Hasher;

    #[derive(Default)]
    struct RecordingHasher(Option<usize>);

    impl Hasher for RecordingHasher {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, _bytes: &[u8]) {
            panic!("write_isize should dispatch to write_usize");
        }

        fn write_usize(&mut self, value: usize) {
            self.0 = Some(value);
        }
    }

    #[test]
    fn write_isize_word_conversion_matches_cast_and_callback_dispatch() {
        for value in [isize::MIN, -1, 0, isize::MAX] {
            assert_eq!(isize_to_usize_word(value), value as usize);

            let mut state = RecordingHasher::default();
            state.write_isize(value);
            assert_eq!(state.0, Some(value as usize));
        }
    }
}
