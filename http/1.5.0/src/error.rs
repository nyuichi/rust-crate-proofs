#[cfg(not(http_error_fmt_leaf))]
use std::error;
use std::fmt;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, DeepModel};

#[cfg(test)]
use crate::{header, status};

#[path = "error/core.rs"]
mod core;
pub use self::core::{Error, Result};
use self::core::ErrorKind;

impl fmt::Debug for Error {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tuple = f.debug_tuple("http::Error");
        match self.inner {
            ErrorKind::StatusCode(ref e) => {
                tuple.field(e);
            }
            ErrorKind::Method(ref e) => {
                tuple.field(e);
            }
            ErrorKind::Uri(ref e) => {
                tuple.field(e);
            }
            ErrorKind::UriParts(ref e) => {
                tuple.field(e);
            }
            ErrorKind::HeaderName(ref e) => {
                tuple.field(e);
            }
            ErrorKind::HeaderValue(ref e) => {
                tuple.field(e);
            }
            ErrorKind::MaxSizeReached(ref e) => {
                tuple.field(e);
            }
        }
        tuple.finish()
    }
}

impl fmt::Display for Error {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            ErrorKind::StatusCode(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::Method(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::Uri(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::UriParts(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::HeaderName(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::HeaderValue(ref e) => fmt::Display::fmt(e, f),
            ErrorKind::MaxSizeReached(ref e) => fmt::Display::fmt(e, f),
        }
    }
}

#[cfg(not(http_error_fmt_leaf))]
impl Error {
    /// Return true if the underlying error has the same type as T.
    pub fn is<T: error::Error + 'static>(&self) -> bool {
        self.get_ref().is::<T>()
    }

    /// Return a reference to the lower level, inner error.
    pub fn get_ref(&self) -> &(dyn error::Error + 'static) {
        use self::ErrorKind::*;

        match self.inner {
            StatusCode(ref e) => e,
            Method(ref e) => e,
            Uri(ref e) => e,
            UriParts(ref e) => e,
            HeaderName(ref e) => e,
            HeaderValue(ref e) => e,
            MaxSizeReached(ref e) => e,
        }
    }
}

#[cfg(not(http_error_fmt_leaf))]
impl error::Error for Error {
    // Return any available cause from the inner error. Note the inner error is
    // not itself the cause.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.get_ref().source()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_error_is_invalid_status_code() {
        if let Err(e) = status::StatusCode::from_u16(6666) {
            let err: Error = e.into();
            let ie = err.get_ref();
            assert!(!ie.is::<header::InvalidHeaderValue>());
            assert!(ie.is::<status::InvalidStatusCode>());
            ie.downcast_ref::<status::InvalidStatusCode>().unwrap();

            assert!(!err.is::<header::InvalidHeaderValue>());
            assert!(err.is::<status::InvalidStatusCode>());
        } else {
            panic!("Bad status allowed!");
        }
    }
}
