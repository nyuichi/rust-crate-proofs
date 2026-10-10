//! Independent exact model for the three-digit `parse_code` helper.
//!
//! The model follows the implementation's ASCII digit pattern and its
//! incremental `next()` cursor effects. It intentionally accepts every
//! three-digit value from 000 through 999; callers may impose narrower
//! protocol ranges separately.

#[allow(unused_imports)]
use creusot_std::prelude::{logic, requires, ensures, DeepModel, Int, Seq};

/// Result category of `parse_code`.
#[derive(Copy, Clone)]
pub enum CodeOutcome {
    /// Exactly three ASCII digits were consumed and converted to this value.
    Complete(Int),
    /// The input ended before three digits were read, after consuming every
    /// available valid digit.
    Partial,
    /// A non-ASCII-digit byte was consumed before returning the error.
    Error,
}

/// Exact output and cursor state after a `parse_code` call.
#[derive(Copy, Clone)]
pub struct CodeResult {
    /// Complete numeric value, partial input, or status-code error.
    pub outcome: CodeOutcome,
    /// Absolute mark before parsing; `parse_code` does not commit it.
    pub mark: Int,
    /// Absolute cursor after parsing.
    pub cursor: Int,
}

/// Whether the byte at `index` is in the implementation's `b'0'..=b'9'` range.
#[logic(open)]
#[requires(0 <= index && index < input.len())]
pub fn is_ascii_digit(input: Seq<u8>, index: Int) -> bool {
    let byte = input[index].deep_model();
    byte >= 48 && byte <= 57
}

/// Decimal value of a byte already known to be an ASCII digit.
#[logic(open)]
#[requires(0 <= index && index < input.len())]
#[requires(is_ascii_digit(input, index))]
#[ensures(result == input[index].deep_model() - 48)]
pub fn decimal_digit_value(input: Seq<u8>, index: Int) -> Int {
    input[index].deep_model() - 48
}

/// Whether the model returns a complete value equal to `value`.
#[logic(open)]
pub fn code_result_is_complete(parsed: CodeResult, value: Int) -> bool {
    match parsed.outcome {
        CodeOutcome::Complete(actual) => actual == value,
        _ => false,
    }
}

/// Whether the model returns partial input.
#[logic(open)]
pub fn code_result_is_partial(parsed: CodeResult) -> bool {
    match parsed.outcome {
        CodeOutcome::Partial => true,
        _ => false,
    }
}

/// Whether the model returns a status-code error.
#[logic(open)]
pub fn code_result_is_error(parsed: CodeResult) -> bool {
    match parsed.outcome {
        CodeOutcome::Error => true,
        _ => false,
    }
}

/// Exact model of the current `parse_code` implementation.
///
/// It reads exactly three ASCII decimal digits, if present. EOF produces
/// `Partial` without advancing beyond the available input. A bad byte is
/// consumed before `Error::Status`. Three digits produce 0 through 999; this
/// helper itself has no 100..=599 protocol-range restriction and leaves any
/// following byte untouched.
#[logic(open)]
#[requires(0 <= mark && mark <= start)]
#[requires(start <= end && end <= input.len())]
#[ensures(result.mark == mark)]
#[ensures(start <= result.cursor && result.cursor <= end)]
#[ensures(result.cursor <= start + 3)]
#[ensures(match result.outcome {
    CodeOutcome::Complete(value) => 0 <= value && value <= 999,
    _ => true,
})]
#[ensures(start + 2 < end && is_ascii_digit(input, start)
    && is_ascii_digit(input, start + 1) && is_ascii_digit(input, start + 2) ==>
    code_result_is_complete(result,
        decimal_digit_value(input, start) * 100
            + decimal_digit_value(input, start + 1) * 10
            + decimal_digit_value(input, start + 2))
        && result.cursor == start + 3)]
pub fn parse_code_model(
    input: Seq<u8>,
    mark: Int,
    start: Int,
    end: Int,
) -> CodeResult {
    if start == end {
        CodeResult {
            outcome: CodeOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if !is_ascii_digit(input, start) {
        CodeResult {
            outcome: CodeOutcome::Error,
            mark,
            cursor: start + 1,
        }
    } else if start + 1 == end {
        CodeResult {
            outcome: CodeOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if !is_ascii_digit(input, start + 1) {
        CodeResult {
            outcome: CodeOutcome::Error,
            mark,
            cursor: start + 2,
        }
    } else if start + 2 == end {
        CodeResult {
            outcome: CodeOutcome::Partial,
            mark,
            cursor: end,
        }
    } else if !is_ascii_digit(input, start + 2) {
        CodeResult {
            outcome: CodeOutcome::Error,
            mark,
            cursor: start + 3,
        }
    } else {
        let hundreds = decimal_digit_value(input, start);
        let tens = decimal_digit_value(input, start + 1);
        let ones = decimal_digit_value(input, start + 2);
        CodeResult {
            outcome: CodeOutcome::Complete(hundreds * 100 + tens * 10 + ones),
            mark,
            cursor: start + 3,
        }
    }
}
