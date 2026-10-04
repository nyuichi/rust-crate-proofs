#![allow(unexpected_cfgs)]

use std::fmt;

#[derive(Debug)]
enum ErrorKind {
    InvalidPort,
}

#[derive(Debug)]
struct InvalidUri;

impl From<ErrorKind> for InvalidUri {
    fn from(_kind: ErrorKind) -> Self {
        InvalidUri
    }
}

// Compile and verify the production Port implementation directly.
#[path = "../../../src/uri/port.rs"]
mod port;

// Compile and verify the exact prefix comparison called by Scheme::parse.
#[path = "../../../src/uri/scheme/fixed.rs"]
mod fixed;

// Keep the shim's error type behavior explicit; this formatter is otherwise
// unused by the included implementation.
impl fmt::Display for InvalidUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid URI")
    }
}
