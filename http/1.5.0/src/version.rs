//! HTTP version
//!
//! This module contains a definition of the `Version` type. The `Version`
//! type is intended to be accessed through the root of the crate
//! (`http::Version`) rather than this module.
//!
//! The `Version` type contains constants that represent the various versions
//! of the HTTP protocol.
//!
//! # Examples
//!
//! ```
//! use http::Version;
//!
//! let http11 = Version::HTTP_11;
//! let http2 = Version::HTTP_2;
//! assert!(http11 != http2);
//!
//! println!("{:?}", http2);
//! ```

use std::fmt;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, DeepModel, Int, Invariant, View};

/// Represents a version of the HTTP spec.
#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash)]
pub struct Version(Http);

impl View for Version {
    type ViewTy = Int;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! { self.0.deep_model() }
    }
}

impl DeepModel for Version {
    type DeepModelTy = Int;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self@ }
    }
}

impl Invariant for Version {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            self@ == 9 || self@ == 10 || self@ == 11
                || self@ == 20 || self@ == 30
        }
    }
}

impl Version {
    /// `HTTP/0.9`
    pub const HTTP_09: Version = Version(Http::Http09);

    /// `HTTP/1.0`
    pub const HTTP_10: Version = Version(Http::Http10);

    /// `HTTP/1.1`
    pub const HTTP_11: Version = Version(Http::Http11);

    /// `HTTP/2.0`
    pub const HTTP_2: Version = Version(Http::H2);

    /// `HTTP/3.0`
    pub const HTTP_3: Version = Version(Http::H3);
}

#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash)]
enum Http {
    Http09,
    Http10,
    Http11,
    H2,
    H3,
    __NonExhaustive,
}

impl DeepModel for Http {
    type DeepModelTy = Int;

    #[logic(open)]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! {
            match self {
                Http::Http09 => 9,
                Http::Http10 => 10,
                Http::Http11 => 11,
                Http::H2 => 20,
                Http::H3 => 30,
                Http::__NonExhaustive => 255,
            }
        }
    }
}

impl Default for Version {
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == 11 && result.invariant()))]
    fn default() -> Version {
        Version::HTTP_11
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Http::*;

        f.write_str(match self.0 {
            Http09 => "HTTP/0.9",
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
            H2 => "HTTP/2.0",
            H3 => "HTTP/3.0",
            __NonExhaustive => unreachable!(),
        })
    }
}
