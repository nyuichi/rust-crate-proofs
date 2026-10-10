#![cfg_attr(not(feature = "std"), no_std)]

extern crate creusot_std;

#[cfg(creusot)]
use creusot_std::prelude::{ensures, requires, DeepModel};

include!("../../../../src/invalid_chunk_size.rs");

pub mod config {
    use creusot_std::prelude::{ensures, logic, pearlite, View};
    include!("../../../../src/config.rs");
}
#[path = "../../../../src/chunk.rs"]
pub mod chunk;
pub mod status {
    use creusot_std::prelude::{ensures, requires};
    include!("../../../../src/status.rs");
}

#[cfg(creusot)]
#[path = "../../../../src/verification/chunk.rs"]
pub mod verification_chunk;

pub use chunk::parse_chunk_size;
pub use config::ParserConfig;
pub use status::Status;

/// Exercise the returned mutable borrow across several actual builder calls.
#[cfg(creusot)]
#[ensures(result@ == (false, true, false, false, false, false, false))]
pub fn chained_config_builder() -> ParserConfig {
    let mut config = ParserConfig::default();
    config
        .allow_spaces_after_header_name_in_responses(true)
        .allow_obsolete_multiline_headers_in_responses(true)
        .allow_spaces_after_header_name_in_responses(false);
    config
}

/// Exercise the actual one-byte runtime transition through its published
/// independent model contract.
#[cfg(creusot)]
#[requires(buf@.len() > 0)]
#[ensures(verification_chunk::steps_equal(
    result.deep_model(),
    verification_chunk::step(
        buf@,
        0,
        verification_chunk::ChunkState {
            size: 0,
            digits: 0,
            in_digits: true,
            in_extension: false,
        },
    ),
))]
pub(crate) fn first_chunk_step(buf: &[u8]) -> chunk::ChunkStep {
    chunk::step_chunk_size(
        buf,
        0,
        chunk::ChunkState {
            size: 0,
            digits: 0,
            in_digits: true,
            in_extension: false,
        },
    )
}
