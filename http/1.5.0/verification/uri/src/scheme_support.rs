#[path = "../../../src/uri/error.rs"]
mod error;

use error::ErrorKind;
pub(crate) use error::InvalidUri;

// Include the exact production predicate so the production path.rs
// precondition is available through this source-leaf harness module.
#[cfg(creusot)]
#[path = "../../../src/uri/path_static_domain.rs"]
mod path_static_domain;

#[cfg(creusot)]
pub use self::path_static_domain::{
    path_static_input_is_valid, path_static_path_byte_valid, path_static_query_byte_valid,
};

#[path = "../../../src/uri/limits.rs"]
mod limits;
use limits::MAX_LEN;

#[path = "../../../src/uri/port.rs"]
pub(crate) mod port;
pub(crate) use port::Port;

#[path = "../../../src/uri/authority_chars.rs"]
pub(crate) mod authority_chars;

#[path = "../../../src/uri/path.rs"]
pub(crate) mod path;

#[path = "../../../src/uri/authority.rs"]
pub(crate) mod authority;

#[cfg(creusot)]
pub use self::authority::{
    authority_error_matches_rejection, authority_first_byte_from,
    authority_host_end, authority_host_input_is_safe, authority_host_start,
    authority_input_is_fully_valid, authority_port_number, authority_port_start,
    authority_static_input_is_valid,
};

#[path = "path_scan_support.rs"]
mod path_scan_support;

#[path = "../../../src/uri/scheme.rs"]
pub(crate) mod scheme;
