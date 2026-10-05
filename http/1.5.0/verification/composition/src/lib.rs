#![allow(dead_code, unexpected_cfgs)]

// The external component values are imported from the real `http` runtime
// crate. Request/Response and Method use the checked-in production sources.
// The named URI-default leaf profile substitutes the actual production URI
// module to expose its already-proved default-field model in Parts::new.
pub use http_runtime::Extensions;
#[cfg(not(http_composition_uri_defaults_leaf))]
pub use http_runtime::Uri;
#[cfg(not(http_composition_uri_defaults_leaf))]
pub use http_runtime::{StatusCode, Version};

#[path = "../../../src/ascii.rs"]
pub(crate) mod ascii;

#[cfg(http_composition_uri_defaults_leaf)]
#[path = "../../../src/byte_str.rs"]
pub(crate) mod byte_str;
#[cfg(http_composition_uri_defaults_leaf)]
#[path = "../../../src/bytes_model.rs"]
pub(crate) mod bytes_model;

pub mod header {
    pub use http_runtime::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue, MaxSizeReached};
}

#[path = "../../../src/method.rs"]
pub mod method;

#[cfg(not(http_composition_uri_defaults_leaf))]
pub mod status {
    pub use http_runtime::status::InvalidStatusCode;
    pub use http_runtime::StatusCode;
}
#[cfg(http_composition_uri_defaults_leaf)]
#[path = "../../../src/status.rs"]
pub mod status;
#[cfg(http_composition_uri_defaults_leaf)]
pub use status::StatusCode;

#[cfg(not(http_composition_uri_defaults_leaf))]
pub mod uri {
    pub use http_runtime::uri::*;
}
#[cfg(http_composition_uri_defaults_leaf)]
#[path = "../../../src/uri/mod.rs"]
pub mod uri;
#[cfg(http_composition_uri_defaults_leaf)]
pub use uri::Uri;

#[cfg(not(http_error_fmt_leaf))]
#[path = "../../../src/error/core.rs"]
mod error_core;
#[cfg(not(http_error_fmt_leaf))]
pub use error_core::{Error, Result};
#[cfg(all(creusot, not(http_error_fmt_leaf)))]
pub use error_core::{error_model, error_model_ref, ErrorModel, ErrorModelRef};
#[cfg(http_error_fmt_leaf)]
#[path = "../../../src/error.rs"]
mod error;
#[cfg(http_error_fmt_leaf)]
pub use error::{Error, Result};

#[cfg(not(http_composition_uri_defaults_leaf))]
pub mod version {
    pub use http_runtime::Version;
}
#[cfg(http_composition_uri_defaults_leaf)]
#[path = "../../../src/version.rs"]
pub mod version;
#[cfg(http_composition_uri_defaults_leaf)]
pub use version::Version;

#[cfg(not(http_error_fmt_leaf))]
#[path = "../../../src/request.rs"]
pub mod request;

#[cfg(not(http_error_fmt_leaf))]
#[path = "../../../src/response.rs"]
pub mod response;
