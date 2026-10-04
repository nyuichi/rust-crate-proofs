//! Proof probe for checked variable-width unsigned slice reads.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/byte_codec_wide_ops.rs"]
mod byte_codec_wide_ops;
#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;
#[path = "../../../../src/variable_read_ops.rs"]
mod variable_read_ops;

#[requires(nbytes@ <= 8)]
#[ensures(match result {
    Some(value) => input@.len() >= nbytes@
        && value@ == variable_read_ops::be_weight(input@, nbytes@)
        && (^input)@ == input@[nbytes@..],
    None => input@.len() < nbytes@ && (^input)@ == input@
})]
pub fn read_be_u64(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    variable_read_ops::read_be_u64(input, nbytes)
}

#[requires(nbytes@ <= 8)]
#[ensures(match result {
    Some(value) => input@.len() >= nbytes@
        && value@ == variable_read_ops::le_weight(input@, nbytes@)
        && (^input)@ == input@[nbytes@..],
    None => input@.len() < nbytes@ && (^input)@ == input@
})]
pub fn read_le_u64(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    variable_read_ops::read_le_u64(input, nbytes)
}

#[cfg(feature = "wrong_value")]
#[requires(nbytes@ == 1 && input@.len() >= 1 && input@[0]@ > 0)]
#[ensures(match result {
    Some(value) => value@ == 0,
    None => false
})]
pub fn wrong_value(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    variable_read_ops::read_be_u64(input, nbytes)
}

#[cfg(feature = "wrong_short_consumed")]
#[requires(nbytes@ <= 8 && nbytes@ > input@.len() && input@.len() > 0)]
#[ensures(match result {
    None => (^input)@ == input@,
    Some(_) => false
})]
pub fn wrong_short_consumed(input: &mut &[u8], nbytes: usize) -> Option<u64> {
    crate::slice_ops::advance_slice(input, 1);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATTERNS: [[u8; 8]; 5] = [
        [0; 8],
        [255; 8],
        [0, 1, 2, 3, 4, 5, 6, 7],
        [128, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 128],
    ];

    #[test]
    fn all_widths_match_native_zero_padded_conversions_and_consume_exact_prefix() {
        for bytes in PATTERNS {
            for nbytes in 0..=8 {
                let mut be = [0u8; 8];
                be[8 - nbytes..].copy_from_slice(&bytes[..nbytes]);
                let mut be_input = &bytes[..];
                assert_eq!(
                    read_be_u64(&mut be_input, nbytes),
                    Some(u64::from_be_bytes(be))
                );
                assert_eq!(be_input, &bytes[nbytes..]);

                let mut le = [0u8; 8];
                le[..nbytes].copy_from_slice(&bytes[..nbytes]);
                let mut le_input = &bytes[..];
                assert_eq!(
                    read_le_u64(&mut le_input, nbytes),
                    Some(u64::from_le_bytes(le))
                );
                assert_eq!(le_input, &bytes[nbytes..]);
            }
        }
    }

    #[test]
    fn every_short_width_preserves_the_input() {
        let bytes = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        for nbytes in 0..=8 {
            for len in 0..nbytes {
                let mut be_input = &bytes[..len];
                let original = be_input;
                assert_eq!(read_be_u64(&mut be_input, nbytes), None);
                assert_eq!(be_input, original);

                let mut le_input = &bytes[..len];
                let original = le_input;
                assert_eq!(read_le_u64(&mut le_input, nbytes), None);
                assert_eq!(le_input, original);
            }
        }
    }
}
