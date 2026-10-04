//! Runtime error types and their exact descriptions.

use core::fmt;
extern crate creusot_std;
#[allow(unused_imports)]
use creusot_std::prelude::ensures;

/// An error in parsing.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    /// Invalid byte in header name.
    HeaderName,
    /// Invalid byte in header value.
    HeaderValue,
    /// Invalid byte in new line.
    NewLine,
    /// Invalid byte in Response status.
    Status,
    /// Invalid byte where token is required.
    Token,
    /// Parsed more headers than provided buffer can contain.
    TooManyHeaders,
    /// Invalid byte in HTTP version.
    Version,
}

impl Error {
    #[inline]
    #[ensures(match *self {
        Error::HeaderName => result == "invalid header name",
        Error::HeaderValue => result == "invalid header value",
        Error::NewLine => result == "invalid new line",
        Error::Status => result == "invalid response status",
        Error::Token => result == "invalid token",
        Error::TooManyHeaders => result == "too many headers",
        Error::Version => result == "invalid HTTP version",
    })]
    fn description_str(&self) -> &'static str {
        match *self {
            Error::HeaderName => "invalid header name",
            Error::HeaderValue => "invalid header value",
            Error::NewLine => "invalid new line",
            Error::Status => "invalid response status",
            Error::Token => "invalid token",
            Error::TooManyHeaders => "too many headers",
            Error::Version => "invalid HTTP version",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.description_str()
    }
}

/// An error in parsing a chunk size.
// Note: Move this into the error enum once v2.0 is released.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidChunkSize;

impl fmt::Display for InvalidChunkSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid chunk size")
    }
}
