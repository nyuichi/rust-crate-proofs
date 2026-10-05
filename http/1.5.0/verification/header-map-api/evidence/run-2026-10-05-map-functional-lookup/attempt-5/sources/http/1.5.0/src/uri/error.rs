use std::error::Error;
use std::fmt;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, DeepModel, Int};

/// An error resulting from a failed attempt to construct a URI.
pub struct InvalidUri(pub(super) ErrorKind);

#[derive(Eq)]
#[cfg_attr(creusot, derive(creusot_std::std::cmp::PartialEq))]
#[cfg_attr(not(creusot), derive(PartialEq))]
pub(super) enum ErrorKind {
    InvalidUriChar,
    InvalidScheme,
    InvalidAuthority,
    InvalidPort,
    InvalidFormat,
    SchemeMissing,
    AuthorityMissing,
    PathAndQueryMissing,
    PathDoesNotStartWithSlash,
    TooLong,
    Empty,
    SchemeTooLong,
}

impl fmt::Debug for ErrorKind {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidUriChar => f.write_str("InvalidUriChar"),
            ErrorKind::InvalidScheme => f.write_str("InvalidScheme"),
            ErrorKind::InvalidAuthority => f.write_str("InvalidAuthority"),
            ErrorKind::InvalidPort => f.write_str("InvalidPort"),
            ErrorKind::InvalidFormat => f.write_str("InvalidFormat"),
            ErrorKind::SchemeMissing => f.write_str("SchemeMissing"),
            ErrorKind::AuthorityMissing => f.write_str("AuthorityMissing"),
            ErrorKind::PathAndQueryMissing => f.write_str("PathAndQueryMissing"),
            ErrorKind::PathDoesNotStartWithSlash => f.write_str("PathDoesNotStartWithSlash"),
            ErrorKind::TooLong => f.write_str("TooLong"),
            ErrorKind::Empty => f.write_str("Empty"),
            ErrorKind::SchemeTooLong => f.write_str("SchemeTooLong"),
        }
    }
}

impl fmt::Debug for InvalidUri {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InvalidUri").field(&self.0).finish()
    }
}

impl DeepModel for ErrorKind {
    type DeepModelTy = Int;

    #[logic(open(super))]
    fn deep_model(self) -> Self::DeepModelTy {
        match self {
            ErrorKind::InvalidUriChar => 0,
            ErrorKind::InvalidScheme => 1,
            ErrorKind::InvalidAuthority => 2,
            ErrorKind::InvalidPort => 3,
            ErrorKind::InvalidFormat => 4,
            ErrorKind::SchemeMissing => 5,
            ErrorKind::AuthorityMissing => 6,
            ErrorKind::PathAndQueryMissing => 7,
            ErrorKind::PathDoesNotStartWithSlash => 8,
            ErrorKind::TooLong => 9,
            ErrorKind::Empty => 10,
            ErrorKind::SchemeTooLong => 11,
        }
    }
}

impl DeepModel for InvalidUri {
    type DeepModelTy = Int;

    #[logic(open(super))]
    fn deep_model(self) -> Self::DeepModelTy {
        self.0.deep_model()
    }
}

impl From<ErrorKind> for InvalidUri {
    #[ensures(result.deep_model() == src.deep_model())]
    fn from(src: ErrorKind) -> InvalidUri {
        InvalidUri(src)
    }
}

impl InvalidUri {
    #[ensures(result == (self.deep_model() == 10))]
    pub(crate) fn is_empty(&self) -> bool {
        matches!(self.0, ErrorKind::Empty)
    }

    fn s(&self) -> &str {
        match self.0 {
            ErrorKind::InvalidUriChar => "invalid uri character",
            ErrorKind::InvalidScheme => "invalid scheme",
            ErrorKind::InvalidAuthority => "invalid authority",
            ErrorKind::InvalidPort => "invalid port",
            ErrorKind::InvalidFormat => "invalid format",
            ErrorKind::SchemeMissing => "scheme missing",
            ErrorKind::AuthorityMissing => "authority missing",
            ErrorKind::PathAndQueryMissing => "path missing",
            ErrorKind::PathDoesNotStartWithSlash => "path does not start with slash",
            ErrorKind::TooLong => "uri too long",
            ErrorKind::Empty => "empty string",
            ErrorKind::SchemeTooLong => "scheme too long",
        }
    }
}

impl fmt::Display for InvalidUri {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.s().fmt(f)
    }
}

impl Error for InvalidUri {}
