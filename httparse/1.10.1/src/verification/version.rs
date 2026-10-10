//! Independent exact model for `parse_version`.
//!
//! The model separates the parser's two control-flow paths: an eight-byte
//! fast path and an incremental path for shorter input. It records the exact
//! returned version/status and the absolute cursor after every result.

#[allow(unused_imports)]
use creusot_std::prelude::{
    check, ensures, logic, proof_assert, requires, snapshot, variant, DeepModel, Int, Seq,
    Snapshot,
};

/// Abstract result category from `parse_version`.
#[derive(Copy, Clone)]
pub enum VersionOutcome {
    /// `HTTP/1.0` completed with version value 0.
    Http10,
    /// `HTTP/1.1` completed with version value 1.
    Http11,
    /// A valid short prefix ended before eight bytes were available.
    Partial,
    /// The first eight bytes, or a consumed byte of a short prefix, mismatched.
    Error,
}

/// Exact abstract result of `parse_version`.
#[derive(Copy, Clone)]
pub struct VersionResult {
    /// Complete version, partial status, or version error.
    pub outcome: VersionOutcome,
    /// Absolute mark before parsing; `parse_version` never commits it.
    pub mark: Int,
    /// Absolute cursor after the operation.
    pub cursor: Int,
}

/// Expected byte at an offset in the short-path prefix `HTTP/1.`.
#[logic(open)]
#[requires(0 <= offset && offset < 7)]
pub fn version_prefix_byte(offset: Int) -> Int {
    if offset == 0 {
        72
    } else if offset == 1 || offset == 2 {
        84
    } else if offset == 3 {
        80
    } else if offset == 4 {
        47
    } else if offset == 5 {
        49
    } else {
        46
    }
}

/// Whether the eight bytes at `cursor` spell `HTTP/1.<minor>`.
///
/// The runtime fast path applies `u64::from_ne_bytes` to both the input block
/// and the literal. This bytewise model is endian-independent: applying the
/// same native-endian bijection to equal-width byte arrays preserves equality.
#[logic(open)]
#[requires(0 <= cursor && cursor + 8 <= end && end <= input.len())]
pub fn matches_version_bytes(
    input: Seq<u8>,
    cursor: Int,
    end: Int,
    minor: Int,
) -> bool {
    input[cursor].deep_model() == 72
        && input[cursor + 1].deep_model() == 84
        && input[cursor + 2].deep_model() == 84
        && input[cursor + 3].deep_model() == 80
        && input[cursor + 4].deep_model() == 47
        && input[cursor + 5].deep_model() == 49
        && input[cursor + 6].deep_model() == 46
        && input[cursor + 7].deep_model() == minor
}

/// Exact result of the incremental short-input path for fewer than eight bytes.
///
/// The seven comparisons are unrolled so this proof model has a visible finite
/// definition at every caller. Each mismatch branch consumes the mismatching
/// byte, while an exhausted matching prefix returns Partial at the end.
#[logic(open)]
#[requires(0 <= start && start <= end && end <= input.len())]
#[requires(0 <= mark && mark <= start)]
#[requires(end - start < 8)]
#[ensures(result.mark == mark)]
#[ensures(start <= result.cursor && result.cursor <= end)]
pub fn short_version_model(
    input: Seq<u8>,
    mark: Int,
    start: Int,
    end: Int,
) -> VersionResult {
    if end - start == 0 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start].deep_model() != version_prefix_byte(0) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 1,
        }
    } else if end - start == 1 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 1].deep_model() != version_prefix_byte(1) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 2,
        }
    } else if end - start == 2 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 2].deep_model() != version_prefix_byte(2) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 3,
        }
    } else if end - start == 3 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 3].deep_model() != version_prefix_byte(3) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 4,
        }
    } else if end - start == 4 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 4].deep_model() != version_prefix_byte(4) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 5,
        }
    } else if end - start == 5 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 5].deep_model() != version_prefix_byte(5) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 6,
        }
    } else if end - start == 6 {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[start + 6].deep_model() != version_prefix_byte(6) {
        VersionResult {
            outcome: VersionOutcome::Error,
            mark,
            cursor: start + 7,
        }
    } else {
        VersionResult {
            outcome: VersionOutcome::Partial,
            mark,
            cursor: end,
        }
    }
}

/// Exact model of `parse_version` for an immutable input and absolute cursor.
///
/// If at least eight bytes remain, the operation always consumes eight bytes,
/// including on an invalid version. With fewer than eight bytes, each matching
/// prefix byte is consumed; the first mismatch is consumed before returning an
/// error, while EOF returns `Partial` without another advance.
#[logic(open)]
#[requires(0 <= start && start <= end && end <= input.len())]
#[requires(0 <= mark && mark <= start)]
#[ensures(result.mark == mark)]
#[ensures(start <= result.cursor && result.cursor <= end)]
#[ensures(end - start >= 8 ==> result.cursor == start + 8)]
pub fn parse_version_model(
    input: Seq<u8>,
    mark: Int,
    start: Int,
    end: Int,
) -> VersionResult {
    if end - start >= 8 {
        if matches_version_bytes(input, start, end, 48) {
            VersionResult {
                outcome: VersionOutcome::Http10,
                mark,
                cursor: start + 8,
            }
        } else if matches_version_bytes(input, start, end, 49) {
            VersionResult {
                outcome: VersionOutcome::Http11,
                mark,
                cursor: start + 8,
            }
        } else {
            VersionResult {
                outcome: VersionOutcome::Error,
                mark,
                cursor: start + 8,
            }
        }
    } else {
        short_version_model(input, mark, start, end)
    }
}

/// Whether a modeled result is the requested complete version.
#[logic(open)]
pub fn version_result_is_complete(parsed: VersionResult, version: Int) -> bool {
    match parsed.outcome {
        VersionOutcome::Http10 => version == 0,
        VersionOutcome::Http11 => version == 1,
        _ => false,
    }
}

/// Whether a modeled result is partial.
#[logic(open)]
pub fn version_result_is_partial(parsed: VersionResult) -> bool {
    match parsed.outcome {
        VersionOutcome::Partial => true,
        _ => false,
    }
}

/// Whether a modeled result is a version error.
#[logic(open)]
pub fn version_result_is_error(parsed: VersionResult) -> bool {
    match parsed.outcome {
        VersionOutcome::Error => true,
        _ => false,
    }
}

/// Peel one base-256 digit from equal packed integers.
#[check(ghost)]
#[requires(0 <= *left_byte && *left_byte < 256)]
#[requires(0 <= *right_byte && *right_byte < 256)]
#[requires(*left_byte + 256 * *left_tail == *right_byte + 256 * *right_tail)]
#[ensures(*left_byte == *right_byte)]
#[ensures(*left_tail == *right_tail)]
pub fn base256_head_injective(
    left_byte: Snapshot<Int>,
    right_byte: Snapshot<Int>,
    left_tail: Snapshot<Int>,
    right_tail: Snapshot<Int>,
) {
    proof_assert!(*left_byte == *right_byte);
    proof_assert!(*left_tail == *right_tail);
}

/// Expose one byte and the remaining packed value from the standard-library model.
#[check(ghost)]
#[requires(bytes.len() == 8)]
#[requires(0 <= *offset && *offset < 8)]
#[ensures(creusot_std::std::num::u64_from_ne_bytes_tail(*bytes, *offset)
    == creusot_std::std::num::u64_from_ne_bytes_byte(*bytes, *offset)
        + 256 * creusot_std::std::num::u64_from_ne_bytes_tail(*bytes, *offset + 1))]
pub fn unfold_native_pack_tail(bytes: Snapshot<Seq<u8>>, offset: Snapshot<Int>) {}

/// The native pack tail is zero after its eight bytes.
#[check(ghost)]
#[requires(bytes.len() == 8)]
#[ensures(creusot_std::std::num::u64_from_ne_bytes_tail(*bytes, 8) == 0)]
pub fn native_pack_tail_end(bytes: Snapshot<Seq<u8>>) {}

/// Equal native-endian packed values imply equality of all eight bytes.
///
/// The byte significance order comes from the target-specific standard model;
/// all arithmetic peeling is body-checked here and adds no parser or literal
/// assumptions.
#[check(ghost)]
#[requires(left.len() == 8 && right.len() == 8)]
#[requires(creusot_std::std::num::u64_from_ne_bytes_value(*left)
    == creusot_std::std::num::u64_from_ne_bytes_value(*right))]
#[ensures((*left)[0].deep_model() == (*right)[0].deep_model())]
#[ensures((*left)[1].deep_model() == (*right)[1].deep_model())]
#[ensures((*left)[2].deep_model() == (*right)[2].deep_model())]
#[ensures((*left)[3].deep_model() == (*right)[3].deep_model())]
#[ensures((*left)[4].deep_model() == (*right)[4].deep_model())]
#[ensures((*left)[5].deep_model() == (*right)[5].deep_model())]
#[ensures((*left)[6].deep_model() == (*right)[6].deep_model())]
#[ensures((*left)[7].deep_model() == (*right)[7].deep_model())]
pub fn native_pack8_injective(left: Snapshot<Seq<u8>>, right: Snapshot<Seq<u8>>) {
    #[cfg(target_endian = "little")]
    {
        unfold_native_pack_tail(left, snapshot!(0));
        unfold_native_pack_tail(right, snapshot!(0));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 0)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 0)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 1)),
        );
        unfold_native_pack_tail(left, snapshot!(1));
        unfold_native_pack_tail(right, snapshot!(1));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 2)),
        );
        unfold_native_pack_tail(left, snapshot!(2));
        unfold_native_pack_tail(right, snapshot!(2));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 3)),
        );
        unfold_native_pack_tail(left, snapshot!(3));
        unfold_native_pack_tail(right, snapshot!(3));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 4)),
        );
        unfold_native_pack_tail(left, snapshot!(4));
        unfold_native_pack_tail(right, snapshot!(4));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 5)),
        );
        unfold_native_pack_tail(left, snapshot!(5));
        unfold_native_pack_tail(right, snapshot!(5));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 6)),
        );
        unfold_native_pack_tail(left, snapshot!(6));
        unfold_native_pack_tail(right, snapshot!(6));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 7)),
        );
        unfold_native_pack_tail(left, snapshot!(7));
        unfold_native_pack_tail(right, snapshot!(7));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 8)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 8)),
        );
        native_pack_tail_end(left);
        native_pack_tail_end(right);
    }

    #[cfg(target_endian = "big")]
    {
        unfold_native_pack_tail(left, snapshot!(0));
        unfold_native_pack_tail(right, snapshot!(0));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 0)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 0)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 1)),
        );
        unfold_native_pack_tail(left, snapshot!(1));
        unfold_native_pack_tail(right, snapshot!(1));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 1)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 2)),
        );
        unfold_native_pack_tail(left, snapshot!(2));
        unfold_native_pack_tail(right, snapshot!(2));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 2)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 3)),
        );
        unfold_native_pack_tail(left, snapshot!(3));
        unfold_native_pack_tail(right, snapshot!(3));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 3)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 4)),
        );
        unfold_native_pack_tail(left, snapshot!(4));
        unfold_native_pack_tail(right, snapshot!(4));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 4)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 5)),
        );
        unfold_native_pack_tail(left, snapshot!(5));
        unfold_native_pack_tail(right, snapshot!(5));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 5)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 6)),
        );
        unfold_native_pack_tail(left, snapshot!(6));
        unfold_native_pack_tail(right, snapshot!(6));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 6)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 7)),
        );
        unfold_native_pack_tail(left, snapshot!(7));
        unfold_native_pack_tail(right, snapshot!(7));
        base256_head_injective(
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*left, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_byte(*right, 7)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*left, 8)),
            snapshot!(creusot_std::std::num::u64_from_ne_bytes_tail(*right, 8)),
        );
        native_pack_tail_end(left);
        native_pack_tail_end(right);
    }
}
