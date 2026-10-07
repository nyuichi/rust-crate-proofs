//! Public logical byte models for integer encodings used by `ExclusiveBytes`.
//!
//! These functions describe initialized byte sequences only. They make no
//! claim about floating-point materialization or allocation behavior.

use creusot_std::prelude::*;

/// The power table used by the integer codec contracts, for exponents 0..15.
#[logic(open)]
pub fn byte_power(exponent: Int) -> Int {
    pearlite! {
        if exponent == 0 { 1 }
        else if exponent == 1 { 256 }
        else if exponent == 2 { 65_536 }
        else if exponent == 3 { 16_777_216 }
        else if exponent == 4 { 4_294_967_296 }
        else if exponent == 5 { 1_099_511_627_776 }
        else if exponent == 6 { 281_474_976_710_656 }
        else if exponent == 7 { 72_057_594_037_927_936 }
        else if exponent == 8 { 18_446_744_073_709_551_616 }
        else if exponent == 9 { 4_722_366_482_869_645_213_696 }
        else if exponent == 10 { 1_208_925_819_614_629_174_706_176 }
        else if exponent == 11 { 309_485_009_821_345_068_724_781_056 }
        else if exponent == 12 { 79_228_162_514_264_337_593_543_950_336 }
        else if exponent == 13 { 20_282_409_603_651_670_423_947_251_286_016 }
        else if exponent == 14 { 5_192_296_858_534_827_628_530_496_329_220_096 }
        else if exponent == 15 { 1_329_227_995_784_915_872_903_807_060_280_344_576 }
        else { 1 }
    }
}

/// The low `width` bytes of `value`, ordered most significant byte first.
#[logic(open)]
#[requires(0 <= value)]
#[requires(0 <= width && width <= 16)]
pub fn model_be_bytes(value: Int, width: Int) -> Seq<Int> {
    pearlite! {
        Seq::create(width, |index: Int|
            value / byte_power(width - 1 - index) % 256)
    }
}

/// The low `width` bytes of `value`, ordered least significant byte first.
#[logic(open)]
#[requires(0 <= value)]
#[requires(0 <= width && width <= 16)]
pub fn model_le_bytes(value: Int, width: Int) -> Seq<Int> {
    pearlite! {
        Seq::create(width, |index: Int|
            value / byte_power(index) % 256)
    }
}

/// Exact append relation: preserve the old prefix and append the modeled bytes.
#[logic(open)]
pub fn append_model_holds(
    old: Seq<u8>,
    new: Seq<u8>,
    suffix: Seq<Int>,
) -> bool {
    pearlite! {
        new.len() == old.len() + suffix.len()
            && (forall<i: Int> 0 <= i && i < old.len() ==> new[i] == old[i])
            && (forall<j: Int> 0 <= j && j < suffix.len() ==>
                new[old.len() + j]@ == suffix[j])
    }
}
