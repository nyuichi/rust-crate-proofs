// Chunk-size line parser; this shared file is included in the crate root.

#[cfg(not(creusot))]
use core::fmt;
use core::result;

#[allow(unused_imports)]
use creusot_std::prelude::{ghost, snapshot, ensures, invariant, logic, pearlite, requires, variant, DeepModel};

/// Runtime state after parsing the consumed prefix.
#[cfg_attr(not(creusot), derive(Copy, Clone))]
#[cfg_attr(creusot, derive(Copy, Clone))]
pub(crate) struct ChunkState {
    pub size: u64,
    pub digits: i32,
    pub in_digits: bool,
    pub in_extension: bool,
}

/// Result of consuming one input byte (or a CRLF pair).
pub(crate) enum ChunkStep {
    Continue { next_pos: usize, next: ChunkState },
    Complete { consumed: usize, size: u64 },
    Partial,
    Invalid,
}

#[cfg(creusot)]
impl DeepModel for ChunkState {
    type DeepModelTy = crate::verification_chunk::ChunkState;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! {
            crate::verification_chunk::ChunkState {
                size: self.size@,
                digits: self.digits@,
                in_digits: self.in_digits,
                in_extension: self.in_extension,
            }
        }
    }
}

#[cfg(creusot)]
impl DeepModel for ChunkStep {
    type DeepModelTy = crate::verification_chunk::ChunkStep;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! {
            match self {
                ChunkStep::Continue { next_pos, next } =>
                    crate::verification_chunk::ChunkStep::Continue {
                        next_pos: next_pos@,
                        next: next.deep_model(),
                    },
                ChunkStep::Complete { consumed, size } =>
                    crate::verification_chunk::ChunkStep::Complete {
                        consumed: consumed@,
                        size: size@,
                    },
                ChunkStep::Partial => crate::verification_chunk::ChunkStep::Partial,
                ChunkStep::Invalid => crate::verification_chunk::ChunkStep::Invalid,
            }
        }
    }
}

/// Normalize all three hexadecimal digit ranges to one nibble value.
#[ensures(match result {
    Some(nibble) => match crate::verification_chunk::hex_value(byte) {
        Some(expected) => nibble@ == expected && nibble@ <= 15,
        None => false,
    },
    None => match crate::verification_chunk::hex_value(byte) {
        Some(_) => false,
        None => true,
    },
})]
fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Append one normalized hexadecimal nibble to the current size.
#[requires(0 <= state.digits@ && state.digits@ < 16)]
#[requires(0 <= state.size@ && state.size@ < crate::verification_chunk::hex_capacity(state.digits@))]
#[requires(state.size@ <= u64::MAX@ / 16)]
#[requires(nibble@ <= 15)]
#[ensures(result.size@ == state.size@ * 16 + nibble@)]
#[ensures(result.digits@ == state.digits@ + 1)]
#[ensures(result.in_digits == state.in_digits)]
#[ensures(result.in_extension == state.in_extension)]
fn append_hex_digit(state: ChunkState, nibble: u8) -> ChunkState {
    ChunkState {
        size: state.size * 16 + nibble as u64,
        digits: state.digits + 1,
        in_digits: state.in_digits,
        in_extension: state.in_extension,
    }
}

/// Consume one byte, or a CRLF terminator, using the published parser rules.
#[requires(cursor@ < buf@.len())]
#[requires(0 <= state.digits@ && state.digits@ <= 16)]
#[requires(0 <= state.size@ && state.size@ < crate::verification_chunk::hex_capacity(state.digits@))]
#[requires(state.digits@ == 16 || state.size@ <= u64::MAX@ / 16)]
#[requires(!(state.in_digits && state.in_extension))]
#[ensures(crate::verification_chunk::steps_equal(
    result.deep_model(),
    crate::verification_chunk::step(buf@, cursor@, state.deep_model()),
))]
pub(crate) fn step_chunk_size(buf: &[u8], cursor: usize, state: ChunkState) -> ChunkStep {
    const RADIX: u64 = 16;
    let b = buf[cursor];

    if let Some(nibble) = hex_nibble(b) {
        if state.in_digits {
            if state.digits > 15 {
                return ChunkStep::Invalid;
            }
            if cfg!(debug_assertions) && state.size > (u64::MAX / RADIX) {
                return ChunkStep::Invalid;
            }
            return ChunkStep::Continue {
                next_pos: cursor + 1,
                next: append_hex_digit(state, nibble),
            };
        }
        if state.in_extension {
            return ChunkStep::Continue {
                next_pos: cursor + 1,
                next: state,
            };
        }
        return ChunkStep::Invalid;
    }

    if b == b'\r' {
        if cursor + 1 == buf.len() {
            return ChunkStep::Partial;
        }
        if buf[cursor + 1] == b'\n' {
            return ChunkStep::Complete {
                consumed: cursor + 2,
                size: state.size,
            };
        }
        return ChunkStep::Invalid;
    }

    if b == b';' {
        if state.in_extension {
            return ChunkStep::Continue {
                next_pos: cursor + 1,
                next: state,
            };
        }
        return ChunkStep::Continue {
            next_pos: cursor + 1,
            next: ChunkState {
                size: state.size,
                digits: state.digits,
                in_digits: false,
                in_extension: true,
            },
        };
    }

    if b == b'\t' || b == b' ' {
        if state.in_extension || !state.in_digits {
            return ChunkStep::Continue {
                next_pos: cursor + 1,
                next: state,
            };
        }
        return ChunkStep::Continue {
            next_pos: cursor + 1,
            next: ChunkState {
                size: state.size,
                digits: state.digits,
                in_digits: false,
                in_extension: false,
            },
        };
    }

    if state.in_extension {
        ChunkStep::Continue {
            next_pos: cursor + 1,
            next: state,
        }
    } else {
        ChunkStep::Invalid
    }
}

#[cfg(not(creusot))]
impl fmt::Display for crate::InvalidChunkSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid chunk size")
    }
}

/// Parse a buffer of bytes as a chunk size.
///
/// The return value, if complete and successful, includes the index of the
/// buffer that parsing stopped at, and the size of the following chunk.
///
/// # Example
///
/// ```
/// let buf = b"4\r\nRust\r\n0\r\n\r\n";
/// assert_eq!(httparse::parse_chunk_size(buf),
///            Ok(httparse::Status::Complete((3, 4))));
/// // The upstream parser also accepts zero hex digits before the CRLF.
/// assert_eq!(httparse::parse_chunk_size(b"\r\n"),
///            Ok(httparse::Status::Complete((2, 0))));
/// ```
#[ensures(match result {
    Ok(crate::Status::Complete((consumed, size))) => match crate::verification_chunk::parse_chunk_size_model(buf@) {
        crate::verification_chunk::ChunkOutcome::Complete {
            consumed: expected_consumed,
            size: expected_size,
        } =>
            consumed@ == expected_consumed && size@ == expected_size,
        _ => false,
    },
    Ok(crate::Status::Partial) => crate::verification_chunk::outcomes_equal(
        crate::verification_chunk::parse_chunk_size_model(buf@),
        crate::verification_chunk::ChunkOutcome::Partial,
    ),
    Err(_) => crate::verification_chunk::outcomes_equal(
        crate::verification_chunk::parse_chunk_size_model(buf@),
        crate::verification_chunk::ChunkOutcome::Invalid,
    ),
})]
pub fn parse_chunk_size(buf: &[u8])
    -> result::Result<crate::Status<(usize, u64)>, crate::InvalidChunkSize> {
    let mut cursor = 0;
    let mut state = ChunkState {
        size: 0,
        digits: 0,
        in_digits: true,
        in_extension: false,
    };

    #[cfg(creusot)]
    ghost! {
        crate::verification_chunk::scan_initial(snapshot!(buf@));
    };

    #[invariant(cursor@ <= buf@.len())]
    #[invariant(0 <= state.digits@ && state.digits@ <= 16)]
    #[invariant(!(state.in_digits && state.in_extension))]
    #[invariant(state.digits@ < 16 ==> state.size@ <= u64::MAX@ / 16)]
    #[invariant(0 <= state.size@ && state.size@ < crate::verification_chunk::hex_capacity(state.digits@))]
    #[invariant(crate::verification_chunk::outcomes_equal(
        crate::verification_chunk::scan(
            buf@,
            cursor@,
            state.deep_model(),
        ),
        crate::verification_chunk::parse_chunk_size_model(buf@),
    ))]
    #[variant(buf@.len() - cursor@)]
    while cursor < buf.len() {
        #[cfg(creusot)]
        ghost! {
            crate::verification_chunk::scan_unfold(
                snapshot!(buf@),
                snapshot!(cursor@),
                snapshot!(state.deep_model()),
            );
        };
        match step_chunk_size(buf, cursor, state) {
            ChunkStep::Continue { next_pos, next } => {
                cursor = next_pos;
                state = next;
            }
            ChunkStep::Complete { consumed, size } => {
                return Ok(crate::Status::Complete((consumed, size)));
            }
            ChunkStep::Partial => return Ok(crate::Status::Partial),
            ChunkStep::Invalid => return Err(crate::InvalidChunkSize),
        }
    }
    #[cfg(creusot)]
    ghost! {
        crate::verification_chunk::scan_eof(
            snapshot!(buf@),
            snapshot!(cursor@),
            snapshot!(state.deep_model()),
        );
    };
    Ok(crate::Status::Partial)
}
