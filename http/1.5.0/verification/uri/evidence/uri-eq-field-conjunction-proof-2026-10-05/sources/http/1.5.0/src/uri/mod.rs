//! URI component of request and response lines
//!
//! This module primarily contains the `Uri` type which is a component of all
//! HTTP requests and also reexports this type at the root of the crate. A URI
//! is not always a "full URL" in the sense of something you'd type into a web
//! browser, but HTTP requests may only have paths on servers but may have full
//! schemes and hostnames on clients.
//!
//! # Examples
//!
//! ```
//! use http::Uri;
//!
//! let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
//! assert_eq!(uri.path(), "/foo/bar");
//! assert_eq!(uri.query(), Some("baz"));
//! assert_eq!(uri.host(), None);
//!
//! let uri = "https://www.rust-lang.org/install.html".parse::<Uri>().unwrap();
//! assert_eq!(uri.scheme_str(), Some("https"));
//! assert_eq!(uri.host(), Some("www.rust-lang.org"));
//! assert_eq!(uri.path(), "/install.html");
//! ```

#[cfg(not(http_uri_parts_leaf))]
use crate::byte_str::ByteStr;
use std::convert::TryFrom;

#[cfg(not(http_uri_parts_leaf))]
use bytes::Bytes;

use std::error::Error;
use std::fmt;
#[cfg(not(http_uri_parts_leaf))]
use std::hash::{Hash, Hasher};
#[cfg(not(http_uri_parts_leaf))]
use std::str::{self, FromStr};

use self::scheme::Scheme2;
use self::error::ErrorKind;
use self::limits::MAX_LEN;

#[cfg(creusot)]
use self::path::PathAndQueryModel;
#[cfg(creusot)]
use self::scheme::SchemeModel;

#[allow(unused_imports)]
use creusot_std::prelude::{ensures, logic, pearlite, requires, DeepModel, Int, Seq, View};
#[cfg(creusot)]
use creusot_std::prelude::proof_assert;
#[cfg(creusot)]
use creusot_std::std::partial_eq::PartialEqModel as _;
#[cfg(all(creusot, not(http_uri_parts_leaf)))]
use creusot_std::prelude::inv;

pub use self::authority::Authority;
#[cfg(any(http_uri_builder_leaf, not(any(http_uri_parts_leaf, http_uri_default_leaf))))]
pub use self::builder::Builder;
pub use self::error::InvalidUri;
pub use self::path::PathAndQuery;
pub use self::port::Port;
pub use self::scheme::Scheme;

#[cfg(creusot)]
#[doc(hidden)]
pub use self::path::path_query_boundaries;

/// Proof-visible model of the stored URI fields.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct UriModel {
    pub scheme: SchemeModel,
    pub authority: Seq<u8>,
    pub path_and_query: PathAndQueryModel,
}

/// Canonical model used by `Uri`'s same-type equality. The public `View` stays
/// raw; these fields reflect the equality relation exposed by the runtime.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct UriCompareModel {
    pub scheme: UriSchemeCompareModel,
    pub authority: Seq<Int>,
    pub path: Seq<u8>,
    pub query: Option<Seq<u8>>,
}

#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum UriSchemeCompareModel {
    None,
    Http,
    Https,
    Other(Seq<Int>),
}

/// Proof-visible optional field model of `Parts`.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct PartsModel {
    pub scheme: Option<SchemeModel>,
    pub authority: Option<Seq<u8>>,
    pub path_and_query: Option<PathAndQueryModel>,
}

#[cfg(creusot)]
impl View for Uri {
    type ViewTy = UriModel;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            UriModel {
                scheme: self.scheme@,
                authority: self.authority@,
                path_and_query: self.path_and_query@,
            }
        }
    }
}

#[cfg(creusot)]
impl DeepModel for Uri {
    type DeepModelTy = UriCompareModel;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { uri_comparison_model(self@) }
    }
}

impl Clone for Uri {
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn clone(&self) -> Self {
        Uri {
            scheme: self.scheme.clone(),
            authority: self.authority.clone(),
            path_and_query: self.path_and_query.clone(),
        }
    }
}

#[cfg(creusot)]
impl View for Parts {
    type ViewTy = PartsModel;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            PartsModel {
                scheme: match self.scheme {
                    Some(value) => Some(value@),
                    None => None,
                },
                authority: match self.authority {
                    Some(value) => Some(value@),
                    None => None,
                },
                path_and_query: match self.path_and_query {
                    Some(value) => Some(value@),
                    None => None,
                },
            }
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_is_none(scheme: SchemeModel) -> bool {
    pearlite! { match scheme {
        SchemeModel::None => true,
        _ => false,
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_model_has_path(model: UriModel) -> bool {
    pearlite! {
        model.path_and_query.bytes.len() > 0 || !uri_scheme_is_none(model.scheme)
    }
}

/// Whether a stored URI has the exact component model produced by `Default`.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_is_default(model: UriModel) -> bool {
    pearlite! {
        model.scheme == SchemeModel::None
            && model.authority.len() == 0
            && model.path_and_query.bytes == Seq::singleton(47u8)
            && model.path_and_query.query == None
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_ascii_fold_bytes(bytes: Seq<u8>) -> Seq<Int> {
    pearlite! { authority::authority_folded_bytes(bytes) }
}

/// Model `Scheme::eq`: standard variants stay distinct from custom spellings,
/// while two custom spellings compare with ASCII case folding.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_models_equal(left: SchemeModel, right: SchemeModel) -> bool {
    pearlite! { match (left, right) {
        (SchemeModel::None, SchemeModel::None) => true,
        (SchemeModel::Http, SchemeModel::Http) => true,
        (SchemeModel::Https, SchemeModel::Https) => true,
        (SchemeModel::Other(left), SchemeModel::Other(right)) =>
            uri_ascii_fold_bytes(left) == uri_ascii_fold_bytes(right),
        _ => false,
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_comparison_model(scheme: SchemeModel) -> UriSchemeCompareModel {
    pearlite! { match scheme {
        SchemeModel::None => UriSchemeCompareModel::None,
        SchemeModel::Http => UriSchemeCompareModel::Http,
        SchemeModel::Https => UriSchemeCompareModel::Https,
        SchemeModel::Other(bytes) =>
            UriSchemeCompareModel::Other(uri_ascii_fold_bytes(bytes)),
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_comparison_model(model: UriModel) -> UriCompareModel {
    pearlite! {
        UriCompareModel {
            scheme: uri_scheme_comparison_model(model.scheme),
            authority: uri_ascii_fold_bytes(model.authority),
            path: if uri_model_has_path(model) {
                path::path_component_bytes(model.path_and_query)
            } else {
                Seq::<u8>::empty()
            },
            query: match model.path_and_query.query {
                None => None,
                Some(offset) => Some(model.path_and_query.bytes.subsequence(
                    offset + 1, model.path_and_query.bytes.len(),
                )),
            },
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_comparison_text(scheme: UriSchemeCompareModel) -> Option<Seq<Int>> {
    pearlite! { match scheme {
        UriSchemeCompareModel::None => None,
        UriSchemeCompareModel::Http => Some(
            seq![104u8, 116u8, 116u8, 112u8].map(|byte: u8| byte@)
        ),
        UriSchemeCompareModel::Https => Some(
            seq![104u8, 116u8, 116u8, 112u8, 115u8].map(|byte: u8| byte@)
        ),
        UriSchemeCompareModel::Other(bytes) => Some(bytes),
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_comparison_matches_text(
    scheme: UriSchemeCompareModel,
    text: Seq<char>,
) -> bool {
    pearlite! { match uri_scheme_comparison_text(scheme) {
        None => false,
        Some(expected) =>
            uri_ascii_fold_bytes(text.to_bytes()) == expected,
    } }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<Seq<char>>
    for UriSchemeCompareModel
{
    #[logic(open)]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { uri_scheme_comparison_matches_text(self, rhs) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<UriSchemeCompareModel>
    for Seq<char>
{
    #[logic(open)]
    fn eq_model(self, rhs: UriSchemeCompareModel) -> bool {
        pearlite! { uri_scheme_comparison_matches_text(rhs, self) }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_folded_bytes_match_at(
    bytes: Seq<u8>,
    start: Int,
    expected: Seq<Int>,
) -> bool {
    pearlite! {
        0 <= start && start + expected.len() <= bytes.len()
            && uri_ascii_fold_bytes(
                bytes.subsequence(start, start + expected.len())
            ) == expected
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_exact_bytes_match_at(
    bytes: Seq<u8>,
    start: Int,
    expected: Seq<u8>,
) -> bool {
    pearlite! {
        0 <= start && start + expected.len() <= bytes.len()
            && bytes.subsequence(start, start + expected.len()) == expected
    }
}

/// Compare a canonical URI model with UTF-8 text using the byte-prefix rules
/// in `Uri::eq(str)`, including an omitted absolute `/` and ignored fragments.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_compare_model_matches_text(
    model: UriCompareModel,
    text: Seq<char>,
) -> bool {
    pearlite! {
        let bytes = text.to_bytes();
        let scheme_text = uri_scheme_comparison_text(model.scheme);
        let scheme_present = match scheme_text {
            None => false,
            Some(_) => true,
        };
        let scheme_ok = match scheme_text {
            None => true,
            Some(scheme_bytes) =>
                0 <= scheme_bytes.len() + 3
                    && scheme_bytes.len() + 3 <= bytes.len()
                    && uri_folded_bytes_match_at(bytes, 0, scheme_bytes)
                    && bytes.subsequence(
                        scheme_bytes.len(), scheme_bytes.len() + 3,
                    ) == seq![58u8, 47u8, 47u8],
        };
        let after_scheme = match scheme_text {
            None => 0,
            Some(scheme_bytes) => scheme_bytes.len() + 3,
        };
        let authority_present = model.authority.len() > 0;
        let authority_ok = !authority_present
            || uri_folded_bytes_match_at(bytes, after_scheme, model.authority);
        let after_authority = after_scheme + model.authority.len();
        let absolute = scheme_present || authority_present;
        let path_start = if scheme_ok && authority_ok { after_authority } else { 0 };
        let path_matches = uri_exact_bytes_match_at(bytes, path_start, model.path);
        let omitted_absolute_slash = absolute && model.path == seq![47u8];
        let path_ok = path_matches || omitted_absolute_slash;
        let after_path = if path_matches {
            path_start + model.path.len()
        } else {
            path_start
        };
        let query_ok = match model.query {
            None => true,
            Some(query_bytes) => {
                if after_path == bytes.len() {
                    query_bytes.len() == 0
                } else {
                    after_path < bytes.len()
                        && bytes[after_path]@ == 63
                        && uri_exact_bytes_match_at(
                            bytes, after_path + 1, query_bytes,
                        )
                }
            }
        };
        let after_query = match model.query {
            None => after_path,
            Some(query_bytes) => {
                if after_path == bytes.len() {
                    after_path
                } else {
                    after_path + 1 + query_bytes.len()
                }
            }
        };
        scheme_ok && authority_ok && path_ok && query_ok
            && (after_query == bytes.len()
                || (0 <= after_query && after_query < bytes.len()
                    && bytes[after_query]@ == 35))
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<Seq<char>> for UriCompareModel {
    #[logic(open)]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { uri_compare_model_matches_text(self, rhs) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<UriCompareModel> for Seq<char> {
    #[logic(open)]
    fn eq_model(self, rhs: UriCompareModel) -> bool {
        pearlite! { uri_compare_model_matches_text(rhs, self) }
    }
}

/// Exact component equality used by `Uri::eq`. This follows the public
/// accessors: an empty relative path stays empty, an empty absolute path is
/// exposed as `/`, and a present empty query differs from no query.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_models_equal(left: UriModel, right: UriModel) -> bool {
    pearlite! {
        uri_comparison_model(left) == uri_comparison_model(right)
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_scheme_display_bytes(scheme: SchemeModel) -> Seq<u8> {
    pearlite! { match scheme {
        SchemeModel::None => Seq::<u8>::empty(),
        SchemeModel::Http => seq![104u8, 116u8, 116u8, 112u8],
        SchemeModel::Https => seq![104u8, 116u8, 116u8, 112u8, 115u8],
        SchemeModel::Other(bytes) => bytes,
    } }
}

/// The exact byte text produced by `Uri`'s Display implementation, converted
/// to the formatter's integer-byte model.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
#[requires(path_query_boundaries(model.path_and_query))]
pub fn uri_display_text(model: UriModel) -> Seq<Int> {
    pearlite! {
        let has_path = uri_model_has_path(model);
        let scheme_text = if uri_scheme_is_none(model.scheme) {
            Seq::<u8>::empty()
        } else {
            uri_scheme_display_bytes(model.scheme).concat(seq![58u8, 47u8, 47u8])
        };
        let authority_text = model.authority;
        let path_text = if has_path {
            path::path_component_bytes(model.path_and_query)
        } else {
            Seq::<u8>::empty()
        };
        let query_text = match model.path_and_query.query {
            None => Seq::<u8>::empty(),
            Some(offset) => seq![63u8].concat(
                model.path_and_query.bytes.subsequence(
                    offset + 1, model.path_and_query.bytes.len(),
                )
            ),
        };
        scheme_text.concat(authority_text).concat(path_text)
            .concat(query_text).map(|byte: u8| byte@)
    }
}

/// Whether the formatter has written a prefix of the complete URI display
/// text. `Formatter::write_str` may fail after writing a prefix of its input.
#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_formatter_output_is_prefix(
    before: Seq<Int>,
    after: Seq<Int>,
    text: Seq<Int>,
) -> bool {
    pearlite! {
        exists<end: Int> 0 <= end && end <= text.len()
            && after == before.concat(text.subsequence(0, end))
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_parts_model(model: UriModel) -> PartsModel {
    pearlite! {
        PartsModel {
            scheme: if uri_scheme_is_none(model.scheme) {
                None
            } else {
                Some(model.scheme)
            },
            authority: if model.authority.len() == 0 {
                None
            } else {
                Some(model.authority)
            },
            path_and_query: if uri_model_has_path(model) {
                Some(model.path_and_query)
            } else {
                None
            },
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_parts_model_is_valid(model: PartsModel) -> bool {
    pearlite! {
        match model.scheme {
            Some(_) => uri_option_is_some(model.authority)
                && uri_option_is_some(model.path_and_query),
            None => !(uri_option_is_some(model.authority)
                && uri_option_is_some(model.path_and_query)),
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_option_is_some<T>(value: Option<T>) -> bool {
    pearlite! { match value {
        Some(_) => true,
        None => false,
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_option_is_none<T>(value: Option<T>) -> bool {
    pearlite! { match value {
        Some(_) => false,
        None => true,
    } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_model_from_parts(model: PartsModel) -> UriModel {
    pearlite! {
        UriModel {
            scheme: match model.scheme {
                Some(scheme) => scheme,
                None => SchemeModel::None,
            },
            authority: match model.authority {
                Some(authority) => authority,
                None => Seq::<u8>::empty(),
            },
            path_and_query: match model.path_and_query {
                Some(path_and_query) => path_and_query,
                None => PathAndQueryModel {
                    bytes: Seq::<u8>::empty(),
                    query: None,
                },
            },
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn uri_parts_error_matches(model: PartsModel, error: Int) -> bool {
    pearlite! {
        (uri_option_is_some(model.scheme)
            && uri_option_is_none(model.authority) && error == 6)
            || (uri_option_is_some(model.scheme)
                && uri_option_is_some(model.authority)
                && uri_option_is_none(model.path_and_query) && error == 7)
            || (uri_option_is_none(model.scheme)
                && uri_option_is_some(model.authority)
                && uri_option_is_some(model.path_and_query) && error == 5)
    }
}

#[cfg(creusot)]
#[doc(hidden)]
pub use self::authority::{
    authority_error_matches_rejection, authority_first_byte_from,
    authority_host_end, authority_host_input_is_safe, authority_host_start,
    authority_input_is_fully_valid, authority_port_number, authority_port_start,
    authority_static_input_is_valid,
};

pub(crate) mod authority;
pub(crate) mod authority_chars;
#[cfg(any(http_uri_builder_leaf, not(any(http_uri_parts_leaf, http_uri_default_leaf))))]
mod builder;
mod error;
mod limits;
mod path;
#[cfg(creusot)]
#[path = "path_static_domain.rs"]
mod path_static_domain;
pub(crate) mod port;
mod scheme;
#[cfg(test)]
mod tests;

#[cfg(creusot)]
#[doc(hidden)]
pub use self::path_static_domain::{
    path_static_input_is_valid, path_static_path_byte_valid, path_static_query_byte_valid,
};

#[cfg(all(creusot, http_uri_builder_leaf))]
#[doc(hidden)]
pub use self::builder::{
    uri_builder_authority, uri_builder_error, uri_builder_error_is_from,
    uri_builder_build_post, uri_builder_has_default_parts,
    uri_builder_has_empty_path_and_query, uri_builder_is_valid, uri_builder_parts,
    uri_builder_path_and_query, uri_builder_scheme,
};

/// The URI component of a request.
///
/// For HTTP 1, this is included as part of the request line. From Section 5.3,
/// Request Target:
///
/// > Once an inbound connection is obtained, the client sends an HTTP
/// > request message (Section 3) with a request-target derived from the
/// > target URI.  There are four distinct formats for the request-target,
/// > depending on both the method being requested and whether the request
/// > is to a proxy.
/// >
/// > ```notrust
/// > request-target = origin-form
/// >                / absolute-form
/// >                / authority-form
/// >                / asterisk-form
/// > ```
///
/// The URI is structured as follows:
///
/// ```notrust
/// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
/// |-|   |-------------------------------||--------| |-------------------| |-----|
///  |                  |                       |               |              |
/// scheme          authority                 path            query         fragment
/// ```
///
/// For HTTP 2.0, the URI is encoded using pseudoheaders.
///
/// # Examples
///
/// ```
/// use http::Uri;
///
/// let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
/// assert_eq!(uri.path(), "/foo/bar");
/// assert_eq!(uri.query(), Some("baz"));
/// assert_eq!(uri.host(), None);
///
/// let uri = "https://www.rust-lang.org/install.html".parse::<Uri>().unwrap();
/// assert_eq!(uri.scheme_str(), Some("https"));
/// assert_eq!(uri.host(), Some("www.rust-lang.org"));
/// assert_eq!(uri.path(), "/install.html");
/// ```
pub struct Uri {
    scheme: Scheme,
    authority: Authority,
    path_and_query: PathAndQuery,
}

/// The various parts of a URI.
///
/// This struct is used to provide to and retrieve from a URI.
#[derive(Debug)]
pub struct Parts {
    /// The scheme component of a URI
    pub scheme: Option<Scheme>,

    /// The authority component of a URI
    pub authority: Option<Authority>,

    /// The origin-form component of a URI
    pub path_and_query: Option<PathAndQuery>,

    /// Allow extending in the future
    _priv: (),
}

impl Default for Parts {
    #[cfg_attr(creusot, ensures(
        result@.scheme == None
            && result@.authority == None
            && result@.path_and_query == None
    ))]
    fn default() -> Self {
        Parts {
            scheme: None,
            authority: None,
            path_and_query: None,
            _priv: (),
        }
    }
}

/// An error resulting from a failed attempt to construct a URI.
#[derive(Debug)]
pub struct InvalidUriParts(pub(super) InvalidUri);

#[cfg(creusot)]
impl DeepModel for InvalidUriParts {
    type DeepModelTy = Int;

    #[logic(open(super))]
    fn deep_model(self) -> Self::DeepModelTy {
        self.0.deep_model()
    }
}

// URI_CHARS is a table of valid characters in a URI. An entry in the table is
// 0 for invalid characters. For valid characters the entry is itself (i.e.
// the entry for 33 is b'!' because b'!' == 33u8). An important characteristic
// of this table is that all entries above 127 are invalid. This makes all of the
// valid entries a valid single-byte UTF-8 code point. This means that a slice
// of such valid entries is valid UTF-8.
#[rustfmt::skip]
#[cfg_attr(not(test), allow(dead_code))]
const URI_CHARS: [u8; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //   x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  1x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  2x
        0,     0,     0,  b'!',     0,  b'#',  b'$',     0,  b'&', b'\'', //  3x
     b'(',  b')',  b'*',  b'+',  b',',  b'-',  b'.',  b'/',  b'0',  b'1', //  4x
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',  b':',  b';', //  5x
        0,  b'=',     0,  b'?',  b'@',  b'A',  b'B',  b'C',  b'D',  b'E', //  6x
     b'F',  b'G',  b'H',  b'I',  b'J',  b'K',  b'L',  b'M',  b'N',  b'O', //  7x
     b'P',  b'Q',  b'R',  b'S',  b'T',  b'U',  b'V',  b'W',  b'X',  b'Y', //  8x
     b'Z',  b'[',     0,  b']',     0,  b'_',     0,  b'a',  b'b',  b'c', //  9x
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm', // 10x
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w', // 11x
     b'x',  b'y',  b'z',     0,     0,     0,  b'~',     0,     0,     0, // 12x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 13x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 14x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 15x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 16x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 17x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 18x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 19x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 20x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 21x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 22x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 23x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, // 24x
        0,     0,     0,     0,     0,     0                              // 25x
];

impl Uri {
    /// Creates a new builder-style object to manufacture a `Uri`.
    ///
    /// This method returns an instance of `Builder` which can be usd to
    /// create a `Uri`.
    ///
    /// # Examples
    ///
    /// ```
    /// use http::Uri;
    ///
    /// let uri = Uri::builder()
    ///     .scheme("https")
    ///     .authority("hyper.rs")
    ///     .path_and_query("/")
    ///     .build()
    ///     .unwrap();
    /// ```
    #[cfg(any(http_uri_builder_leaf, not(any(http_uri_parts_leaf, http_uri_default_leaf))))]
    #[cfg_attr(creusot, ensures(builder::uri_builder_is_valid(&result)))]
    #[cfg_attr(creusot, ensures(builder::uri_builder_has_default_parts(&result)))]
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Attempt to convert a `Parts` into a `Uri`.
    ///
    /// # Examples
    ///
    /// Relative URI
    ///
    /// ```
    /// # use http::uri::*;
    /// let mut parts = Parts::default();
    /// parts.path_and_query = Some("/foo".parse().unwrap());
    ///
    /// let uri = Uri::from_parts(parts).unwrap();
    ///
    /// assert_eq!(uri.path(), "/foo");
    ///
    /// assert!(uri.scheme().is_none());
    /// assert!(uri.authority().is_none());
    /// ```
    ///
    /// Absolute URI
    ///
    /// ```
    /// # use http::uri::*;
    /// let mut parts = Parts::default();
    /// parts.scheme = Some("http".parse().unwrap());
    /// parts.authority = Some("foo.com".parse().unwrap());
    /// parts.path_and_query = Some("/foo".parse().unwrap());
    ///
    /// let uri = Uri::from_parts(parts).unwrap();
    ///
    /// assert_eq!(uri.scheme().unwrap().as_str(), "http");
    /// assert_eq!(uri.authority().unwrap(), "foo.com");
    /// assert_eq!(uri.path(), "/foo");
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Ok(uri) => uri_parts_model_is_valid(src@)
            && uri@.scheme == uri_model_from_parts(src@).scheme
            && uri@.authority == uri_model_from_parts(src@).authority
            && uri@.path_and_query == uri_model_from_parts(src@).path_and_query,
        Err(error) => uri_parts_error_matches(src@, error.deep_model()),
    }))]
    pub fn from_parts(src: Parts) -> Result<Uri, InvalidUriParts> {
        if src.scheme.is_some() {
            if src.authority.is_none() {
                return Err(ErrorKind::AuthorityMissing.into());
            }

            if src.path_and_query.is_none() {
                return Err(ErrorKind::PathAndQueryMissing.into());
            }
        } else if src.authority.is_some() && src.path_and_query.is_some() {
            return Err(ErrorKind::SchemeMissing.into());
        }

        let scheme = match src.scheme {
            Some(scheme) => scheme,
            None => Scheme::empty(),
        };

        let authority = match src.authority {
            Some(authority) => authority,
            None => Authority::empty(),
        };

        let path_and_query = match src.path_and_query {
            Some(path_and_query) => path_and_query,
            None => PathAndQuery::empty(),
        };

        Ok(Uri {
            scheme,
            authority,
            path_and_query,
        })
    }

    /// Attempt to convert a `Bytes` buffer to a `Uri`.
    ///
    /// This will try to prevent a copy if the type passed is the type used
    /// internally, and will copy the data if it is not.
    #[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf, http_uri_leaf)))]
    pub fn from_maybe_shared<T>(src: T) -> Result<Self, InvalidUri>
    where
        T: AsRef<[u8]> + 'static,
    {
        if_downcast_into!(T, Bytes, src, {
            return Uri::from_shared(src);
        });

        Uri::try_from(src.as_ref())
    }

    // Not public while `bytes` is unstable.
    #[cfg(not(http_uri_parts_leaf))]
    fn from_shared(s: Bytes) -> Result<Uri, InvalidUri> {
        use self::ErrorKind::*;

        if s.len() > MAX_LEN {
            return Err(TooLong.into());
        }

        match s.len() {
            0 => {
                return Err(Empty.into());
            }
            1 => match s[0] {
                b'/' => {
                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority: Authority::empty(),
                        path_and_query: PathAndQuery::slash(),
                    });
                }
                b'*' => {
                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority: Authority::empty(),
                        path_and_query: PathAndQuery::star(),
                    });
                }
                _ => {
                    let authority = Authority::from_shared(s)?;

                    return Ok(Uri {
                        scheme: Scheme::empty(),
                        authority,
                        path_and_query: PathAndQuery::empty(),
                    });
                }
            },
            _ => {}
        }

        if s[0] == b'/' {
            return Ok(Uri {
                scheme: Scheme::empty(),
                authority: Authority::empty(),
                path_and_query: PathAndQuery::from_shared(s)?,
            });
        }

        parse_full(s)
    }

    /// Convert a `Uri` from a static string.
    ///
    /// This function will not perform any copying, however the string is
    /// checked to ensure that it is valid.
    ///
    /// # Panics
    ///
    /// This function panics if the argument is an invalid URI.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::Uri;
    /// let uri = Uri::from_static("http://example.com/foo");
    ///
    /// assert_eq!(uri.host().unwrap(), "example.com");
    /// assert_eq!(uri.path(), "/foo");
    /// ```
    #[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf, http_uri_compare_leaf)))]
    pub fn from_static(src: &'static str) -> Self {
        let s = Bytes::from_static(src.as_bytes());
        match Uri::from_shared(s) {
            Ok(uri) => uri,
            Err(e) => panic!("static str is not valid URI: {}", e),
        }
    }

    /// Convert a `Uri` into `Parts`.
    ///
    /// # Note
    ///
    /// This is just an inherent method providing the same functionality as
    /// `let parts: Parts = uri.into()`
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::*;
    /// let uri: Uri = "/foo".parse().unwrap();
    ///
    /// let parts = uri.into_parts();
    ///
    /// assert_eq!(parts.path_and_query.unwrap(), "/foo");
    ///
    /// assert!(parts.scheme.is_none());
    /// assert!(parts.authority.is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result@.scheme == uri_parts_model(self@).scheme
        && result@.authority == uri_parts_model(self@).authority
        && result@.path_and_query == uri_parts_model(self@).path_and_query))]
    pub fn into_parts(self) -> Parts {
        self.into()
    }

    /// Returns the path & query components of the Uri
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Some(path) => path@ == self@.path_and_query,
        None => uri_scheme_is_none(self@.scheme) && self@.authority.len() > 0,
    }))]
    pub fn path_and_query(&self) -> Option<&PathAndQuery> {
        if !self.scheme.inner.is_none() || self.authority.data.len() == 0 {
            Some(&self.path_and_query)
        } else {
            None
        }
    }

    /// Get the path of this `Uri`.
    ///
    /// Both relative and absolute URIs contain a path component, though it
    /// might be the empty string. The path component is **case sensitive**.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                                        |--------|
    ///                                             |
    ///                                           path
    /// ```
    ///
    /// If the URI is `*` then the path component is equal to `*`.
    ///
    /// # Examples
    ///
    /// A relative URI
    ///
    /// ```
    /// # use http::Uri;
    ///
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.path(), "/hello/world");
    /// ```
    ///
    /// An absolute URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.path(), "/hello/world");
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(path_query_boundaries(self@.path_and_query)))]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == if uri_model_has_path(self@) {
        crate::uri::path::path_component_bytes(self@.path_and_query)
    } else {
        Seq::<u8>::empty()
    }))]
    pub fn path(&self) -> &str {
        if self.has_path() {
            self.path_and_query.path()
        } else {
            ""
        }
    }

    /// Get the scheme of this `Uri`.
    ///
    /// The URI scheme refers to a specification for assigning identifiers
    /// within that scheme. Only absolute URIs contain a scheme component, but
    /// not all absolute URIs will contain a scheme component.  Although scheme
    /// names are case-insensitive, the canonical form is lowercase.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    /// |-|
    ///  |
    /// scheme
    /// ```
    ///
    /// # Examples
    ///
    /// Absolute URI
    ///
    /// ```
    /// use http::uri::{Scheme, Uri};
    ///
    /// let uri: Uri = "http://example.org/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.scheme(), Some(&Scheme::HTTP));
    /// ```
    ///
    ///
    /// Relative URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert!(uri.scheme().is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Some(scheme) => scheme@ == self@.scheme
            && !uri_scheme_is_none(self@.scheme),
        None => uri_scheme_is_none(self@.scheme),
    }))]
    pub fn scheme(&self) -> Option<&Scheme> {
        if self.scheme.inner.is_none() {
            None
        } else {
            Some(&self.scheme)
        }
    }

    /// Get the scheme of this `Uri` as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.scheme_str(), Some("http"));
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Some(text) => !uri_scheme_is_none(self@.scheme)
            && scheme::scheme_text_matches(self@.scheme, text@),
        None => uri_scheme_is_none(self@.scheme),
    }))]
    pub fn scheme_str(&self) -> Option<&str> {
        if self.scheme.inner.is_none() {
            None
        } else {
            Some(self.scheme.as_str())
        }
    }

    /// Get the authority of this `Uri`.
    ///
    /// The authority is a hierarchical element for naming authority such that
    /// the remainder of the URI is delegated to that authority. For HTTP, the
    /// authority consists of the host and port. The host portion of the
    /// authority is **case-insensitive**.
    ///
    /// The authority also includes a `username:password` component, however
    /// the use of this is deprecated and should be avoided.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///       |-------------------------------|
    ///                     |
    ///                 authority
    /// ```
    ///
    /// # Examples
    ///
    /// Absolute URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org:80/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.authority().map(|a| a.as_str()), Some("example.org:80"));
    /// ```
    ///
    ///
    /// Relative URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert!(uri.authority().is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Some(authority) => authority@ == self@.authority
            && self@.authority.len() > 0,
        None => self@.authority.len() == 0,
    }))]
    pub fn authority(&self) -> Option<&Authority> {
        if self.authority.data.len() == 0 {
            None
        } else {
            Some(&self.authority)
        }
    }

    /// Get the host of this `Uri`.
    ///
    /// The host subcomponent of authority is identified by an IP literal
    /// encapsulated within square brackets, an IPv4 address in dotted- decimal
    /// form, or a registered name.  The host subcomponent is **case-insensitive**.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                         |---------|
    ///                              |
    ///                             host
    /// ```
    ///
    /// # Examples
    ///
    /// Absolute URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org:80/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.host(), Some("example.org"));
    /// ```
    ///
    ///
    /// Relative URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert!(uri.host().is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(self@.authority.len() == 0
        || authority_host_input_is_safe(self@.authority)))]
    #[cfg_attr(creusot, ensures(match result {
        Some(host) => authority_host_input_is_safe(self@.authority)
            && host@.to_bytes() == self@.authority.subsequence(
                authority_host_start(self@.authority),
                authority_host_end(self@.authority),
            ),
        None => self@.authority.len() == 0,
    }))]
    pub fn host(&self) -> Option<&str> {
        self.authority().map(|a| a.host())
    }

    /// Get the port part of this `Uri`.
    ///
    /// The port subcomponent of authority is designated by an optional port
    /// number following the host and delimited from it by a single colon (":")
    /// character. It can be turned into a decimal port number with the `as_u16`
    /// method or as a `str` with the `as_str` method.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                                     |-|
    ///                                      |
    ///                                     port
    /// ```
    ///
    /// # Examples
    ///
    /// Absolute URI with port
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org:80/hello/world".parse().unwrap();
    ///
    /// let port = uri.port().unwrap();
    /// assert_eq!(port.as_u16(), 80);
    /// ```
    ///
    /// Absolute URI without port
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org/hello/world".parse().unwrap();
    ///
    /// assert!(uri.port().is_none());
    /// ```
    ///
    /// Relative URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert!(uri.port().is_none());
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(port) => authority_port_number(self@.authority) == Some(port@@),
        None => authority_port_number(self@.authority) == None,
    }))]
    pub fn port(&self) -> Option<Port<&str>> {
        self.authority().and_then(|a| a.port())
    }

    /// Get the port of this `Uri` as a `u16`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use http::{Uri, uri::Port};
    /// let uri: Uri = "http://example.org:80/hello/world".parse().unwrap();
    ///
    /// assert_eq!(uri.port_u16(), Some(80));
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Some(port) => authority_port_number(self@.authority) == Some(port@),
        None => authority_port_number(self@.authority) == None,
    }))]
    pub fn port_u16(&self) -> Option<u16> {
        self.port().map(|p| p.as_u16())
    }

    /// Get the query string of this `Uri`, starting after the `?`.
    ///
    /// The query component contains non-hierarchical data that, along with data
    /// in the path component, serves to identify a resource within the scope of
    /// the URI's scheme and naming authority (if any). The query component is
    /// indicated by the first question mark ("?") character and terminated by a
    /// number sign ("#") character or by the end of the URI.
    ///
    /// ```notrust
    /// abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
    ///                                                   |-------------------|
    ///                                                             |
    ///                                                           query
    /// ```
    ///
    /// # Examples
    ///
    /// Absolute URI
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "http://example.org/hello/world?key=value".parse().unwrap();
    ///
    /// assert_eq!(uri.query(), Some("key=value"));
    /// ```
    ///
    /// Relative URI with a query string component
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world?key=value&foo=bar".parse().unwrap();
    ///
    /// assert_eq!(uri.query(), Some("key=value&foo=bar"));
    /// ```
    ///
    /// Relative URI without a query string component
    ///
    /// ```
    /// # use http::Uri;
    /// let uri: Uri = "/hello/world".parse().unwrap();
    ///
    /// assert!(uri.query().is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(path_query_boundaries(self@.path_and_query)))]
    #[cfg_attr(creusot, ensures(match (self@.path_and_query.query, result) {
        (None, None) => true,
        (Some(offset), Some(value)) => value@.to_bytes()
            == self@.path_and_query.bytes.subsequence(
                offset + 1, self@.path_and_query.bytes.len(),
            ),
        _ => false,
    }))]
    pub fn query(&self) -> Option<&str> {
        self.path_and_query.query()
    }

    #[cfg_attr(creusot, ensures(result == uri_model_has_path(self@)))]
    fn has_path(&self) -> bool {
        self.path_and_query.data.len() > 0 || !self.scheme.inner.is_none()
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl TryFrom<&[u8]> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &[u8]) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::copy_from_slice(t))
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl TryFrom<&str> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &str) -> Result<Self, Self::Error> {
        t.parse()
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl TryFrom<&String> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: &String) -> Result<Self, Self::Error> {
        t.parse()
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl TryFrom<String> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(t: String) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::from(t))
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl TryFrom<Vec<u8>> for Uri {
    type Error = InvalidUri;

    #[inline]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Uri::from_shared(Bytes::from(vec))
    }
}

impl TryFrom<Parts> for Uri {
    type Error = InvalidUriParts;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(uri) => uri_parts_model_is_valid(src@)
            && uri@.scheme == uri_model_from_parts(src@).scheme
            && uri@.authority == uri_model_from_parts(src@).authority
            && uri@.path_and_query == uri_model_from_parts(src@).path_and_query,
        Err(error) => uri_parts_error_matches(src@, error.deep_model()),
    }))]
    fn try_from(src: Parts) -> Result<Self, Self::Error> {
        Uri::from_parts(src)
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl TryFrom<&Uri> for Uri {
    type Error = crate::Error;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(uri) => uri@ == src@,
        Err(_) => false,
    }))]
    fn try_from(src: &Uri) -> Result<Self, Self::Error> {
        Ok(src.clone())
    }
}

/// Convert an `Authority` into a `Uri`.
impl From<Authority> for Uri {
    #[cfg_attr(creusot, ensures(result@.scheme == SchemeModel::None
        && result@.authority == authority@
        && result@.path_and_query.bytes == Seq::<u8>::empty()
        && result@.path_and_query.query == None))]
    fn from(authority: Authority) -> Self {
        Self {
            scheme: Scheme::empty(),
            authority,
            path_and_query: PathAndQuery::empty(),
        }
    }
}

/// Convert a `PathAndQuery` into a `Uri`.
impl From<PathAndQuery> for Uri {
    #[cfg_attr(creusot, ensures(result@.scheme == SchemeModel::None
        && result@.authority == Seq::<u8>::empty()
        && result@.path_and_query == path_and_query@))]
    fn from(path_and_query: PathAndQuery) -> Self {
        Self {
            scheme: Scheme::empty(),
            authority: Authority::empty(),
            path_and_query,
        }
    }
}

/// Convert a `Uri` into `Parts`
impl From<Uri> for Parts {
    #[cfg_attr(creusot, ensures(result@.scheme == uri_parts_model(src@).scheme
        && result@.authority == uri_parts_model(src@).authority
        && result@.path_and_query == uri_parts_model(src@).path_and_query))]
    fn from(src: Uri) -> Self {
        let path_and_query = if src.has_path() {
            Some(src.path_and_query)
        } else {
            None
        };

        let scheme = match src.scheme.inner {
            Scheme2::None => None,
            _ => Some(src.scheme),
        };

        let authority = if src.authority.data.len() == 0 {
            None
        } else {
            Some(src.authority)
        };

        Parts {
            scheme,
            authority,
            path_and_query,
            _priv: (),
        }
    }
}

#[cfg(not(http_uri_parts_leaf))]
fn parse_full(mut s: Bytes) -> Result<Uri, InvalidUri> {
    // Parse the scheme
    let scheme = match Scheme2::parse(&s[..])? {
        Scheme2::None => Scheme2::None,
        Scheme2::Standard(p) => {
            // TODO: use truncate
            let _ = s.split_to(p.len() + 3);
            Scheme2::Standard(p)
        }
        Scheme2::Other(n) => {
            // Grab the protocol
            let mut scheme = s.split_to(n + 3);

            // Strip ://, TODO: truncate
            let _ = scheme.split_off(n);

            // Allocate the ByteStr
            let val = unsafe { ByteStr::from_utf8_unchecked(scheme) };

            Scheme2::Other(Box::new(val))
        }
    };

    // Find the end of the authority. The scheme will already have been
    // extracted.
    let authority_end = Authority::parse(&s[..])?;

    if scheme.is_none() {
        if authority_end != s.len() {
            return Err(ErrorKind::InvalidFormat.into());
        }

        let authority = Authority {
            data: unsafe { ByteStr::from_utf8_unchecked(s) },
        };

        return Ok(Uri {
            scheme: scheme.into(),
            authority,
            path_and_query: PathAndQuery::empty(),
        });
    }

    // Authority is required when absolute
    if authority_end == 0 {
        return Err(ErrorKind::InvalidFormat.into());
    }

    let authority = s.split_to(authority_end);
    let authority = Authority {
        data: unsafe { ByteStr::from_utf8_unchecked(authority) },
    };

    // When absolute, path is coered to / if empty.
    let path_and_query = if s.is_empty() {
        PathAndQuery::slash()
    } else {
        PathAndQuery::from_shared(s)?
    };

    Ok(Uri {
        scheme: scheme.into(),
        authority,
        path_and_query,
    })
}

#[cfg(not(http_uri_parts_leaf))]
impl FromStr for Uri {
    type Err = InvalidUri;

    #[inline]
    fn from_str(s: &str) -> Result<Uri, InvalidUri> {
        Uri::try_from(s.as_bytes())
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl PartialEq for Uri {
    #[cfg_attr(creusot, ensures(result == (
        uri_comparison_model(self@).scheme == uri_comparison_model(other@).scheme
            && uri_comparison_model(self@).authority == uri_comparison_model(other@).authority
            && uri_comparison_model(self@).path == uri_comparison_model(other@).path
            && uri_comparison_model(self@).query == uri_comparison_model(other@).query
    )))]
    fn eq(&self, other: &Uri) -> bool {
        if self.scheme() != other.scheme() {
            return false;
        }

        if self.authority() != other.authority() {
            return false;
        }

        let left_path = self.path().as_bytes();
        let right_path = other.path().as_bytes();
        let path_matches = left_path == right_path;
        if !path_matches {
            return false;
        }

        #[cfg(creusot)]
        {
            proof_assert! { left_path@.eq_model(right_path.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left_path@,
                    right_path@,
                    right_path.deep_model(),
                )
            };
            proof_assert! { left_path@ == right_path@ };
        }

        match (self.query(), other.query()) {
            (None, None) => true,
            (Some(left), Some(right)) => {
                let left_bytes = left.as_bytes();
                let right_bytes = right.as_bytes();
                let equal = left_bytes == right_bytes;
                if !equal {
                    return false;
                }

                #[cfg(creusot)]
                {
                    proof_assert! { left_bytes@.eq_model(right_bytes.deep_model()) };
                    proof_assert! {
                        creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                            left_bytes@,
                            right_bytes@,
                            right_bytes.deep_model(),
                        )
                    };
                    proof_assert! { left_bytes@ == right_bytes@ };
                }

                true
            }
            _ => false,
        }
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl PartialEq<str> for Uri {
    #[cfg_attr(creusot, ensures(result ==
        uri_compare_model_matches_text(self.deep_model(), other@)
    ))]
    fn eq(&self, other: &str) -> bool {
        let mut other = other.as_bytes();
        let mut absolute = false;

        if let Some(scheme) = self.scheme() {
            let scheme = scheme.as_str().as_bytes();
            absolute = true;

            if other.len() < scheme.len() + 3 {
                return false;
            }

            if !authority::authority_ascii_case_eq(
                scheme,
                &other[..scheme.len()],
            ) {
                return false;
            }

            other = &other[scheme.len()..];

            if other[0] != 58 || other[1] != 47 || other[2] != 47 {
                return false;
            }

            other = &other[3..];
        }

        if let Some(auth) = self.authority() {
            let len = auth.data.len();
            absolute = true;

            if other.len() < len {
                return false;
            }

            if !authority::authority_ascii_case_eq(
                auth.data.as_bytes(),
                &other[..len],
            ) {
                return false;
            }

            other = &other[len..];
        }

        let path = self.path();

        if other.len() < path.len() || path.as_bytes() != &other[..path.len()] {
            if absolute && path.len() == 1 && path.as_bytes()[0] == 47 {
                // PathAndQuery can be omitted, fall through
            } else {
                return false;
            }
        } else {
            other = &other[path.len()..];
        }

        if let Some(query) = self.query() {
            if other.is_empty() {
                return query.len() == 0;
            }

            if other[0] != b'?' {
                return false;
            }

            other = &other[1..];

            if other.len() < query.len() {
                return false;
            }

            if query.as_bytes() != &other[..query.len()] {
                return false;
            }

            other = &other[query.len()..];
        }

        other.len() == 0 || other[0] == b'#'
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl PartialEq<Uri> for str {
    #[cfg_attr(creusot, ensures(result ==
        uri_compare_model_matches_text(uri.deep_model(), self@)
    ))]
    fn eq(&self, uri: &Uri) -> bool {
        uri == self
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl PartialEq<&str> for Uri {
    #[cfg_attr(creusot, ensures(result ==
        uri_compare_model_matches_text(self.deep_model(), (*other)@)
    ))]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl PartialEq<Uri> for &str {
    #[cfg_attr(creusot, ensures(result ==
        uri_compare_model_matches_text(uri.deep_model(), (*self)@)
    ))]
    fn eq(&self, uri: &Uri) -> bool {
        uri == *self
    }
}

#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf)))]
impl Eq for Uri {}

/// Returns a `Uri` representing `/`
impl Default for Uri {
    #[inline]
    #[cfg_attr(creusot, ensures(uri_is_default(result@)))]
    fn default() -> Uri {
        Uri {
            scheme: Scheme::empty(),
            authority: Authority::empty(),
            path_and_query: PathAndQuery::slash(),
        }
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl fmt::Display for Uri {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^f).deep_model() == f.deep_model().concat(uri_display_text(self@)),
        Err(_) => uri_formatter_output_is_prefix(
            f.deep_model(), (^f).deep_model(), uri_display_text(self@),
        ),
    }))]
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(scheme) = self.scheme() {
            f.write_str(scheme.as_str())?;
            f.write_str("://")?;
        }

        if let Some(authority) = self.authority() {
            f.write_str(authority.as_str())?;
        }

        f.write_str(self.path())?;

        if let Some(query) = self.query() {
            f.write_str("?")?;
            f.write_str(query)?;
        }

        Ok(())
    }
}

#[cfg(not(http_uri_parts_leaf))]
impl fmt::Debug for Uri {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^f).deep_model() == f.deep_model().concat(uri_display_text(self@)),
        Err(_) => uri_formatter_output_is_prefix(
            f.deep_model(), (^f).deep_model(), uri_display_text(self@),
        ),
    }))]
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<ErrorKind> for InvalidUriParts {
    #[cfg_attr(creusot, ensures(result.deep_model() == src.deep_model()))]
    fn from(src: ErrorKind) -> InvalidUriParts {
        InvalidUriParts(src.into())
    }
}

impl fmt::Display for InvalidUriParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for InvalidUriParts {}

#[cfg(not(http_uri_parts_leaf))]
impl Hash for Uri {
    #[cfg_attr(creusot, ensures(inv(^state)))]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        if !self.scheme.inner.is_none() {
            self.scheme.hash(state);
            state.write_u8(0xff);
        }

        if let Some(auth) = self.authority() {
            auth.hash(state);
        }

        Hash::hash_slice(self.path().as_bytes(), state);

        if let Some(query) = self.query() {
            b'?'.hash(state);
            Hash::hash_slice(query.as_bytes(), state);
        }
    }
}
