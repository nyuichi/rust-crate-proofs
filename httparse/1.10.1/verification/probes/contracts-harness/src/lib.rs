#![cfg_attr(not(feature = "std"), no_std)]

extern crate creusot_std;

#[cfg(creusot)]
use creusot_std::prelude::ensures;

#[path = "../../../../src/config.rs"]
pub mod config;
#[path = "../../../../src/status.rs"]
pub mod status;

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
