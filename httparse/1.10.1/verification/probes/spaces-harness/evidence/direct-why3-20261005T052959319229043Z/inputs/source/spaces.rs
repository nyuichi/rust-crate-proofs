//! Independent exact cursor model for the `skip_spaces` helper.

#![allow(dead_code)]

#[allow(unused_imports)]
use creusot_std::prelude::{check, logic, requires, ensures, variant, DeepModel, Int, Seq, Snapshot};

/// End of the maximal initial run of literal ASCII spaces in `input[start..end]`.
#[logic]
#[variant(end - start)]
#[requires(0 <= start && start <= end && end <= input.len())]
#[ensures(start <= result && result <= end)]
#[ensures(forall<i: Int> start <= i && i < result
    ==> input[i].deep_model() == 32)]
#[ensures(result < end ==> input[result].deep_model() != 32)]
pub fn space_prefix_end(input: Seq<u8>, start: Int, end: Int) -> Int {
    if start == end || input[start].deep_model() != 32 {
        start
    } else {
        space_prefix_end(input, start + 1, end)
    }
}

/// Relate the opaque maximal-prefix endpoint to the current scan cursor.
///
/// The caller maintains that every byte from `start` up to `cursor` is a
/// space. At a non-space byte (or EOF), the cursor is exactly the endpoint;
/// when the current byte is a space, the endpoint lies strictly after it.
/// Keeping this fact in a small checked lemma gives the scanner loop a local
/// bridge without unfolding the recursive endpoint model in its invariant.
#[check(ghost)]
#[requires(0 <= *start && *start <= *cursor && *cursor <= *end)]
#[requires(*end <= input.len())]
#[requires(forall<i: Int> *start <= i && i < *cursor
    ==> (*input)[i].deep_model() == 32)]
#[ensures(*cursor == *end || (*input)[*cursor].deep_model() != 32
    ==> *cursor == space_prefix_end(*input, *start, *end))]
#[ensures(*cursor < *end && (*input)[*cursor].deep_model() == 32
    ==> *cursor < space_prefix_end(*input, *start, *end))]
pub fn space_prefix_at_cursor(
    input: Snapshot<Seq<u8>>,
    start: Snapshot<Int>,
    cursor: Snapshot<Int>,
    end: Snapshot<Int>,
) {}

/// Terminal result category of `skip_spaces`.
#[derive(Copy, Clone)]
pub enum SpacesOutcome {
    /// A non-space byte remains and the helper commits its current cursor.
    Complete,
    /// The available input ended after zero or more spaces.
    Partial,
}

/// Exact result and absolute cursor state after `skip_spaces`.
#[derive(Copy, Clone)]
pub struct SpacesResult {
    /// Completion or input exhaustion.
    pub outcome: SpacesOutcome,
    /// Committed start offset after the call.
    pub mark: Int,
    /// Absolute next-byte offset after the call.
    pub cursor: Int,
}

/// Whether the independent model describes completion at a non-space byte.
#[logic(open)]
pub fn spaces_result_is_complete(parsed: SpacesResult) -> bool {
    match parsed.outcome {
        SpacesOutcome::Complete => true,
        SpacesOutcome::Partial => false,
    }
}

/// Whether the independent model describes exhaustion after spaces.
#[logic(open)]
pub fn spaces_result_is_partial(parsed: SpacesResult) -> bool {
    match parsed.outcome {
        SpacesOutcome::Complete => false,
        SpacesOutcome::Partial => true,
    }
}

/// Exact model of the runtime helper from an arbitrary valid `Bytes` state.
///
/// The helper consumes a maximal run of `0x20`. A following non-space byte is
/// left at the cursor and causes `slice()` to commit that cursor. If the run
/// reaches EOF, the helper returns partial without committing, so the old
/// mark remains observable even though the cursor is at `end`.
#[logic(open)]
#[requires(0 <= mark && mark <= cursor)]
#[requires(cursor <= end && end <= input.len())]
#[ensures(result.cursor == space_prefix_end(input, cursor, end))]
#[ensures(match result.outcome {
    SpacesOutcome::Complete => result.cursor < end && result.mark == result.cursor,
    SpacesOutcome::Partial => result.cursor == end && result.mark == mark,
})]
pub fn skip_spaces_model(input: Seq<u8>, mark: Int, cursor: Int, end: Int) -> SpacesResult {
    let after_spaces = space_prefix_end(input, cursor, end);
    if after_spaces < end {
        SpacesResult {
            outcome: SpacesOutcome::Complete,
            mark: after_spaces,
            cursor: after_spaces,
        }
    } else {
        SpacesResult {
            outcome: SpacesOutcome::Partial,
            mark,
            cursor: end,
        }
    }
}
