#![allow(dead_code, unexpected_cfgs)]

// Use the real published component types and the checked-in Method and Error
// implementations. This leaf translates Error formatting without pulling in
// Request/Response builder contracts.
#[path = "../../../src/ascii.rs"]
pub(crate) mod ascii;

pub mod header {
    pub use http_runtime::header::{
        InvalidHeaderName, InvalidHeaderValue, MaxSizeReached,
    };
}

#[path = "../../../src/method.rs"]
pub mod method;

pub mod status {
    pub use http_runtime::status::InvalidStatusCode;
}

pub mod uri {
    pub use http_runtime::uri::{InvalidUri, InvalidUriParts};
}

#[path = "../../../src/error.rs"]
pub mod error;

pub use error::{Error, Result};
