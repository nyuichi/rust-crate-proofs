#![allow(unexpected_cfgs)]

use std::result;

use crate::header;
use crate::header::MaxSizeReached;
use crate::method;
use crate::status;
use crate::uri;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, DeepModel};

/// A generic "error" for HTTP connections.
///
/// This error type is less specific than the error returned from other
/// functions in this crate, but all other errors can be converted to this
/// error. Consumers of this crate can typically consume and work with this
/// form of error for conversions with the `?` operator.
pub struct Error {
    pub(super) inner: ErrorKind,
}

/// A `Result` typedef to use with the `http::Error` type.
pub type Result<T> = result::Result<T, Error>;

pub(super) enum ErrorKind {
    StatusCode(status::InvalidStatusCode),
    Method(method::InvalidMethod),
    Uri(uri::InvalidUri),
    // The full Error formatting and std::error::Error paths consume this
    // payload. Some named Creusot leaf profiles omit those runtime consumers.
    UriParts(#[cfg_attr(creusot, allow(dead_code))] uri::InvalidUriParts),
    HeaderName(header::InvalidHeaderName),
    HeaderValue(header::InvalidHeaderValue),
    MaxSizeReached(MaxSizeReached),
}

/// Proof-level view that preserves the real payload of the wrapped error.
/// The enum is a Creusot-only observer type; production stores `ErrorKind`.
#[cfg(creusot)]
#[doc(hidden)]
pub enum ErrorModel {
    StatusCode(status::InvalidStatusCode),
    Method(method::InvalidMethod),
    Uri(uri::InvalidUri),
    UriParts(#[cfg_attr(creusot, allow(dead_code))] uri::InvalidUriParts),
    HeaderName(header::InvalidHeaderName),
    HeaderValue(header::InvalidHeaderValue),
    MaxSizeReached(MaxSizeReached),
}

/// Borrowed proof-level view used when an `Error` remains inside an enclosing
/// value such as a builder. Each variant borrows the exact stored payload.
#[cfg(creusot)]
#[doc(hidden)]
#[derive(Debug)]
pub enum ErrorModelRef<'a> {
    StatusCode(&'a status::InvalidStatusCode),
    Method(&'a method::InvalidMethod),
    Uri(&'a uri::InvalidUri),
    UriParts(#[cfg_attr(creusot, allow(dead_code))] &'a uri::InvalidUriParts),
    HeaderName(&'a header::InvalidHeaderName),
    HeaderValue(&'a header::InvalidHeaderValue),
    MaxSizeReached(&'a MaxSizeReached),
}

/// Opaque public projection, so contracts expose the variant and payload
/// without making the private `ErrorKind` representation transparent.
#[cfg(creusot)]
#[logic]
#[doc(hidden)]
pub fn error_model(error: Error) -> ErrorModel {
    match error.inner {
        ErrorKind::StatusCode(err) => ErrorModel::StatusCode(err),
        ErrorKind::Method(err) => ErrorModel::Method(err),
        ErrorKind::Uri(err) => ErrorModel::Uri(err),
        ErrorKind::UriParts(err) => ErrorModel::UriParts(err),
        ErrorKind::HeaderName(err) => ErrorModel::HeaderName(err),
        ErrorKind::HeaderValue(err) => ErrorModel::HeaderValue(err),
        ErrorKind::MaxSizeReached(err) => ErrorModel::MaxSizeReached(err),
    }
}

/// Opaque projection preserving both the error variant and its payload while
/// borrowing the original `Error`.
#[cfg(creusot)]
#[logic]
#[doc(hidden)]
pub fn error_model_ref<'a>(error: &'a Error) -> ErrorModelRef<'a> {
    match &error.inner {
        ErrorKind::StatusCode(err) => ErrorModelRef::StatusCode(err),
        ErrorKind::Method(err) => ErrorModelRef::Method(err),
        ErrorKind::Uri(err) => ErrorModelRef::Uri(err),
        ErrorKind::UriParts(err) => ErrorModelRef::UriParts(err),
        ErrorKind::HeaderName(err) => ErrorModelRef::HeaderName(err),
        ErrorKind::HeaderValue(err) => ErrorModelRef::HeaderValue(err),
        ErrorKind::MaxSizeReached(err) => ErrorModelRef::MaxSizeReached(err),
    }
}

/// Whether an error is the empty-URI conversion error consumed by the URI
/// builder's path-and-query setter.
#[cfg(all(
    creusot,
    http_uri_builder_leaf,
    not(http_composition_leaf)
))]
#[logic(open)]
#[doc(hidden)]
pub fn error_is_empty_uri(error: &Error) -> bool {
    match error_model(*error) {
        ErrorModel::Uri(uri_error) => uri_error.deep_model() == 10,
        _ => false,
    }
}

impl Error {
    #[cfg(not(http_composition_leaf))]
    #[cfg_attr(
        all(creusot, not(http_uri_builder_leaf)),
        ensures(match error_model(*self) {
            ErrorModel::Uri(err) => result == (err.deep_model() == 10),
            _ => result == false,
        })
    )]
    #[cfg_attr(
        all(creusot, http_uri_builder_leaf, not(http_composition_leaf)),
        ensures(result == error_is_empty_uri(self))
    )]
    pub(crate) fn is_empty_uri(&self) -> bool {
        match self.inner {
            ErrorKind::Uri(ref err) => err.is_empty(),
            _ => false,
        }
    }
}

impl From<MaxSizeReached> for Error {
    #[ensures(error_model(result) == ErrorModel::MaxSizeReached(err))]
    fn from(err: MaxSizeReached) -> Error {
        Error {
            inner: ErrorKind::MaxSizeReached(err),
        }
    }
}

impl From<status::InvalidStatusCode> for Error {
    #[ensures(error_model(result) == ErrorModel::StatusCode(err))]
    fn from(err: status::InvalidStatusCode) -> Error {
        Error {
            inner: ErrorKind::StatusCode(err),
        }
    }
}

impl From<method::InvalidMethod> for Error {
    #[ensures(error_model(result) == ErrorModel::Method(err))]
    fn from(err: method::InvalidMethod) -> Error {
        Error {
            inner: ErrorKind::Method(err),
        }
    }
}

impl From<uri::InvalidUri> for Error {
    #[ensures(error_model(result) == ErrorModel::Uri(err))]
    fn from(err: uri::InvalidUri) -> Error {
        Error {
            inner: ErrorKind::Uri(err),
        }
    }
}

impl From<uri::InvalidUriParts> for Error {
    #[ensures(error_model(result) == ErrorModel::UriParts(err))]
    fn from(err: uri::InvalidUriParts) -> Error {
        Error {
            inner: ErrorKind::UriParts(err),
        }
    }
}

impl From<header::InvalidHeaderName> for Error {
    #[ensures(error_model(result) == ErrorModel::HeaderName(err))]
    fn from(err: header::InvalidHeaderName) -> Error {
        Error {
            inner: ErrorKind::HeaderName(err),
        }
    }
}

impl From<header::InvalidHeaderValue> for Error {
    #[ensures(error_model(result) == ErrorModel::HeaderValue(err))]
    fn from(err: header::InvalidHeaderValue) -> Error {
        Error {
            inner: ErrorKind::HeaderValue(err),
        }
    }
}

impl From<std::convert::Infallible> for Error {
    // No value of `Infallible` exists, so this conversion has no returning
    // execution. Do not assign a fake ErrorKind tag to an unreachable result.
    #[ensures(false)]
    fn from(err: std::convert::Infallible) -> Error {
        match err {}
    }
}
