//! Independent logical specifications for byte classes and scalar spans.
//!
//! This module deliberately models the grammar from byte values instead of
//! referring to the runtime lookup tables or scanner implementations.  It is
//! the first leaf of the parser model; stage and public-state outcomes are
//! added only after these predicates and spans are available to callers.

#![allow(dead_code)]

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, variant, DeepModel, Int, Seq};

/// Byte class understood by the scalar parser model.
#[derive(Copy, Clone)]
pub enum ByteClass {
    /// RFC `tchar`, used for methods and header names.
    Token,
    /// Runtime URI byte rule: visible ASCII except DEL, plus obs-text.
    Uri,
    /// Runtime header-value byte rule: HTAB, visible ASCII, or obs-text.
    HeaderValue,
}

/// Whether `byte` is accepted by the specified scalar byte class.
///
/// This is an arithmetic definition and does not depend on `TOKEN_MAP`,
/// `URI_MAP`, `HEADER_VALUE_MAP`, or any SIMD/SWAR mask.
#[logic(open)]
pub fn accepts(class: ByteClass, byte: u8) -> bool {
    match class {
        ByteClass::Token => pearlite! {
            (byte@ >= 48 && byte@ <= 57)
                || (byte@ >= 65 && byte@ <= 90)
                || (byte@ >= 97 && byte@ <= 122)
                || byte@ == 33 || byte@ == 35 || byte@ == 36 || byte@ == 37 || byte@ == 38
                || byte@ == 39 || byte@ == 42 || byte@ == 43 || byte@ == 45 || byte@ == 46
                || byte@ == 94 || byte@ == 95 || byte@ == 96 || byte@ == 124 || byte@ == 126
        },
        ByteClass::Uri => pearlite! { byte@ >= 33 && byte@ != 127 },
        ByteClass::HeaderValue => pearlite! {
            byte@ == 9 || (byte@ >= 32 && byte@ != 127)
        },
    }
}

/// Scalar `tchar` predicate used by methods and header names.
#[logic(open)]
pub fn is_tchar(byte: u8) -> bool {
    accepts(ByteClass::Token, byte)
}

/// Scalar URI predicate used before UTF-8 validation.
#[logic(open)]
pub fn is_uri_byte(byte: u8) -> bool {
    accepts(ByteClass::Uri, byte)
}

/// Scalar header-value predicate; HTAB is accepted for later trimming.
#[logic(open)]
pub fn is_header_value_byte(byte: u8) -> bool {
    accepts(ByteClass::HeaderValue, byte)
}

/// Exact end of the maximal accepted prefix of `input[start..end]`.
///
/// `result` is either `end` or the first byte not accepted by `class`; thus
/// callers can compose this scalar result with a conservative vector prefix
/// without assuming any runtime mask is exact.
#[logic]
#[variant(end - start)]
#[requires(0 <= start && start <= end && end <= input.len())]
#[ensures(start <= result && result <= end)]
#[ensures(forall<i: Int> start <= i && i < result ==> accepts(class, input[i]))]
#[ensures(result < end ==> !accepts(class, input[result]))]
pub fn maximal_prefix_end(input: Seq<u8>, start: Int, end: Int, class: ByteClass) -> Int {
    if start == end || !accepts(class, input[start]) {
        start
    } else {
        maximal_prefix_end(input, start + 1, end, class)
    }
}

/// Representative scalar scanner caller which packages the maximal prefix
/// as an absolute input span.
#[logic]
#[requires(0 <= start && start <= end && end <= input.len())]
#[ensures(result.start == start)]
#[ensures(result.end == maximal_prefix_end(input, start, end, class))]
#[ensures(valid_span(result, input.len()))]
#[ensures(forall<i: Int> result.start <= i && i < result.end ==> accepts(class, input[i]))]
#[ensures(result.end < end ==> !accepts(class, input[result.end]))]
pub fn accepted_prefix_span(
    input: Seq<u8>,
    start: Int,
    end: Int,
    class: ByteClass,
) -> Span {
    let span_end = maximal_prefix_end(input, start, end, class);
    Span {
        start,
        end: span_end,
    }
}

/// Deterministic outcomes for the token scanner model.
#[derive(Copy, Clone)]
pub enum TokenOutcome {
    /// A nonempty token ended at a space delimiter.
    Complete,
    /// The input ended before a space delimiter.
    Partial,
    /// The first byte or a later non-space byte was not a token byte.
    Error,
}

#[logic(open)]
pub fn token_outcome_is_complete(outcome: TokenOutcome) -> bool {
    match outcome {
        TokenOutcome::Complete => true,
        _ => false,
    }
}

#[logic(open)]
pub fn token_outcome_is_partial(outcome: TokenOutcome) -> bool {
    match outcome {
        TokenOutcome::Partial => true,
        _ => false,
    }
}

#[logic(open)]
pub fn token_outcome_is_error(outcome: TokenOutcome) -> bool {
    match outcome {
        TokenOutcome::Error => true,
        _ => false,
    }
}

/// Exact abstract result of `parse_token` when its mark equals its cursor.
#[derive(Copy, Clone)]
pub struct TokenResult {
    /// Complete, partial, or invalid token outcome.
    pub outcome: TokenOutcome,
    /// Absolute cursor after the operation, including the delimiter or bad
    /// byte consumed by `next()` when one exists.
    pub cursor: Int,
    /// Token bytes on complete success. The trailing space is excluded.
    pub token: Option<Span>,
}

#[logic(open)]
pub fn token_result_has_no_span(parsed: TokenResult) -> bool {
    match parsed.token {
        None => true,
        Some(_) => false,
    }
}

#[logic(open)]
pub fn token_result_span_is(parsed: TokenResult, start: Int, end: Int) -> bool {
    match parsed.token {
        None => false,
        Some(span) => span.start == start && span.end == end,
    }
}

/// Independent deterministic model of the scalar `parse_token` behavior.
///
/// This is a representative caller of `maximal_prefix_end`: after proving the
/// first byte is a token, it distinguishes end-of-input, a space delimiter,
/// and an invalid byte while retaining the exact consumed cursor.
#[logic]
#[requires(0 <= start && start <= end && end <= input.len())]
#[ensures(start == end ==> token_outcome_is_partial(result.outcome))]
#[ensures(start == end ==> result.cursor == end && token_result_has_no_span(result))]
#[ensures(start < end && !accepts(ByteClass::Token, input[start])
    ==> token_outcome_is_error(result.outcome))]
#[ensures(start < end && !accepts(ByteClass::Token, input[start])
    ==> result.cursor == start + 1 && token_result_has_no_span(result))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) == end
    ==> token_outcome_is_partial(result.outcome))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) == end
    ==> result.cursor == end && token_result_has_no_span(result))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) < end
    && input[maximal_prefix_end(input, start, end, ByteClass::Token)]@ == 32
    ==> token_outcome_is_complete(result.outcome))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) < end
    && input[maximal_prefix_end(input, start, end, ByteClass::Token)]@ == 32
    ==> result.cursor == maximal_prefix_end(input, start, end, ByteClass::Token) + 1)]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) < end
    && input[maximal_prefix_end(input, start, end, ByteClass::Token)]@ == 32
    ==> token_result_span_is(result, start,
        maximal_prefix_end(input, start, end, ByteClass::Token)))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) < end
    && input[maximal_prefix_end(input, start, end, ByteClass::Token)]@ != 32
    ==> token_outcome_is_error(result.outcome))]
#[ensures(start < end && accepts(ByteClass::Token, input[start])
    && maximal_prefix_end(input, start, end, ByteClass::Token) < end
    && input[maximal_prefix_end(input, start, end, ByteClass::Token)]@ != 32
    ==> result.cursor == maximal_prefix_end(input, start, end, ByteClass::Token) + 1
        && token_result_has_no_span(result))]
pub fn parse_token_model(input: Seq<u8>, start: Int, end: Int) -> TokenResult {
    if start == end {
        TokenResult {
            outcome: TokenOutcome::Partial,
            cursor: end,
            token: None,
        }
    } else if !accepts(ByteClass::Token, input[start]) {
        TokenResult {
            outcome: TokenOutcome::Error,
            cursor: start + 1,
            token: None,
        }
    } else {
        let token_end = maximal_prefix_end(input, start, end, ByteClass::Token);
        if token_end == end {
            TokenResult {
                outcome: TokenOutcome::Partial,
                cursor: end,
                token: None,
            }
        } else if input[token_end].deep_model() == 32 {
            TokenResult {
                outcome: TokenOutcome::Complete,
                cursor: token_end + 1,
                token: Some(Span {
                    start,
                    end: token_end,
                }),
            }
        } else {
            TokenResult {
                outcome: TokenOutcome::Error,
                cursor: token_end + 1,
                token: None,
            }
        }
    }
}

/// The runtime parser's absolute input span corresponding to a byte range.
///
/// `Bytes::pos()` is relative to the current mark.  Parser models keep absolute
/// indices so a returned span can be related directly to the immutable input.
#[derive(Copy, Clone)]
pub struct Span {
    /// Absolute first byte, inclusive.
    pub start: Int,
    /// Absolute byte after the span, exclusive.
    pub end: Int,
}

/// A span lies inside the original input and has ordered endpoints.
#[logic(open)]
pub fn valid_span(span: Span, input_len: Int) -> bool {
    pearlite! { 0 <= span.start && span.start <= span.end && span.end <= input_len }
}
