//! Independent exact cursor model for `skip_empty_lines`.

#![allow(dead_code)]

#[allow(unused_imports)]
use creusot_std::prelude::{
    check, ensures, logic, proof_assert, requires, variant, DeepModel, Int, Seq, Snapshot,
};

/// Whether a complete LF or CRLF terminator starts at `cursor`.
#[logic(open)]
#[requires(0 <= cursor && cursor <= end && end <= input.len())]
pub fn complete_line_ending_at(input: Seq<u8>, cursor: Int, end: Int) -> bool {
    if cursor < end {
        if input[cursor].deep_model() == 10 {
            true
        } else if input[cursor].deep_model() == 13 && cursor + 1 < end {
            input[cursor + 1].deep_model() == 10
        } else {
            false
        }
    } else {
        false
    }
}

/// Whether `input[start..stop]` is a concatenation of complete LF or CRLF
/// line endings. A final CR is deliberately excluded: the runtime consumes
/// it and returns partial when the following `next()` reaches EOF.
#[logic]
#[variant(stop - start)]
#[requires(0 <= start && start <= stop && stop <= input.len())]
pub fn complete_line_prefix(input: Seq<u8>, start: Int, stop: Int) -> bool {
    if start == stop {
        true
    } else if input[start].deep_model() == 10 {
        complete_line_prefix(input, start + 1, stop)
    } else if input[start].deep_model() == 13
        && start + 1 < stop
        && input[start + 1].deep_model() == 10
    {
        complete_line_prefix(input, start + 2, stop)
    } else {
        false
    }
}

/// End of the maximal initial run of complete LF and CRLF line endings.
#[logic]
#[variant(end - start)]
#[requires(0 <= start && start <= end && end <= input.len())]
#[ensures(start <= result && result <= end)]
#[ensures(complete_line_prefix(input, start, result))]
#[ensures(result == end || !complete_line_ending_at(input, result, end))]
#[ensures(
    if start == end || !complete_line_ending_at(input, start, end) {
        result == start
    } else if input[start].deep_model() == 10 {
        result == complete_line_prefix_end(input, start + 1, end)
    } else {
        result == complete_line_prefix_end(input, start + 2, end)
    }
)]
pub fn complete_line_prefix_end(input: Seq<u8>, start: Int, end: Int) -> Int {
    if start == end || !complete_line_ending_at(input, start, end) {
        start
    } else if input[start].deep_model() == 10 {
        complete_line_prefix_end(input, start + 1, end)
    } else {
        complete_line_prefix_end(input, start + 2, end)
    }
}

/// Compose two adjacent complete-newline prefixes.
///
/// The induction follows the first prefix from `start` to `mid`, consuming
/// one LF byte or one CRLF pair at each recursive step. The suffix beginning
/// at `mid` is kept fixed, so the result proves concatenation without
/// unfolding the whole recursive predicate in each caller.
#[logic]
#[variant(mid - start)]
#[requires(0 <= start && start <= mid && mid <= stop)]
#[requires(stop <= input.len())]
#[requires(complete_line_prefix(input, start, mid))]
#[requires(complete_line_prefix(input, mid, stop))]
#[ensures(result == mid)]
#[ensures(complete_line_prefix(input, start, stop))]
pub fn complete_line_prefix_concat(input: Seq<u8>, start: Int, mid: Int, stop: Int) -> Int {
    if start < mid {
        if input[start].deep_model() == 10 {
            proof_assert! { complete_line_prefix(input, start + 1, mid) };
            complete_line_prefix_concat(input, start + 1, mid, stop);
            proof_assert! { complete_line_prefix(input, start, stop) };
        } else {
            proof_assert! { input[start].deep_model() == 13 };
            proof_assert! { start + 1 < mid };
            proof_assert! { input[start + 1].deep_model() == 10 };
            proof_assert! { complete_line_prefix(input, start + 2, mid) };
            complete_line_prefix_concat(input, start + 2, mid, stop);
            proof_assert! { complete_line_prefix(input, start, stop) };
        }
        mid
    } else {
        proof_assert! { complete_line_prefix(input, start, stop) };
        mid
    }
}

/// Extend a complete-newline prefix by one bare LF.
#[check(ghost)]
#[requires(0 <= *start && *start <= *cursor && *cursor < *end)]
#[requires(*end <= input.len())]
#[requires(complete_line_prefix(*input, *start, *cursor))]
#[requires((*input)[*cursor].deep_model() == 10)]
#[ensures(complete_line_prefix(*input, *start, *cursor + 1))]
pub fn complete_line_prefix_extend_lf(
    input: Snapshot<Seq<u8>>,
    start: Snapshot<Int>,
    cursor: Snapshot<Int>,
    end: Snapshot<Int>,
) {
    proof_assert! { complete_line_prefix(*input, *cursor, *cursor + 1) };
    proof_assert! { complete_line_prefix_concat(*input, *start, *cursor, *cursor + 1) == *cursor };
}

/// Extend a complete-newline prefix by one CRLF pair.
#[check(ghost)]
#[requires(0 <= *start && *start <= *cursor && *cursor + 1 < *end)]
#[requires(*end <= input.len())]
#[requires(complete_line_prefix(*input, *start, *cursor))]
#[requires((*input)[*cursor].deep_model() == 13)]
#[requires((*input)[*cursor + 1].deep_model() == 10)]
#[ensures(complete_line_prefix(*input, *start, *cursor + 2))]
pub fn complete_line_prefix_extend_crlf(
    input: Snapshot<Seq<u8>>,
    start: Snapshot<Int>,
    cursor: Snapshot<Int>,
    end: Snapshot<Int>,
) {
    proof_assert! { complete_line_prefix(*input, *cursor, *cursor + 2) };
    proof_assert! { complete_line_prefix_concat(*input, *start, *cursor, *cursor + 2) == *cursor };
}

/// Follow a known complete-line prefix one LF byte or CRLF pair at a time.
/// The endpoint's checked recurrence connects each step with the maximal
/// complete-line prefix end, so the induction relates every cursor position
/// without assuming a general prefix-cancellation property.
#[logic]
#[variant(cursor - start)]
#[requires(0 <= start && start <= cursor && cursor <= end)]
#[requires(end <= input.len())]
#[requires(complete_line_prefix(input, start, cursor))]
#[ensures(result == cursor)]
#[ensures(cursor <= complete_line_prefix_end(input, start, end))]
#[ensures(complete_line_ending_at(input, cursor, end)
    ==> cursor < complete_line_prefix_end(input, start, end))]
#[ensures((cursor == end || !complete_line_ending_at(input, cursor, end))
    ==> cursor == complete_line_prefix_end(input, start, end))]
pub fn complete_line_prefix_cursor_induction(
    input: Seq<u8>,
    start: Int,
    cursor: Int,
    end: Int,
) -> Int {
    if start < cursor {
        if input[start].deep_model() == 10 {
            proof_assert! { complete_line_prefix(input, start + 1, cursor) };
            complete_line_prefix_cursor_induction(input, start + 1, cursor, end);
            proof_assert! {
                complete_line_prefix_end(input, start, end)
                    == complete_line_prefix_end(input, start + 1, end)
            };
        } else {
            proof_assert! { input[start].deep_model() == 13 };
            proof_assert! { start + 1 < cursor };
            proof_assert! { input[start + 1].deep_model() == 10 };
            proof_assert! { complete_line_prefix(input, start + 2, cursor) };
            complete_line_prefix_cursor_induction(input, start + 2, cursor, end);
            proof_assert! {
                complete_line_prefix_end(input, start, end)
                    == complete_line_prefix_end(input, start + 2, end)
            };
        }
    }
    cursor
}

/// Relate the maximal complete-line prefix endpoint to the loop cursor.
#[check(ghost)]
#[requires(0 <= *start && *start <= *cursor && *cursor <= *end)]
#[requires(*end <= input.len())]
#[requires(complete_line_prefix(*input, *start, *cursor))]
#[ensures(*cursor <= complete_line_prefix_end(*input, *start, *end))]
#[ensures(complete_line_ending_at(*input, *cursor, *end)
    ==> *cursor < complete_line_prefix_end(*input, *start, *end))]
#[ensures((*cursor == *end
    || !complete_line_ending_at(*input, *cursor, *end))
    ==> *cursor == complete_line_prefix_end(*input, *start, *end))]
pub fn complete_line_prefix_at_cursor(
    input: Snapshot<Seq<u8>>,
    start: Snapshot<Int>,
    cursor: Snapshot<Int>,
    end: Snapshot<Int>,
) {
    proof_assert! {
        complete_line_prefix_cursor_induction(*input, *start, *cursor, *end) == *cursor
    };
}

/// Terminal behavior category of `skip_empty_lines`.
#[derive(Copy, Clone)]
pub enum EmptyLinesOutcome {
    /// A non-CR/LF byte remains and the runtime commits the cursor.
    Complete,
    /// Input ended after complete line endings or after a trailing CR.
    Partial,
    /// A CR was followed by a consumed byte other than LF.
    NewLineError,
}

/// Exact absolute cursor and committed mark after `skip_empty_lines`.
#[derive(Copy, Clone)]
pub struct EmptyLinesResult {
    /// The complete, partial, or newline-error result.
    pub outcome: EmptyLinesOutcome,
    /// Committed start offset after the call.
    pub mark: Int,
    /// Absolute next-byte offset after the call.
    pub cursor: Int,
}

#[logic(open)]
pub fn empty_lines_is_complete(parsed: EmptyLinesResult) -> bool {
    match parsed.outcome {
        EmptyLinesOutcome::Complete => true,
        _ => false,
    }
}

#[logic(open)]
pub fn empty_lines_is_partial(parsed: EmptyLinesResult) -> bool {
    match parsed.outcome {
        EmptyLinesOutcome::Partial => true,
        _ => false,
    }
}

#[logic(open)]
pub fn empty_lines_is_newline_error(parsed: EmptyLinesResult) -> bool {
    match parsed.outcome {
        EmptyLinesOutcome::NewLineError => true,
        _ => false,
    }
}

/// Exact independent model of the runtime consumption and commit behavior.
///
/// Complete LF and CRLF pairs are consumed repeatedly. At the first other
/// byte, `slice()` commits that cursor without consuming the byte. Exhaustion
/// after line endings is partial with the old mark. A final CR is consumed
/// before `next()` observes EOF, so it is also partial at `end` with the old
/// mark. A CR followed by a non-LF byte consumes both bytes before returning
/// `Error::NewLine`, again preserving the old mark.
#[logic]
#[requires(0 <= mark && mark <= cursor)]
#[requires(cursor <= end && end <= input.len())]
#[ensures(match result.outcome {
    EmptyLinesOutcome::Complete => {
        let line_end = complete_line_prefix_end(input, cursor, end);
        line_end < end
            && input[line_end].deep_model() != 13
            && input[line_end].deep_model() != 10
            && result.cursor == line_end
            && result.mark == line_end
    },
    EmptyLinesOutcome::Partial => {
        let line_end = complete_line_prefix_end(input, cursor, end);
        result.cursor == end
            && result.mark == mark
            && (line_end == end
                || (input[line_end].deep_model() == 13 && line_end + 1 == end))
    },
    EmptyLinesOutcome::NewLineError => {
        let line_end = complete_line_prefix_end(input, cursor, end);
        line_end + 1 < end
            && input[line_end].deep_model() == 13
            && input[line_end + 1].deep_model() != 10
            && result.cursor == line_end + 2
            && result.mark == mark
    },
})]
pub fn skip_empty_lines_model(
    input: Seq<u8>,
    mark: Int,
    cursor: Int,
    end: Int,
) -> EmptyLinesResult {
    let line_end = complete_line_prefix_end(input, cursor, end);
    if line_end == end {
        EmptyLinesResult {
            outcome: EmptyLinesOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[line_end].deep_model() == 13 {
        if line_end + 1 == end {
            EmptyLinesResult {
                outcome: EmptyLinesOutcome::Partial,
                mark,
                cursor: end,
            }
        } else {
            EmptyLinesResult {
                outcome: EmptyLinesOutcome::NewLineError,
                mark,
                cursor: line_end + 2,
            }
        }
    } else {
        EmptyLinesResult {
            outcome: EmptyLinesOutcome::Complete,
            mark: line_end,
            cursor: line_end,
        }
    }
}
