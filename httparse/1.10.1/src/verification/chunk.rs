//! Independent state-machine specification for the published chunk-size parser.
//!
//! The model preserves the implementation's permissive behavior: zero hex
//! digits are accepted before CRLF, at most sixteen hex digits contribute to
//! the size, whitespace ends the digit run, and extension octets are ignored
//! except that CR must be followed by LF.

#[allow(unused_imports)]
use creusot_std::prelude::{logic, pearlite, requires, ensures, variant, Int, Seq};

/// Abstract outcome of scanning a chunk-size line.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ChunkOutcome {
    /// CRLF ended the line; `consumed` includes both newline bytes.
    Complete { consumed: Int, size: Int },
    /// The available bytes are a valid prefix but contain no complete CRLF.
    Partial,
    /// A byte or digit count makes the line invalid.
    Invalid,
}

/// Parser state after the already-consumed input prefix.
#[derive(Copy, Clone)]
struct ChunkState {
    size: Int,
    digits: Int,
    in_digits: bool,
    in_extension: bool,
}

/// Exclusive upper bound for a value represented by `digits` hexadecimal
/// digits (one for zero digits, then sixteen times the previous bound).
#[logic]
#[variant(digits)]
#[requires(0 <= digits && digits <= 16)]
fn hex_capacity(digits: Int) -> Int {
    if digits == 0 {
        1
    } else {
        16 * hex_capacity(digits - 1)
    }
}

/// Whether `byte` is an ASCII hexadecimal digit and its corresponding value.
#[logic(open)]
fn hex_value(byte: u8) -> Option<Int> {
    let b = byte@;
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
fn scan(input: Seq<u8>, pos: Int, state: ChunkState) -> ChunkOutcome {
    if pos == input.len() {
        ChunkOutcome::Partial
    } else {
        let byte = input[pos];
        match hex_value(byte) {
            Some(digit) if state.in_digits => {
                if state.digits > 15 {
                    ChunkOutcome::Invalid
                } else {
                    scan(
                        input,
                        pos + 1,
                        ChunkState {
                            size: state.size * 16 + digit,
                            digits: state.digits + 1,
                            in_digits: true,
                            in_extension: state.in_extension,
                        },
                    )
                }
            }
            _ => match byte {
                b'\r' => {
                    if pos + 1 == input.len() {
                        ChunkOutcome::Partial
                    } else if input[pos + 1] == b'\n' {
                        ChunkOutcome::Complete {
                            consumed: pos + 2,
                            size: state.size,
                        }
                    } else {
                        ChunkOutcome::Invalid
                    }
                }
                b';' if !state.in_extension => scan(
                    input,
                    pos + 1,
                    ChunkState {
                        size: state.size,
                        digits: state.digits,
                        in_digits: false,
                        in_extension: true,
                    },
                ),
                b' ' | b'\t' if !state.in_extension && !state.in_digits => {
                    scan(input, pos + 1, state)
                }
                b' ' | b'\t' if state.in_digits => scan(
                    input,
                    pos + 1,
                    ChunkState {
                        size: state.size,
                        digits: state.digits,
                        in_digits: false,
                        in_extension: false,
                    },
                ),
                _ if state.in_extension => scan(input, pos + 1, state),
                _ => ChunkOutcome::Invalid,
            },
        }
    }
}

/// Exact abstract result for `parse_chunk_size` on a complete input slice.
#[logic]
#[ensures(result == scan(
    input,
    0,
    ChunkState {
        size: 0,
        digits: 0,
        in_digits: true,
        in_extension: false,
    },
))]
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
