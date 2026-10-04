//! Original-capacity metadata encoding used by `BytesMut`.
//!
//! The capacity conversion functions directly extract `original_capacity_to_repr`
//! and `original_capacity_from_repr` from `bytes_mut.rs` (lines 1502--1516).
//! The packed metadata helpers follow the native expressions at lines 781, 934,
//! 1025--1026, and 1076--1083. Keep their constants and bit operations in sync
//! with that implementation. Pointer/integer casts remain at the native call
//! sites and are outside this pure arithmetic model.

#[cfg(creusot)]
use creusot_std::prelude::*;

// The maximum original capacity width. Any `Bytes` allocated with a greater
// initial capacity will default to this.
pub(crate) const MAX_ORIGINAL_CAPACITY_WIDTH: usize = 17;
// Original-capacity encoding starts at 1 KiB.
pub(crate) const MIN_ORIGINAL_CAPACITY_WIDTH: usize = 10;
// These are consumed by the surrounding `BytesMut` metadata packing and
// extraction sites, outside this pair of pure helpers.
#[allow(dead_code)]
pub(crate) const ORIGINAL_CAPACITY_MASK: usize = 0b11100;
#[allow(dead_code)]
pub(crate) const ORIGINAL_CAPACITY_OFFSET: usize = 2;

pub(crate) const KIND_VEC: usize = 0b1;
#[allow(dead_code)]
pub(crate) const KIND_MASK: usize = 0b1;
pub(crate) const VEC_POS_OFFSET: usize = 5;
pub(crate) const NOT_VEC_POS_MASK: usize = 0b11111;
pub(crate) const MAX_VEC_POS: usize = usize::MAX >> VEC_POS_OFFSET;

#[cfg(target_pointer_width = "64")]
const PTR_WIDTH: usize = 64;
#[cfg(target_pointer_width = "32")]
const PTR_WIDTH: usize = 32;

/// Encode a vector capacity as the capped bit width above the 1 KiB floor.
///
/// The result is in `0..=7`: zero represents capacities below 1 KiB, and each
/// positive value represents a power-of-two capacity class from 1 KiB through
/// 64 KiB. Larger capacities saturate at the 64 KiB class.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result@ <= MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@))]
#[cfg_attr(creusot, ensures(
    result@ == if (cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@ == PTR_WIDTH@ {
        0
    } else if PTR_WIDTH@ - (cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@ > MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@ {
        MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@
    } else {
        PTR_WIDTH@ - (cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@
    }
))]
pub(crate) fn original_capacity_to_repr(cap: usize) -> usize {
    let width = PTR_WIDTH - ((cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros() as usize);
    core::cmp::min(
        width,
        MAX_ORIGINAL_CAPACITY_WIDTH - MIN_ORIGINAL_CAPACITY_WIDTH,
    )
}

/// Reconstruct the power-of-two capacity represented by the metadata.
///
/// `repr` is a three-bit field extracted from the `BytesMut` metadata word.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, requires(repr@ <= MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@))]
#[cfg_attr(creusot, ensures(repr == 0usize ==> result == 0usize))]
#[cfg_attr(creusot, ensures(repr > 0usize ==> result == (1usize << (repr + 9usize))))]
#[cfg_attr(creusot, ensures(repr > 0usize ==> result >= (1usize << 10usize)))]
#[cfg_attr(creusot, ensures(result <= (1usize << 16usize)))]
pub(crate) fn original_capacity_from_repr(repr: usize) -> usize {
    if repr == 0 {
        return 0;
    }

    1 << (repr + (MIN_ORIGINAL_CAPACITY_WIDTH - 1))
}

/// Encode and reconstruct the capacity floor stored for a shared buffer.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result <= cap))]
#[cfg_attr(creusot, ensures(cap < (1usize << 10usize) ==> result == 0usize))]
#[cfg_attr(creusot, ensures(cap >= (1usize << 10usize) ==> result >= (1usize << 10usize)))]
#[cfg_attr(creusot, ensures(result <= (1usize << 16usize)))]
#[allow(dead_code)]
pub(crate) fn reconstructed_original_capacity(cap: usize) -> usize {
    original_capacity_from_repr(original_capacity_to_repr(cap))
}

/// Pack the original-capacity class and `Vec` kind flag into `BytesMut::data`.
///
/// This matches both native sites that construct `(repr << 2) | KIND_VEC`.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, requires(repr@ <= MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@))]
#[cfg_attr(creusot, ensures(result == (repr << ORIGINAL_CAPACITY_OFFSET) | KIND_VEC))]
#[cfg_attr(creusot, ensures(((result & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET) == repr))]
#[cfg_attr(creusot, ensures(result & KIND_MASK == KIND_VEC))]
pub(crate) fn pack_vec_metadata(repr: usize) -> usize {
    (repr << ORIGINAL_CAPACITY_OFFSET) | KIND_VEC
}

/// Read the original-capacity class from the packed metadata word.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result == (data & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET))]
#[cfg_attr(creusot, ensures(result@ <= MAX_ORIGINAL_CAPACITY_WIDTH@ - MIN_ORIGINAL_CAPACITY_WIDTH@))]
pub(crate) fn original_capacity_repr_from_data(data: usize) -> usize {
    (data & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET
}

/// Read the encoded vector offset from the packed metadata word.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, ensures(result == data >> VEC_POS_OFFSET))]
pub(crate) fn vec_pos_from_data(data: usize) -> usize {
    data >> VEC_POS_OFFSET
}

/// Replace the packed vector offset while preserving all five low metadata bits.
#[cfg_attr(creusot, bitwise_proof)]
#[cfg_attr(creusot, requires(pos@ <= MAX_VEC_POS@))]
#[cfg_attr(creusot, ensures(result == (pos << VEC_POS_OFFSET) | (data & NOT_VEC_POS_MASK)))]
#[cfg_attr(creusot, ensures((result >> VEC_POS_OFFSET) == pos))]
#[cfg_attr(creusot, ensures(result & NOT_VEC_POS_MASK == data & NOT_VEC_POS_MASK))]
#[cfg_attr(creusot, ensures(result & KIND_MASK == data & KIND_MASK))]
pub(crate) fn set_vec_pos_in_data(data: usize, pos: usize) -> usize {
    (pos << VEC_POS_OFFSET) | (data & NOT_VEC_POS_MASK)
}
