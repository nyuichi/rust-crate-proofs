//! Independent state-machine specification for the published chunk-size parser.
//!
//! The model preserves the implementation's permissive behavior: zero hex
//! digits are accepted before CRLF, at most sixteen hex digits contribute to
//! the size, whitespace ends the digit run, and extension octets are ignored
//! except that CR must be followed by LF.

#[allow(unused_imports)]
use creusot_std::prelude::{check, logic, pearlite, requires, ensures, variant, DeepModel, Int, Seq, Snapshot};

/// Abstract outcome of scanning a chunk-size line.
#[derive(Copy, Clone)]
pub enum ChunkOutcome {
    /// CRLF ended the line; `consumed` includes both newline bytes.
    Complete { consumed: Int, size: Int },
    /// The available bytes are a valid prefix but contain no complete CRLF.
    Partial,
    /// A byte or digit count makes the line invalid.
    Invalid,
}

/// Logical equality for outcomes containing Creusot `Int` values.
#[logic(open)]
pub fn outcomes_equal(left: ChunkOutcome, right: ChunkOutcome) -> bool {
    match (left, right) {
        (
            ChunkOutcome::Complete {
                consumed: left_consumed,
                size: left_size,
            },
            ChunkOutcome::Complete {
                consumed: right_consumed,
                size: right_size,
            },
        ) => left_consumed == right_consumed && left_size == right_size,
        (ChunkOutcome::Partial, ChunkOutcome::Partial) => true,
        (ChunkOutcome::Invalid, ChunkOutcome::Invalid) => true,
        _ => false,
    }
}

/// Parser state after the already-consumed input prefix.
#[derive(Copy, Clone)]
pub struct ChunkState {
    pub size: Int,
    pub digits: Int,
    pub in_digits: bool,
    pub in_extension: bool,
}

/// Result of consuming one byte from the chunk-size line.
///
/// A CR transition may inspect and consume the following LF in the same
/// step. All other continuing transitions advance exactly one byte.
#[derive(Copy, Clone)]
pub enum ChunkStep {
    Continue { next_pos: Int, next: ChunkState },
    Complete { consumed: Int, size: Int },
    Partial,
    Invalid,
}

#[logic(open)]
pub fn states_equal(left: ChunkState, right: ChunkState) -> bool {
    left.size == right.size
        && left.digits == right.digits
        && left.in_digits == right.in_digits
        && left.in_extension == right.in_extension
}

#[logic(open)]
pub fn steps_equal(left: ChunkStep, right: ChunkStep) -> bool {
    match (left, right) {
        (
            ChunkStep::Continue {
                next_pos: left_pos,
                next: left_state,
            },
            ChunkStep::Continue {
                next_pos: right_pos,
                next: right_state,
            },
        ) => left_pos == right_pos && states_equal(left_state, right_state),
        (
            ChunkStep::Complete {
                consumed: left_consumed,
                size: left_size,
            },
            ChunkStep::Complete {
                consumed: right_consumed,
                size: right_size,
            },
        ) => left_consumed == right_consumed && left_size == right_size,
        (ChunkStep::Partial, ChunkStep::Partial) => true,
        (ChunkStep::Invalid, ChunkStep::Invalid) => true,
        _ => false,
    }
}

/// Exclusive upper bound for a value represented by `digits` hexadecimal
/// digits (one for zero digits, then sixteen times the previous bound).
#[logic]
#[variant(digits)]
#[requires(0 <= digits && digits <= 16)]
#[ensures(result >= 1)]
pub fn hex_capacity(digits: Int) -> Int {
    if digits == 0 {
        1
    } else {
        16 * hex_capacity(digits - 1)
    }
}

/// Whether `byte` is an ASCII hexadecimal digit and its corresponding value.
#[logic(open)]
#[ensures(match result {
    Some(value) => 0 <= value && value <= 15,
    None => true,
})]
pub fn hex_value(byte: u8) -> Option<Int> {
    let b = byte.deep_model();
    if b >= 48 && b <= 57 {
        Some(b - 48)
    } else if b >= 97 && b <= 102 {
        Some(b - 87)
    } else if b >= 65 && b <= 70 {
        Some(b - 55)
    } else {
        None
    }
}

/// Consume exactly one ordinary byte, or a CRLF terminator, from `pos`.
#[logic(open)]
#[requires(0 <= pos && pos < input.len())]
#[requires(0 <= state.digits && state.digits <= 16)]
#[requires(0 <= state.size && state.size < hex_capacity(state.digits))]
#[requires(!(state.in_digits && state.in_extension))]
#[ensures(match result {
    ChunkStep::Continue { next_pos, next } =>
        next_pos == pos + 1
        && next_pos <= input.len()
        && 0 <= next.digits
        && next.digits <= 16
        && 0 <= next.size
        && next.size < hex_capacity(next.digits)
        && (next.digits < 16 ==> next.size <= u64::MAX@ / 16)
        && !(next.in_digits && next.in_extension),
    ChunkStep::Complete { consumed, size } =>
        pos < consumed && consumed <= input.len()
        && 0 <= size && size <= u64::MAX@,
    ChunkStep::Partial => pos < input.len(),
    ChunkStep::Invalid => true,
})]
pub fn step(input: Seq<u8>, pos: Int, state: ChunkState) -> ChunkStep {
    let byte = input[pos];
    match hex_value(byte) {
        Some(digit) => {
            if state.in_digits {
                if state.digits > 15 {
                    ChunkStep::Invalid
                } else {
                    ChunkStep::Continue {
                        next_pos: pos + 1,
                        next: ChunkState {
                            size: state.size * 16 + digit,
                            digits: state.digits + 1,
                            in_digits: true,
                            in_extension: state.in_extension,
                        },
                    }
                }
            } else if state.in_extension {
                ChunkStep::Continue {
                    next_pos: pos + 1,
                    next: state,
                }
            } else {
                ChunkStep::Invalid
            }
        }
        None => {
            let b = byte.deep_model();
            if b == 13 {
                if pos + 1 == input.len() {
                    ChunkStep::Partial
                } else if input[pos + 1].deep_model() == 10 {
                    ChunkStep::Complete {
                        consumed: pos + 2,
                        size: state.size,
                    }
                } else {
                    ChunkStep::Invalid
                }
            } else if b == 59 {
                if state.in_extension {
                    ChunkStep::Continue {
                        next_pos: pos + 1,
                        next: state,
                    }
                } else {
                    ChunkStep::Continue {
                        next_pos: pos + 1,
                        next: ChunkState {
                            size: state.size,
                            digits: state.digits,
                            in_digits: false,
                            in_extension: true,
                        },
                    }
                }
            } else if b == 32 || b == 9 {
                if state.in_extension || !state.in_digits {
                    ChunkStep::Continue {
                        next_pos: pos + 1,
                        next: state,
                    }
                } else {
                    ChunkStep::Continue {
                        next_pos: pos + 1,
                        next: ChunkState {
                            size: state.size,
                            digits: state.digits,
                            in_digits: false,
                            in_extension: false,
                        },
                    }
                }
            } else if state.in_extension {
                ChunkStep::Continue {
                    next_pos: pos + 1,
                    next: state,
                }
            } else {
                ChunkStep::Invalid
            }
        }
    }
}

/// Deterministic model from one parser state to the exact terminal result.
///
/// `pos` is the next unread absolute input offset. A CR at the final available
/// position yields Partial because the runtime's following `next!` sees EOF.
#[logic]
#[variant(input.len() - pos)]
#[requires(0 <= pos && pos <= input.len())]
#[requires(0 <= state.digits && state.digits <= 16)]
#[requires(0 <= state.size && state.size < hex_capacity(state.digits))]
#[requires(!(state.in_digits && state.in_extension))]
#[ensures(match result {
    ChunkOutcome::Complete { consumed, size } =>
        pos < consumed && consumed <= input.len() && 0 <= size && size <= u64::MAX@,
    ChunkOutcome::Partial => pos <= input.len(),
    ChunkOutcome::Invalid => true,
})]
pub fn scan(input: Seq<u8>, pos: Int, state: ChunkState) -> ChunkOutcome {
    if pos == input.len() {
        ChunkOutcome::Partial
    } else {
        match step(input, pos, state) {
            ChunkStep::Continue { next_pos, next } => scan(input, next_pos, next),
            ChunkStep::Complete { consumed, size } => ChunkOutcome::Complete { consumed, size },
            ChunkStep::Partial => ChunkOutcome::Partial,
            ChunkStep::Invalid => ChunkOutcome::Invalid,
        }
    }
}

/// Exact abstract result for `parse_chunk_size` on a complete input slice.
#[logic]
pub fn parse_chunk_size_model(input: Seq<u8>) -> ChunkOutcome {
    scan(
        input,
        0,
        ChunkState {
            size: 0,
            digits: 0,
            in_digits: true,
            in_extension: false,
        },
    )
}

/// Connect the parser's initial runtime model state to its published model result.
#[check(ghost)]
#[ensures(outcomes_equal(
    scan(
        *input,
        0,
        ChunkState {
            size: 0,
            digits: 0,
            in_digits: true,
            in_extension: false,
        },
    ),
    parse_chunk_size_model(*input),
))]
pub fn scan_initial(input: Snapshot<Seq<u8>>) {}

/// Give the exact model result when the cursor reaches the input end.
#[check(ghost)]
#[requires(0 <= *pos && *pos <= input.len())]
#[requires(*pos == input.len())]
#[requires(0 <= (*state).digits && (*state).digits <= 16)]
#[requires(0 <= (*state).size && (*state).size < hex_capacity((*state).digits))]
#[requires(!((*state).in_digits && (*state).in_extension))]
#[ensures(outcomes_equal(
    scan(*input, *pos, *state),
    ChunkOutcome::Partial,
))]
pub fn scan_eof(
    input: Snapshot<Seq<u8>>,
    pos: Snapshot<Int>,
    state: Snapshot<ChunkState>,
) {}

/// Relate one recursive model scan to exactly one transition.
#[check(ghost)]
#[requires(0 <= *pos && *pos < input.len())]
#[requires(0 <= (*state).digits && (*state).digits <= 16)]
#[requires(0 <= (*state).size && (*state).size < hex_capacity((*state).digits))]
#[requires(!((*state).in_digits && (*state).in_extension))]
#[ensures(match step(*input, *pos, *state) {
    ChunkStep::Continue { next_pos, next } => outcomes_equal(
        scan(*input, *pos, *state),
        scan(*input, next_pos, next),
    ),
    ChunkStep::Complete { consumed, size } => outcomes_equal(
        scan(*input, *pos, *state),
        ChunkOutcome::Complete { consumed, size },
    ),
    ChunkStep::Partial => outcomes_equal(
        scan(*input, *pos, *state),
        ChunkOutcome::Partial,
    ),
    ChunkStep::Invalid => outcomes_equal(
        scan(*input, *pos, *state),
        ChunkOutcome::Invalid,
    ),
})]
pub fn scan_unfold(
    input: Snapshot<Seq<u8>>,
    pos: Snapshot<Int>,
    state: Snapshot<ChunkState>,
) {}
