use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::str;

use bytes::Bytes;

use super::{ErrorKind, InvalidUri};
use crate::byte_str::ByteStr;

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, variant, DeepModel, Int, Seq,
};
#[cfg(creusot)]
use creusot_std::prelude::{inv, View};

#[path = "scheme/protocol.rs"]
mod protocol;
#[path = "scheme/types.rs"]
mod types;

use self::protocol::Protocol;

#[doc(hidden)]
pub(super) type Scheme2<T = Box<ByteStr>> = types::Scheme2<T>;

#[path = "scheme/fixed.rs"]
mod fixed;

/// Represents the scheme component of a URI
pub struct Scheme {
    pub(super) inner: Scheme2,
}

impl Clone for Scheme {
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            Scheme2::None => Scheme2::None,
            Scheme2::Standard(protocol) => Scheme2::Standard(*protocol),
            Scheme2::Other(bytes) => Scheme2::Other(Box::new((**bytes).clone())),
        };
        Scheme { inner }
    }
}

/// Proof-visible distinction between protocol constants and preserved custom
/// scheme spellings. This model is absent from ordinary Rust builds.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum SchemeModel {
    None,
    Http,
    Https,
    Other(Seq<u8>),
}

const HTTP_SCHEME_BYTES: [u8; 4] = [104, 116, 116, 112];
const HTTPS_SCHEME_BYTES: [u8; 5] = [104, 116, 116, 112, 115];

#[cfg(creusot)]
#[logic]
#[doc(hidden)]
pub fn scheme_text_matches(model: SchemeModel, text: Seq<char>) -> bool {
    pearlite! { match model {
        SchemeModel::None => false,
        SchemeModel::Http => text.to_bytes().len() == 4
            && text.to_bytes()[0]@ == 104 && text.to_bytes()[1]@ == 116
            && text.to_bytes()[2]@ == 116 && text.to_bytes()[3]@ == 112,
        SchemeModel::Https => text.to_bytes().len() == 5
            && text.to_bytes()[0]@ == 104 && text.to_bytes()[1]@ == 116
            && text.to_bytes()[2]@ == 116 && text.to_bytes()[3]@ == 112
            && text.to_bytes()[4]@ == 115,
        SchemeModel::Other(bytes) => text.to_bytes() == bytes,
    } }
}

#[cfg(creusot)]
impl View for Scheme2 {
    type ViewTy = SchemeModel;

    #[logic(open(super))]
    fn view(self) -> Self::ViewTy {
        match self {
            Scheme2::None => SchemeModel::None,
            Scheme2::Standard(Protocol::Http) => SchemeModel::Http,
            Scheme2::Standard(Protocol::Https) => SchemeModel::Https,
            Scheme2::Other(bytes) => SchemeModel::Other((*bytes).view()),
        }
    }
}

#[cfg(creusot)]
impl View for Scheme {
    type ViewTy = SchemeModel;

    #[logic(open(super))]
    fn view(self) -> Self::ViewTy {
        self.inner.view()
    }
}

#[cfg(creusot)]
impl DeepModel for Scheme {
    type DeepModelTy = super::UriSchemeCompareModel;

    #[logic(open(super))]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { super::uri_scheme_comparison_model(self@) }
    }
}

impl Scheme {
    /// HTTP protocol scheme
    pub const HTTP: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Http),
    };

    /// HTTP protocol over TLS.
    pub const HTTPS: Scheme = Scheme {
        inner: Scheme2::Standard(Protocol::Https),
    };

    #[cfg_attr(creusot, ensures(result@ == SchemeModel::None))]
    pub(super) fn empty() -> Self {
        Scheme {
            inner: Scheme2::None,
        }
    }

    /// Return a str representation of the scheme
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::*;
    /// let scheme: Scheme = "http".parse().unwrap();
    /// assert_eq!(scheme.as_str(), "http");
    /// ```
    #[inline]
    #[requires(self@ != SchemeModel::None)]
    #[ensures(scheme_text_matches(self@, result@))]
    pub fn as_str(&self) -> &str {
        use self::types::Scheme2::*;
        use self::Protocol::*;

        match self.inner {
            Standard(Http) => {
                #[cfg(creusot)]
                proof_assert! {
                    crate::ascii::ascii_bytes_are_valid_utf8(HTTP_SCHEME_BYTES@);
                    creusot_std::std::string::valid_utf8(HTTP_SCHEME_BYTES@)
                };
                // Safety: the fixed scheme bytes are ASCII and therefore valid UTF-8.
                unsafe { str::from_utf8_unchecked(&HTTP_SCHEME_BYTES) }
            }
            Standard(Https) => {
                #[cfg(creusot)]
                proof_assert! {
                    crate::ascii::ascii_bytes_are_valid_utf8(HTTPS_SCHEME_BYTES@);
                    creusot_std::std::string::valid_utf8(HTTPS_SCHEME_BYTES@)
                };
                // Safety: the fixed scheme bytes are ASCII and therefore valid UTF-8.
                unsafe { str::from_utf8_unchecked(&HTTPS_SCHEME_BYTES) }
            }
            Other(ref v) => &**v,
            None => unreachable!(),
        }
    }
}

impl TryFrom<&[u8]> for Scheme {
    type Error = InvalidUri;
    #[inline]
    #[ensures(match result {
        Ok(scheme) => match scheme@ {
            SchemeModel::Http => s@.len() == 4
                && s@[0]@ == 104 && s@[1]@ == 116
                && s@[2]@ == 116 && s@[3]@ == 112,
            SchemeModel::Https => s@.len() == 5
                && s@[0]@ == 104 && s@[1]@ == 116
                && s@[2]@ == 116 && s@[3]@ == 112 && s@[4]@ == 115,
            SchemeModel::Other(repr) => repr == s@
                && s@.len() <= 64
                && scheme_bytes_valid(s@)
                && !(s@.len() == 4
                    && s@[0]@ == 104 && s@[1]@ == 116
                    && s@[2]@ == 116 && s@[3]@ == 112)
                && !(s@.len() == 5
                    && s@[0]@ == 104 && s@[1]@ == 116
                    && s@[2]@ == 116 && s@[3]@ == 112 && s@[4]@ == 115),
            SchemeModel::None => false,
        },
        Err(error) => (s@.len() > 64 && error.deep_model() == 11)
            || (s@.len() <= 64 && !scheme_bytes_valid(s@)
                && error.deep_model() == 1),
    })]
    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        use self::types::Scheme2::*;

        match Scheme2::parse_exact(s)? {
            None => Err(ErrorKind::InvalidScheme.into()),
            Standard(p) => Ok(Scheme {
                inner: Standard(p),
            }),
            Other(_) => {
                let bytes = Bytes::copy_from_slice(s);

                // Safety: postcondition on parse_exact() means that s and
                // hence bytes are valid UTF-8.
                #[cfg(creusot)]
                proof_assert! {
                    crate::ascii::ascii_bytes_are_valid_utf8(s@);
                    creusot_std::std::string::valid_utf8(s@)
                };
                let string = unsafe { ByteStr::from_utf8_unchecked(bytes) };

                Ok(Scheme {
                    inner: Other(Box::new(string)),
                })
            }
        }
    }
}

impl TryFrom<&str> for Scheme {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(scheme) => match scheme@ {
            SchemeModel::Http => s@.to_bytes().len() == 4
                && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112,
            SchemeModel::Https => s@.to_bytes().len() == 5
                && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112
                && s@.to_bytes()[4]@ == 115,
            SchemeModel::Other(repr) => repr == s@.to_bytes()
                && s@.to_bytes().len() <= 64
                && scheme_bytes_valid(s@.to_bytes())
                && !(s@.to_bytes().len() == 4
                    && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                    && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112)
                && !(s@.to_bytes().len() == 5
                    && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                    && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112
                    && s@.to_bytes()[4]@ == 115),
            SchemeModel::None => false,
        },
        Err(error) => (s@.to_bytes().len() > 64 && error.deep_model() == 11)
            || (s@.to_bytes().len() <= 64 && !scheme_bytes_valid(s@.to_bytes())
                && error.deep_model() == 1),
    }))]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_bytes())
    }
}

impl FromStr for Scheme {
    type Err = InvalidUri;

    #[cfg_attr(creusot, ensures(match result {
        Ok(scheme) => match scheme@ {
            SchemeModel::Http => s@.to_bytes().len() == 4
                && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112,
            SchemeModel::Https => s@.to_bytes().len() == 5
                && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112
                && s@.to_bytes()[4]@ == 115,
            SchemeModel::Other(repr) => repr == s@.to_bytes()
                && s@.to_bytes().len() <= 64
                && scheme_bytes_valid(s@.to_bytes())
                && !(s@.to_bytes().len() == 4
                    && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                    && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112)
                && !(s@.to_bytes().len() == 5
                    && s@.to_bytes()[0]@ == 104 && s@.to_bytes()[1]@ == 116
                    && s@.to_bytes()[2]@ == 116 && s@.to_bytes()[3]@ == 112
                    && s@.to_bytes()[4]@ == 115),
            SchemeModel::None => false,
        },
        Err(error) => (s@.to_bytes().len() > 64 && error.deep_model() == 11)
            || (s@.to_bytes().len() <= 64 && !scheme_bytes_valid(s@.to_bytes())
                && error.deep_model() == 1),
    }))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}

impl fmt::Debug for Scheme {
    #[cfg_attr(creusot, requires(self@ != SchemeModel::None))]
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Scheme {
    #[cfg_attr(creusot, requires(self@ != SchemeModel::None))]
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^f).deep_model() == f.deep_model().concat(
            super::uri_scheme_display_bytes(self@).map(|byte: u8| byte@),
        ),
        Err(_) => super::uri_formatter_output_is_prefix(
            f.deep_model(),
            (^f).deep_model(),
            super::uri_scheme_display_bytes(self@).map(|byte: u8| byte@),
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
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Scheme {
    #[inline]
    #[cfg_attr(creusot, requires(self@ != SchemeModel::None))]
    #[cfg_attr(creusot, ensures(scheme_text_matches(self@, result@)))]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// The URI leaf harness proves parsing and representation leaves. Scheme
// comparison is tested in the regular crate suite but excluded from that
// harness until the case-folded ByteStr model is available there.
#[cfg(not(http_uri_scheme_leaf))]
impl PartialEq for Scheme {
    #[cfg_attr(creusot, requires(self@ != SchemeModel::None
        && other@ != SchemeModel::None))]
    #[cfg_attr(creusot, ensures(result ==
        super::uri_scheme_models_equal(self@, other@)
    ))]
    fn eq(&self, other: &Scheme) -> bool {
        use self::types::Scheme2::*;
        use self::Protocol::*;

        match (&self.inner, &other.inner) {
            (&Standard(Http), &Standard(Http)) => true,
            (&Standard(Https), &Standard(Https)) => true,
            (Other(a), Other(b)) =>
                super::authority::authority_ascii_case_eq(a.as_bytes(), b.as_bytes()),
            (&None, _) | (_, &None) => unreachable!(),
            _ => false,
        }
    }
}

#[cfg(not(http_uri_scheme_leaf))]
impl Eq for Scheme {}

/// Case-insensitive equality
///
/// # Examples
///
/// ```
/// # use http::uri::Scheme;
/// let scheme: Scheme = "HTTP".parse().unwrap();
/// assert_eq!(scheme, *"http");
/// ```
#[cfg(not(http_uri_scheme_leaf))]
impl PartialEq<str> for Scheme {
    #[cfg_attr(creusot, requires(self@ != SchemeModel::None))]
    #[cfg_attr(creusot, ensures(result ==
        super::uri_scheme_comparison_matches_text(self.deep_model(), other@)
    ))]
    fn eq(&self, other: &str) -> bool {
        super::authority::authority_ascii_case_eq(
            self.as_str().as_bytes(),
            other.as_bytes(),
        )
    }
}

/// Case-insensitive equality
#[cfg(not(http_uri_scheme_leaf))]
impl PartialEq<Scheme> for str {
    #[cfg_attr(creusot, requires(other@ != SchemeModel::None))]
    #[cfg_attr(creusot, ensures(result ==
        super::uri_scheme_comparison_matches_text(other.deep_model(), self@)
    ))]
    fn eq(&self, other: &Scheme) -> bool {
        super::authority::authority_ascii_case_eq(
            self.as_bytes(),
            other.as_str().as_bytes(),
        )
    }
}

/// Case-insensitive hashing
impl Hash for Scheme {
    #[cfg_attr(creusot, ensures(inv(^state)))]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self.inner {
            Scheme2::None => (),
            Scheme2::Standard(Protocol::Http) => state.write_u8(1),
            Scheme2::Standard(Protocol::Https) => state.write_u8(2),
            Scheme2::Other(ref other) => {
                other.len().hash(state);
                for &b in other.as_bytes() {
                    state.write_u8(super::authority::authority_ascii_lowercase_byte(b));
                }
            }
        }
    }
}

// Require the scheme to not be too long in order to enable further
// optimizations later.
const MAX_SCHEME_LEN: usize = 64;

/// A byte accepted by `parse_exact`'s current scheme character table and
/// explicit colon rejection. This intentionally mirrors this implementation,
/// including `~` and the absence of an initial-alpha requirement.
#[cfg(creusot)]
#[logic(open)]
pub fn scheme_byte_valid(byte: u8) -> bool {
    pearlite! {
        byte@ == 43 || byte@ == 45 || byte@ == 46
            || (48 <= byte@ && byte@ <= 57)
            || (65 <= byte@ && byte@ <= 90)
            || (97 <= byte@ && byte@ <= 122)
            || byte@ == 126
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn scheme_bytes_valid(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i: Int> 0 <= i && i < bytes.len() ==> scheme_byte_valid(bytes[i])
    }
}

/// The bytes before `index` are valid non-delimiter scheme characters, and
/// `index` begins a `://` separator.
#[logic(open(self))]
fn scheme_delimiter_candidate(bytes: Seq<u8>, index: Int) -> bool {
    pearlite! {
        0 <= index && index + 2 < bytes.len()
            && (forall<j: Int> 0 <= j && j < index ==> scheme_byte_valid(bytes[j]))
            && bytes[index]@ == 58
            && bytes[index + 1]@ == 47
            && bytes[index + 2]@ == 47
    }
}

/// Exact result classification for the URI scheme-prefix scanner.
#[logic(open(self))]
pub(super) fn scheme_parse_matches(
    bytes: Seq<u8>,
    outcome: Result<Scheme2<usize>, InvalidUri>,
) -> bool {
    pearlite! {
        if fixed::has_http_scheme_prefix(bytes) {
            match outcome {
                Ok(Scheme2::Standard(Protocol::Http)) => true,
                _ => false,
            }
        } else if fixed::has_https_scheme_prefix(bytes) {
            match outcome {
                Ok(Scheme2::Standard(Protocol::Https)) => true,
                _ => false,
            }
        } else if bytes.len() > 3
            && exists<i: Int> 0 <= i && i < bytes.len()
                && scheme_delimiter_candidate(bytes, i)
        {
            match outcome {
                Ok(Scheme2::Other(index)) => exists<i: Int>
                    0 <= i && i < bytes.len()
                    && scheme_delimiter_candidate(bytes, i)
                    && i <= MAX_SCHEME_LEN@
                    && index@ == i,
                Err(err) => (match err.0 {
                    ErrorKind::SchemeTooLong => true,
                    _ => false,
                })
                    && exists<i: Int> 0 <= i && i < bytes.len()
                        && scheme_delimiter_candidate(bytes, i)
                        && i > MAX_SCHEME_LEN@,
                _ => false,
            }
        } else {
            match outcome {
                Ok(Scheme2::None) => true,
                _ => false,
            }
        }
    }
}

#[ensures(match result {
    Some(index) => scheme_delimiter_candidate(s@, index@),
    None => forall<i: Int> 0 <= i && i < s@.len()
        ==> !scheme_delimiter_candidate(s@, i),
})]
fn find_scheme_delimiter(s: &[u8]) -> Option<usize> {
    let mut i = 0;
    #[invariant(i@ <= s@.len())]
    #[invariant(forall<j: Int> 0 <= j && j < i@ ==> scheme_byte_valid(s@[j]))]
    #[variant(s@.len() - i@)]
    while i < s.len() {
        let byte = s[i];
        if byte == b':' {
            if s.len() - i < 3 || s[i + 1] != b'/' || s[i + 2] != b'/' {
                return None;
            }
            return Some(i);
        }

        if !scheme_byte_is_allowed(byte) {
            return None;
        }
        i += 1;
    }

    None
}

#[ensures(result == scheme_byte_valid(byte))]
fn scheme_byte_is_allowed(byte: u8) -> bool {
    matches!(
        byte,
        b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'~'
    )
}

// The scanner treats ':' as a possible delimiter and uses this exact byte
// predicate for all other characters. This preserves the former table's
// accepted bytes, including '~' and the lack of an initial-alpha check.

impl Scheme2<usize> {
    #[ensures(match result {
        Ok(Scheme2::Standard(Protocol::Http)) => fixed::is_http_bytes(s@),
        Ok(Scheme2::Standard(Protocol::Https)) => fixed::is_https_bytes(s@),
        Ok(Scheme2::Other(())) => s@.len() <= MAX_SCHEME_LEN@
            && scheme_bytes_valid(s@)
            && !fixed::is_http_bytes(s@)
            && !fixed::is_https_bytes(s@),
        Ok(Scheme2::None) => false,
        Err(err) => match err.0 {
            ErrorKind::SchemeTooLong => s@.len() > MAX_SCHEME_LEN@,
            ErrorKind::InvalidScheme => s@.len() <= MAX_SCHEME_LEN@
                && !scheme_bytes_valid(s@),
            _ => false,
        },
    })]
    fn parse_exact(s: &[u8]) -> Result<Scheme2<()>, InvalidUri> {
        if fixed::is_http(s) {
            return Ok(Protocol::Http.into());
        }

        if fixed::is_https(s) {
            return Ok(Protocol::Https.into());
        }

        if s.len() > MAX_SCHEME_LEN {
            return Err(ErrorKind::SchemeTooLong.into());
        }

        // Check the scheme alphabet, which also rejects `:` and all bytes
        // above ASCII.
        let byte_len = s.len();
        let mut i = 0;
        #[invariant(i@ <= byte_len@)]
        #[invariant(byte_len@ == s@.len())]
        #[invariant(byte_len@ <= MAX_SCHEME_LEN@)]
        #[invariant(!fixed::is_http_bytes(s@))]
        #[invariant(!fixed::is_https_bytes(s@))]
        #[invariant(forall<j: Int> 0 <= j && j < i@ ==> scheme_byte_valid(s@[j]))]
        #[variant(byte_len - i)]
        while i < byte_len {
            if !scheme_byte_is_allowed(s[i]) {
                return Err(ErrorKind::InvalidScheme.into());
            }
            i += 1;
        }

        Ok(Scheme2::Other(()))
    }

    #[ensures(scheme_parse_matches(s@, result))]
    pub(super) fn parse(s: &[u8]) -> Result<Scheme2<usize>, InvalidUri> {
        if s.len() >= 7 {
            // Check for HTTP
            if fixed::has_http_scheme(s) {
                // Prefix will be striped
                return Ok(Protocol::Http.into());
            }
        }

        if s.len() >= 8 {
            // Check for HTTPs
            if fixed::has_https_scheme(s) {
                return Ok(Protocol::Https.into());
            }
        }

        if s.len() > 3 {
            if let Some(index) = find_scheme_delimiter(s) {
                if index > MAX_SCHEME_LEN {
                    return Err(ErrorKind::SchemeTooLong.into());
                }

                return Ok(Scheme2::Other(index));
            }
        }

        Ok(Scheme2::None)
    }
}

#[doc(hidden)]
impl From<Scheme2> for Scheme {
    #[cfg_attr(creusot, ensures(result@ == src@))]
    fn from(src: Scheme2) -> Self {
        Scheme { inner: src }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scheme_eq_to_str() {
        assert_eq!(&scheme("http"), "http");
        assert_eq!(&scheme("https"), "https");
        assert_eq!(&scheme("ftp"), "ftp");
        assert_eq!(&scheme("my+funky+scheme"), "my+funky+scheme");
    }

    #[test]
    fn standard_and_custom_scheme_equality_keeps_the_variant() {
        let uppercase = Scheme::try_from("HTTP").expect("valid custom spelling");

        assert_ne!(Scheme::HTTP, uppercase);
        assert_eq!(&Scheme::HTTP, "http");
        assert_eq!(&uppercase, "http");
    }

    #[test]
    fn invalid_scheme_is_error() {
        Scheme::try_from("my_funky_scheme").expect_err("Unexpectedly valid Scheme");

        // Invalid UTF-8
        Scheme::try_from([0xC0].as_ref()).expect_err("Unexpectedly valid Scheme");
    }

    fn scheme(s: &str) -> Scheme {
        s.parse().expect(&format!("Invalid scheme: {}", s))
    }
}
