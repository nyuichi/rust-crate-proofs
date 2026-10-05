//! Independent exact cursor model for the request-line newline helper.

#![allow(dead_code)]

#[allow(unused_imports)]
use creusot_std::prelude::{logic, requires, DeepModel, Int, Seq};

/// Terminal result category of `parse_newline`.
#[derive(Copy, Clone)]
pub enum NewlineOutcome {
    /// LF or CRLF was consumed and committed.
    Complete,
    /// The input ended before a full newline was available.
    Partial,
    /// A non-newline byte, or a non-LF byte after CR, was consumed.
    Error,
}

/// Exact absolute cursor and committed mark after `parse_newline`.
#[derive(Copy, Clone)]
pub struct NewlineResult {
    /// Completion, input exhaustion, or malformed newline.
    pub outcome: NewlineOutcome,
    /// Committed start offset after the call.
    pub mark: Int,
    /// Absolute next-byte offset after the call.
    pub cursor: Int,
}

/// Whether the model describes a successfully consumed and committed newline.
#[logic(open)]
pub fn newline_result_is_complete(parsed: NewlineResult) -> bool {
    match parsed.outcome {
        NewlineOutcome::Complete => true,
        _ => false,
    }
}

/// Whether the model describes exhaustion before a complete newline.
#[logic(open)]
pub fn newline_result_is_partial(parsed: NewlineResult) -> bool {
    match parsed.outcome {
        NewlineOutcome::Partial => true,
        _ => false,
    }
}

/// Whether the model describes a consumed malformed newline byte sequence.
#[logic(open)]
pub fn newline_result_is_error(parsed: NewlineResult) -> bool {
    match parsed.outcome {
        NewlineOutcome::Error => true,
        _ => false,
    }
}

/// Exact result, cursor, and mark model of the runtime helper.
///
/// LF consumes and commits one byte. CRLF consumes and commits two bytes. EOF
/// before the first byte or after CR is partial and preserves the old mark.
/// An invalid first byte is consumed before `Error::NewLine`; after CR, one
/// following non-LF byte is also consumed before the error. Error paths retain
/// the old mark because they do not call `slice()`.
#[logic(open)]
#[requires(0 <= mark && mark <= cursor)]
#[requires(cursor <= end && end <= input.len())]
pub fn parse_newline_model(
    input: Seq<u8>,
    mark: Int,
    cursor: Int,
    end: Int,
) -> NewlineResult {
    if cursor == end {
        NewlineResult {
            outcome: NewlineOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if input[cursor].deep_model() == 10 {
        NewlineResult {
            outcome: NewlineOutcome::Complete,
            mark: cursor + 1,
            cursor: cursor + 1,
        }
    } else if input[cursor].deep_model() == 13 {
        if cursor + 1 == end {
            NewlineResult {
                outcome: NewlineOutcome::Partial,
                mark,
                cursor: end,
            }
        } else if input[cursor + 1].deep_model() == 10 {
            NewlineResult {
                outcome: NewlineOutcome::Complete,
                mark: cursor + 2,
                cursor: cursor + 2,
            }
        } else {
            NewlineResult {
                outcome: NewlineOutcome::Error,
                mark,
                cursor: cursor + 2,
            }
        }
    } else {
        NewlineResult {
            outcome: NewlineOutcome::Error,
            mark,
            cursor: cursor + 1,
        }
    }
}
