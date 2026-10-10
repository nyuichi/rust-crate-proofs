// Runtime error types and their exact descriptions.

/// An error in parsing.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(creusot, derive(creusot_std::prelude::DeepModel))]
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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.description_str())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.description_str()
    }
}
