//! Creusot 0.13 probe for the native original-capacity metadata helpers.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;

/// Expose the helper's range and the exact leading-zero width equation to a
/// representative caller.
#[ensures(result@ <= 7)]
#[ensures(
    result@ == if (capacity >> capacity_ops::MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@ == 64 {
        0
    } else if 64 - (capacity >> capacity_ops::MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@ > 7 {
        7
    } else {
        64 - (capacity >> capacity_ops::MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros_logic()@
    }
)]
pub fn encode_capacity(capacity: usize) -> usize {
    capacity_ops::original_capacity_to_repr(capacity)
}

/// Decode only values that can be stored in the three-bit metadata field.
#[requires(repr@ <= 7)]
#[ensures(repr == 0usize ==> result == 0usize)]
#[ensures(repr > 0usize ==> result == (1usize << (repr + 9usize)))]
#[ensures(result <= (1usize << 16usize))]
pub fn decode_capacity(repr: usize) -> usize {
    capacity_ops::original_capacity_from_repr(repr)
}

/// Check the useful round-trip fact: the decoded capacity is a retained floor
/// of the original capacity, capped at the 64 KiB representation limit.
#[ensures(result <= capacity)]
#[ensures(capacity < (1usize << 10usize) ==> result == 0usize)]
#[ensures(capacity >= (1usize << 10usize) ==> result >= (1usize << 10usize))]
#[ensures(result <= (1usize << 16usize))]
pub fn reconstructed_capacity(capacity: usize) -> usize {
    capacity_ops::reconstructed_original_capacity(capacity)
}

/// Packing then extracting a valid original-capacity class returns that class.
#[requires(repr@ <= 7)]
#[ensures(result == repr)]
pub fn packed_repr_roundtrip(repr: usize) -> usize {
    capacity_ops::original_capacity_repr_from_data(capacity_ops::pack_vec_metadata(repr))
}

/// Updating a packed vector position and reading it back returns the new value.
#[requires(pos@ <= capacity_ops::MAX_VEC_POS@)]
#[ensures(result == pos)]
pub fn updated_vec_position(data: usize, pos: usize) -> usize {
    capacity_ops::vec_pos_from_data(capacity_ops::set_vec_pos_in_data(data, pos))
}

/// Updating the vector position leaves all low five metadata bits untouched.
#[requires(pos@ <= capacity_ops::MAX_VEC_POS@)]
#[ensures(result == data & capacity_ops::NOT_VEC_POS_MASK)]
pub fn low_metadata_after_position_update(data: usize, pos: usize) -> usize {
    let updated = capacity_ops::set_vec_pos_in_data(data, pos);
    updated & capacity_ops::NOT_VEC_POS_MASK
}

/// Updating the vector position retains the existing Vec/Arc kind bit.
#[requires(pos@ <= capacity_ops::MAX_VEC_POS@)]
#[ensures(result == data & capacity_ops::KIND_MASK)]
pub fn kind_after_position_update(data: usize, pos: usize) -> usize {
    let updated = capacity_ops::set_vec_pos_in_data(data, pos);
    updated & capacity_ops::KIND_MASK
}

#[cfg(feature = "wrong_repr")]
#[requires(capacity == (1usize << 10usize))]
#[ensures(result == 0usize)]
pub fn wrong_repr(capacity: usize) -> usize {
    capacity_ops::original_capacity_to_repr(capacity)
}

#[cfg(feature = "wrong_reconstruction")]
#[requires(repr == 1usize)]
#[ensures(result == (1usize << 11usize))]
pub fn wrong_reconstruction(repr: usize) -> usize {
    capacity_ops::original_capacity_from_repr(repr)
}

#[cfg(feature = "lostflag")]
#[requires(data & capacity_ops::KIND_MASK == capacity_ops::KIND_VEC)]
#[requires(pos@ <= capacity_ops::MAX_VEC_POS@)]
#[ensures(result & capacity_ops::KIND_MASK == 0usize)]
pub fn lostflag(data: usize, pos: usize) -> usize {
    capacity_ops::set_vec_pos_in_data(data, pos)
}

#[cfg(feature = "wrongpos")]
#[requires(pos@ < capacity_ops::MAX_VEC_POS@)]
#[ensures(result == pos + 1usize)]
pub fn wrongpos(data: usize, pos: usize) -> usize {
    capacity_ops::vec_pos_from_data(capacity_ops::set_vec_pos_in_data(data, pos))
}
