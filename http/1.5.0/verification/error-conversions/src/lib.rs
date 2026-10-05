#![allow(dead_code, unexpected_cfgs)]

pub mod header {
    pub use http_runtime::header::{InvalidHeaderName, InvalidHeaderValue, MaxSizeReached};
}

pub mod method {
    pub use http_runtime::method::InvalidMethod;
}

pub mod status {
    pub use http_runtime::status::InvalidStatusCode;
}

#[path = "../../../src/uri/error.rs"]
pub mod uri_error;

pub mod uri {
    pub use super::uri_error::InvalidUri;
    pub use http_runtime::uri::InvalidUriParts;
}

// The concrete Error representation, conversion bodies, and empty-URI check
// are included directly from production. Trait-object formatting and downcast
// methods remain outside this leaf target.
#[path = "../../../src/error/core.rs"]
mod error_core;

// Optional translation-only probe of the outer production Error methods that
// use trait objects. It uses the same concrete payload modules as the core
// harness and never replaces the Error implementation.
#[cfg(error_dynamic_api)]
#[path = "../../../src/error.rs"]
mod error;
