use std::convert::TryFrom;
use std::str::FromStr;
use std::{cmp, fmt, hash, str};

use bytes::Bytes;

use super::{ErrorKind, InvalidUri, MAX_LEN};
use crate::byte_str::ByteStr;
#[path = "path_scan.rs"]
mod path_scan;
use self::path_scan::{scan_path_and_query, Scanned};

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, logic, pearlite, proof_assert, requires, DeepModel, Int, Invariant, Seq, View,
};
#[cfg(creusot)]
use creusot_std::logic::OrdLogic as _;
#[cfg(creusot)]
use creusot_std::std::partial_eq::PartialEqModel as _;
#[cfg(creusot)]
use creusot_std::std::partial_ord::PartialOrdModel as _;
#[cfg(creusot)]
use creusot_std::std::str_index::utf8::{prefix_at_ascii_byte, prefix_through_ascii_byte};
#[cfg(creusot)]
use creusot_std::prelude::{inv, snapshot};

/// Proof-visible component representation. `query` is a byte offset to `?`;
/// `None` represents the `u16::MAX` sentinel.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct PathAndQueryModel {
    pub bytes: Seq<u8>,
    pub query: Option<Int>,
}

#[cfg(creusot)]
impl View for PathAndQuery {
    type ViewTy = PathAndQueryModel;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            PathAndQueryModel {
                bytes: self.data@,
                query: if self.query == NONE { None } else { Some(self.query@) },
            }
        }
    }
}

#[cfg(creusot)]
impl Invariant for PathAndQuery {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! { path_query_boundaries(self@) }
    }
}

#[cfg(creusot)]
impl DeepModel for PathAndQuery {
    type DeepModelTy = PathAndQueryCompareModel;

    #[logic]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { PathAndQueryCompareModel { raw: self.data@ } }
    }
}

/// Comparison model keeps the stored bytes while allowing the source type's
/// rendered-string comparisons to map the empty representation to `/`.
#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct PathAndQueryCompareModel {
    pub raw: Seq<u8>,
}

#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn path_and_query_rendered_bytes(raw: Seq<u8>) -> Seq<u8> {
    pearlite! { if raw.len() == 0 { Seq::singleton(47u8) } else { raw } }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic(open)]
pub fn path_and_query_display_text(model: PathAndQueryModel) -> Seq<Int> {
    pearlite! {
        path_and_query_rendered_bytes(model.bytes).map(|byte: u8| byte@)
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<Seq<Int>>
    for PathAndQueryCompareModel
{
    #[logic(open)]
    fn eq_model(self, rhs: Seq<Int>) -> bool {
        pearlite! {
            path_and_query_rendered_bytes(self.raw).len() == rhs.len()
                && forall<i: Int> 0 <= i && i < rhs.len()
                    ==> path_and_query_rendered_bytes(self.raw)[i]@ == rhs[i]
        }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<Seq<char>>
    for PathAndQueryCompareModel
{
    #[logic(open)]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { path_and_query_rendered_bytes(self.raw) == rhs.to_bytes() }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<PathAndQueryCompareModel>
    for Seq<Int>
{
    #[logic(open)]
    fn eq_model(self, rhs: PathAndQueryCompareModel) -> bool {
        pearlite! {
            self.len() == path_and_query_rendered_bytes(rhs.raw).len()
                && forall<i: Int> 0 <= i && i < self.len()
                    ==> self[i] == path_and_query_rendered_bytes(rhs.raw)[i]@
        }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_eq::PartialEqModel<PathAndQueryCompareModel>
    for Seq<char>
{
    #[logic(open)]
    fn eq_model(self, rhs: PathAndQueryCompareModel) -> bool {
        pearlite! { self.to_bytes() == path_and_query_rendered_bytes(rhs.raw) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<PathAndQueryCompareModel>
    for PathAndQueryCompareModel
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: PathAndQueryCompareModel) -> cmp::Ordering {
        pearlite! {
            path_and_query_rendered_bytes(self.raw)
                .cmp_log(path_and_query_rendered_bytes(rhs.raw))
        }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<Seq<char>>
    for PathAndQueryCompareModel
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<char>) -> cmp::Ordering {
        pearlite! { path_and_query_rendered_bytes(self.raw).cmp_log(rhs.to_bytes()) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<Seq<Int>>
    for PathAndQueryCompareModel
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: Seq<Int>) -> cmp::Ordering {
        creusot_std::std::partial_ord::seq_cmp_u8_int(
            path_and_query_rendered_bytes(self.raw), rhs,
        )
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<PathAndQueryCompareModel>
    for Seq<char>
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: PathAndQueryCompareModel) -> cmp::Ordering {
        pearlite! { self.to_bytes().cmp_log(path_and_query_rendered_bytes(rhs.raw)) }
    }
}

#[cfg(creusot)]
impl creusot_std::std::partial_ord::PartialOrdModel<PathAndQueryCompareModel>
    for Seq<Int>
{
    #[logic(open)]
    fn partial_cmp_model(self, rhs: PathAndQueryCompareModel) -> cmp::Ordering {
        creusot_std::std::seq_ord::reverse_ordering(
            creusot_std::std::partial_ord::seq_cmp_u8_int(
                path_and_query_rendered_bytes(rhs.raw), self,
            ),
        )
    }
}

#[cfg(creusot)]
#[logic(opaque)]
#[requires(
    creusot_std::std::fmt::formatter_extends(before, middle)
        && creusot_std::std::fmt::formatter_extends(middle, after)
)]
#[ensures(result)]
#[ensures(result == creusot_std::std::fmt::formatter_extends(before, after))]
fn path_formatter_extends_transitive(
    before: Seq<Int>,
    middle: Seq<Int>,
    after: Seq<Int>,
) -> bool {
    proof_assert! {
        forall<x: Seq<Int>, y: Seq<Int>>
            middle == before.concat(x) && after == middle.concat(y) ==> {
                creusot_std::std::str_index::utf8::seq_concat_assoc(before, x, y);
                proof_assert! { after == before.concat(x.concat(y)) };
                creusot_std::std::fmt::formatter_extends(before, after)
            }
    };
    true
}

/// The cached query position is an in-bounds ASCII `?` delimiter. The UTF-8
/// split helpers derive the corresponding character-prefix witness from this
/// byte fact.
#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn path_query_boundaries(model: PathAndQueryModel) -> bool {
    pearlite! {
        match model.query {
            None => true,
            Some(query) => 0 <= query && query < model.bytes.len()
                && model.bytes[query]@ == 63,
        }
    }
}

#[cfg(creusot)]
#[logic(open(crate))]
#[doc(hidden)]
pub fn path_component_bytes(model: PathAndQueryModel) -> Seq<u8> {
    pearlite! {
        let end = match model.query { None => model.bytes.len(), Some(query) => query };
        if end == 0 { Seq::singleton(47u8) } else { model.bytes.subsequence(0, end) }
    }
}

/// Exact retained-input and cached-query relation for a successful
/// `from_shared` call. The result length canonically fixes the fragment
/// boundary, avoiding a second existential search over the input.
#[cfg(creusot)]
#[logic(open(crate))]
#[doc(hidden)]
pub fn path_projection_matches_input(input: Seq<u8>, model: PathAndQueryModel) -> bool {
    pearlite! {
        let end = model.bytes.len();
        end <= input.len()
            && (end == input.len() || input[end]@ == 35)
            && (forall<i: Int> 0 <= i && i < end ==> input[i]@ != 35)
            && model.bytes == input.subsequence(0, end)
            && match model.query {
                None => forall<i: Int> 0 <= i && i < end ==> input[i]@ != 63,
                Some(query) => 0 <= query && query < end
                    && input[query]@ == 63
                    && (forall<i: Int> 0 <= i && i < query ==> input[i]@ != 63),
            }
    }
}

// The path API maps an empty stored component to "/". Keep the UTF-8
// witness in bytes so the proof does not depend on a named string literal's
// logical encoding.
#[cfg_attr(creusot, ensures(result@.to_bytes() == Seq::singleton(47u8)))]
fn slash_str() -> &'static str {
    let slash = &[47u8];
    #[cfg(creusot)]
    proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(slash@);
        creusot_std::std::string::valid_utf8(slash@)
    };
    // SAFETY: the one-byte sequence contains ASCII slash, hence valid UTF-8.
    unsafe { str::from_utf8_unchecked(slash) }
}

#[cfg_attr(creusot, ensures(result@.to_bytes() == Seq::singleton(42u8)))]
fn star_str() -> &'static str {
    let star = &[42u8];
    #[cfg(creusot)]
    proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(star@);
        creusot_std::std::string::valid_utf8(star@)
    };
    // SAFETY: the one-byte sequence contains ASCII `*`, hence valid UTF-8.
    unsafe { str::from_utf8_unchecked(star) }
}

/// Represents the path component of a URI
pub struct PathAndQuery {
    pub(super) data: ByteStr,
    pub(super) query: u16,
}

impl Clone for PathAndQuery {
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn clone(&self) -> Self {
        PathAndQuery {
            data: self.data.clone(),
            query: self.query,
        }
    }
}

const NONE: u16 = u16::MAX;

impl PathAndQuery {
    // Not public while `bytes` is unstable.
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(src@, value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    pub(super) fn from_shared(mut src: Bytes) -> Result<Self, InvalidUri> {
        let Scanned {
            query,
            fragment,
            is_maybe_not_utf8,
        } = scan_path_and_query(&src)?;

        #[cfg(creusot)]
        let original = snapshot!(src@);
        #[cfg(creusot)]
        let retained_end = snapshot!(match fragment {
            None => src@.len(),
            Some(index) => index@,
        });

        #[cfg(creusot)]
        path_scan::successful_scan_has_delimiters(
            original,
            query,
            fragment,
            is_maybe_not_utf8,
        );

        if let Some(i) = fragment {
            src.truncate(i as usize);
        }

        #[cfg(creusot)]
        {
            proof_assert!(src@.len() == *retained_end);
            proof_assert!(src@ == (*original).subsequence(0, *retained_end));
        }

        let data = if is_maybe_not_utf8 {
            match ByteStr::from_utf8(src) {
                Ok(value) => value,
                Err(_) => return Err(ErrorKind::InvalidUriChar.into()),
            }
        } else {
            #[cfg(creusot)]
            {
                proof_assert! {
                    forall<i: Int> 0 <= i && i < src@.len() ==> src@[i]@ < 128
                };
                proof_assert! {
                    crate::ascii::ascii_bytes_are_valid_utf8(src@);
                    true
                };
            }
            unsafe { ByteStr::from_utf8_unchecked(src) }
        };

        Ok(PathAndQuery { data, query })
    }

    /// Convert a `PathAndQuery` from a static string.
    ///
    /// This function will not perform any copying, however the string is
    /// checked to ensure that it is valid.
    ///
    /// # Panics
    ///
    /// This function panics if the argument is an invalid path and query.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::uri::*;
    /// let v = PathAndQuery::from_static("/hello?world");
    ///
    /// assert_eq!(v.path(), "/hello");
    /// assert_eq!(v.query(), Some("world"));
    /// ```
    #[cfg_attr(creusot, requires(super::path_static_input_is_valid(src@.to_bytes())))]
    #[cfg_attr(creusot, ensures(result@.bytes == src@.to_bytes()))]
    #[cfg_attr(creusot, ensures(match result@.query {
        None => forall<i: Int> 0 <= i && i < src@.to_bytes().len()
            ==> src@.to_bytes()[i]@ != 63,
        Some(query) => 0 <= query && query < src@.to_bytes().len()
            && src@.to_bytes()[query]@ == 63
            && forall<i: Int> 0 <= i && i < query
                ==> src@.to_bytes()[i]@ != 63,
    }))]
    #[cfg_attr(creusot, ensures(path_query_boundaries(result@)))]
    #[inline]
    pub const fn from_static(src: &'static str) -> Self {
        match scan_path_and_query(src.as_bytes()) {
            Ok(Scanned {
                query,
                fragment: None,
                is_maybe_not_utf8: false,
            }) => PathAndQuery {
                data: ByteStr::from_static(src),
                query,
            },
            // Yes, we reject fragments and non-utf8
            _ => panic!("static str is not valid path"),
        }
    }

    /// Attempt to convert a `Bytes` buffer to a `PathAndQuery`.
    ///
    /// This will try to prevent a copy if the type passed is the type used
    /// internally, and will copy the data if it is not.
    // This optimization relies on a dyn Any downcast, which Creusot does not
    // model. It affects allocation/sharing only; the verified constructors
    // below use the same scanner and return the same byte value.
    #[cfg(not(any(http_uri_leaf, http_uri_parts_leaf)))]
    pub fn from_maybe_shared<T>(src: T) -> Result<Self, InvalidUri>
    where
        T: AsRef<[u8]> + 'static,
    {
        if_downcast_into!(T, Bytes, src, {
            return PathAndQuery::from_shared(src);
        });

        PathAndQuery::try_from(src.as_ref())
    }

    #[cfg_attr(creusot, ensures(result@.bytes == Seq::<u8>::empty()))]
    #[cfg_attr(creusot, ensures(result@.query == None))]
    #[cfg_attr(creusot, ensures(path_query_boundaries(result@)))]
    pub(super) fn empty() -> Self {
        PathAndQuery {
            data: ByteStr::new(),
            query: NONE,
        }
    }

    #[cfg_attr(creusot, ensures(result@.bytes == Seq::singleton(47u8)))]
    #[cfg_attr(creusot, ensures(result@.query == None))]
    #[cfg_attr(creusot, ensures(path_query_boundaries(result@)))]
    pub(super) fn slash() -> Self {
        PathAndQuery {
            data: ByteStr::from_static(slash_str()),
            query: NONE,
        }
    }

    #[cfg_attr(creusot, ensures(result@.bytes == Seq::singleton(42u8)))]
    #[cfg_attr(creusot, ensures(result@.query == None))]
    #[cfg_attr(creusot, ensures(path_query_boundaries(result@)))]
    pub(super) fn star() -> Self {
        PathAndQuery {
            data: ByteStr::from_static(star_str()),
            query: NONE,
        }
    }

    /// Returns the path component
    ///
    /// The path component is **case sensitive**.
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
    /// ```
    /// # use http::uri::*;
    ///
    /// let path_and_query: PathAndQuery = "/hello/world".parse().unwrap();
    ///
    /// assert_eq!(path_and_query.path(), "/hello/world");
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(path_query_boundaries(self@)))]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == path_component_bytes(self@)))]
    pub fn path(&self) -> &str {
        #[cfg(creusot)]
        if self.query != NONE {
            let path_and_query = self.as_str();
            proof_assert! {
                prefix_at_ascii_byte(path_and_query@, self.query@)
                    .0.to_bytes().len() == self.query@
            };
        }

        let ret = if self.query == NONE {
            &*self.data
        } else {
            self.data.split_at(self.query as usize).0
        };

        if ret.len() == 0 {
            return slash_str();
        }

        ret
    }

    /// Returns the query string component
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
    /// With a query string component
    ///
    /// ```
    /// # use http::uri::*;
    /// let path_and_query: PathAndQuery = "/hello/world?key=value&foo=bar".parse().unwrap();
    ///
    /// assert_eq!(path_and_query.query(), Some("key=value&foo=bar"));
    /// ```
    ///
    /// Without a query string component
    ///
    /// ```
    /// # use http::uri::*;
    /// let path_and_query: PathAndQuery = "/hello/world".parse().unwrap();
    ///
    /// assert!(path_and_query.query().is_none());
    /// ```
    #[inline]
    #[cfg_attr(creusot, requires(path_query_boundaries(self@)))]
    #[cfg_attr(creusot, ensures(match (self@.query, result) {
        (None, None) => true,
        (Some(offset), Some(value)) => value@.to_bytes()
            == self@.bytes.subsequence(offset + 1, self@.bytes.len()),
        _ => false,
    }))]
    pub fn query(&self) -> Option<&str> {
        if self.query == NONE {
            None
        } else {
            #[cfg(creusot)]
            let path_and_query = self.as_str();
            #[cfg(creusot)]
            proof_assert! {
                prefix_through_ascii_byte(path_and_query@, self.query@)
                    .0.to_bytes().len() == self.query@ + 1
            };

            let i = self.query + 1;
            Some(self.data.split_at(i as usize).1)
        }
    }

    /// Returns the path and query as a string component.
    ///
    /// # Examples
    ///
    /// With a query string component
    ///
    /// ```
    /// # use http::uri::*;
    /// let path_and_query: PathAndQuery = "/hello/world?key=value&foo=bar".parse().unwrap();
    ///
    /// assert_eq!(path_and_query.as_str(), "/hello/world?key=value&foo=bar");
    /// ```
    ///
    /// Without a query string component
    ///
    /// ```
    /// # use http::uri::*;
    /// let path_and_query: PathAndQuery = "/hello/world".parse().unwrap();
    ///
    /// assert_eq!(path_and_query.as_str(), "/hello/world");
    /// ```
    #[inline]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == if self@.bytes.len() == 0 {
        Seq::singleton(47u8)
    } else {
        self@.bytes
    }))]
    pub fn as_str(&self) -> &str {
        let ret = &*self.data;
        if ret.len() == 0 {
            return slash_str();
        }
        ret
    }
}

impl TryFrom<&[u8]> for PathAndQuery {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(s@, value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        PathAndQuery::from_shared(Bytes::copy_from_slice(s))
    }
}

impl TryFrom<&str> for PathAndQuery {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(s@.to_bytes(), value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for PathAndQuery {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(vec@, value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        PathAndQuery::from_shared(vec.into())
    }
}

impl TryFrom<String> for PathAndQuery {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(s@.to_bytes(), value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        PathAndQuery::from_shared(s.into())
    }
}

impl TryFrom<&String> for PathAndQuery {
    type Error = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(s@.to_bytes(), value@)
            && path_query_boundaries(value@),
        Err(_) => true,
    }))]
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        let text: &str = s;
        TryFrom::try_from(text.as_bytes())
    }
}

impl FromStr for PathAndQuery {
    type Err = InvalidUri;
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(value) => path_projection_matches_input(s@.to_bytes(), value@),
        Err(_) => true,
    }))]
    fn from_str(s: &str) -> Result<Self, InvalidUri> {
        TryFrom::try_from(s)
    }
}

impl fmt::Debug for PathAndQuery {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^f).deep_model() ==
            f.deep_model().concat(path_and_query_display_text(self@)),
        Err(_) => super::uri_formatter_output_is_prefix(
            f.deep_model(), (^f).deep_model(), path_and_query_display_text(self@),
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

impl fmt::Display for PathAndQuery {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^fmt).deep_model() ==
            fmt.deep_model().concat(path_and_query_display_text(self@)),
        Err(_) => super::uri_formatter_output_is_prefix(
            fmt.deep_model(), (^fmt).deep_model(), path_and_query_display_text(self@),
        ),
    }))]
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            fmt.deep_model(),
            (^fmt).deep_model()
        ))
    )]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.len() != 0 {
            match self.data.as_bytes()[0] {
                b'/' | b'*' => fmt.write_str(&self.data),
                _ => {
                    #[cfg(creusot)]
                    let before = snapshot!(fmt.deep_model());
                    let first = fmt.write_str("/");
                    #[cfg(creusot)]
                    let after_first = snapshot!(fmt.deep_model());
                    first?;

                    let second = fmt.write_str(&self.data);
                    #[cfg(creusot)]
                    proof_assert! {
                        path_formatter_extends_transitive(
                            *before,
                            *after_first,
                            fmt.deep_model(),
                        )
                    };
                    second
                }
            }
        } else {
            fmt.write_str("/")
        }
    }
}

impl hash::Hash for PathAndQuery {
    #[cfg_attr(creusot, ensures(inv(^state)))]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

// ===== PartialEq / PartialOrd =====

impl PartialEq for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &PathAndQuery) -> bool {
        self.data == other.data
    }
}

impl Eq for PathAndQuery {}

impl PartialEq<str> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &str) -> bool {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

impl PartialEq<PathAndQuery> for &str {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &PathAndQuery) -> bool {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

impl PartialEq<&str> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &&str) -> bool {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

impl PartialEq<PathAndQuery> for str {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &PathAndQuery) -> bool {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

impl PartialEq<String> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &String) -> bool {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

impl PartialEq<PathAndQuery> for String {
    #[inline]
    #[cfg_attr(creusot, ensures(result ==
        self.deep_model().eq_model(other.deep_model())
    ))]
    fn eq(&self, other: &PathAndQuery) -> bool {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        let equal = left == right;
        #[cfg(creusot)]
        if equal {
            proof_assert! { left@.eq_model(right.deep_model()) };
            proof_assert! {
                creusot_std::std::partial_eq::seq_eq_u8_int_view_transport(
                    left@,
                    right@,
                    right.deep_model(),
                )
            };
            proof_assert! { left@ == right@ };
        }
        equal
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &PathAndQuery) -> Option<cmp::Ordering> {
        let left = self.as_str().as_bytes();
        let right = other.as_str().as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! {
                left@ == path_and_query_rendered_bytes(self.deep_model().raw)
            };
            proof_assert! {
                right@ == path_and_query_rendered_bytes(other.deep_model().raw)
            };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<str> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! {
                left@ == path_and_query_rendered_bytes(self.deep_model().raw)
            };
            proof_assert! { right@ == other.deep_model().to_bytes() };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<PathAndQuery> for str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &PathAndQuery) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! { left@ == self.deep_model().to_bytes() };
            proof_assert! {
                right@ == path_and_query_rendered_bytes(other.deep_model().raw)
            };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<&str> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! {
                left@ == path_and_query_rendered_bytes(self.deep_model().raw)
            };
            proof_assert! { right@ == other.deep_model().to_bytes() };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<PathAndQuery> for &str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &PathAndQuery) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! { left@ == self.deep_model().to_bytes() };
            proof_assert! {
                right@ == path_and_query_rendered_bytes(other.deep_model().raw)
            };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<String> for PathAndQuery {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        let left = self.as_str().as_bytes();
        let right = other.as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! {
                left@ == path_and_query_rendered_bytes(self.deep_model().raw)
            };
            proof_assert! { right@ == other.deep_model().to_bytes() };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

#[cfg(not(http_uri_path_component_leaf))]
impl PartialOrd<PathAndQuery> for String {
    #[inline]
    #[cfg_attr(creusot, ensures(result == Some(
        self.deep_model().partial_cmp_model(other.deep_model())
    )))]
    fn partial_cmp(&self, other: &PathAndQuery) -> Option<cmp::Ordering> {
        let left = self.as_bytes();
        let right = other.as_str().as_bytes();
        #[cfg(creusot)]
        {
            proof_assert! { left@ == self.deep_model().to_bytes() };
            proof_assert! {
                right@ == path_and_query_rendered_bytes(other.deep_model().raw)
            };
            proof_assert! {
                creusot_std::std::partial_ord::seq_cmp_u8_int_pair_transport_preserved(
                    left@,
                    left.deep_model(),
                    right@,
                    right.deep_model(),
                )
            };
        }
        left.partial_cmp(right)
    }
}

// Scanner implementation that is `const fn`, usable by both `from_static`
// and `from_shared`.
// =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_to_self_of_same_path() {
        let p1: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        let p2: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        assert_eq!(p1, p2);
        assert_eq!(p2, p1);
    }

    #[test]
    fn not_equal_to_self_of_different_path() {
        let p1: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        let p2: PathAndQuery = "/world&foo=bar".parse().unwrap();
        assert_ne!(p1, p2);
        assert_ne!(p2, p1);
    }

    #[test]
    fn equates_with_a_str() {
        let path_and_query: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        assert_eq!(&path_and_query, "/hello/world&foo=bar");
        assert_eq!("/hello/world&foo=bar", &path_and_query);
        assert_eq!(path_and_query, "/hello/world&foo=bar");
        assert_eq!("/hello/world&foo=bar", path_and_query);
    }

    #[test]
    fn not_equal_with_a_str_of_a_different_path() {
        let path_and_query: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        // as a reference
        assert_ne!(&path_and_query, "/hello&foo=bar");
        assert_ne!("/hello&foo=bar", &path_and_query);
        // without reference
        assert_ne!(path_and_query, "/hello&foo=bar");
        assert_ne!("/hello&foo=bar", path_and_query);
    }

    #[test]
    fn equates_with_a_string() {
        let path_and_query: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        assert_eq!(path_and_query, "/hello/world&foo=bar".to_string());
        assert_eq!("/hello/world&foo=bar".to_string(), path_and_query);
    }

    #[test]
    fn not_equal_with_a_string_of_a_different_path() {
        let path_and_query: PathAndQuery = "/hello/world&foo=bar".parse().unwrap();
        assert_ne!(path_and_query, "/hello&foo=bar".to_string());
        assert_ne!("/hello&foo=bar".to_string(), path_and_query);
    }

    #[test]
    fn compares_to_self() {
        let p1: PathAndQuery = "/a/world&foo=bar".parse().unwrap();
        let p2: PathAndQuery = "/b/world&foo=bar".parse().unwrap();
        assert!(p1 < p2);
        assert!(p2 > p1);
    }

    #[test]
    fn compares_with_a_str() {
        let path_and_query: PathAndQuery = "/b/world&foo=bar".parse().unwrap();
        // by ref
        assert!(&path_and_query < "/c/world&foo=bar");
        assert!("/c/world&foo=bar" > &path_and_query);
        assert!(&path_and_query > "/a/world&foo=bar");
        assert!("/a/world&foo=bar" < &path_and_query);

        // by val
        assert!(path_and_query < "/c/world&foo=bar");
        assert!("/c/world&foo=bar" > path_and_query);
        assert!(path_and_query > "/a/world&foo=bar");
        assert!("/a/world&foo=bar" < path_and_query);
    }

    #[test]
    fn compares_with_a_string() {
        let path_and_query: PathAndQuery = "/b/world&foo=bar".parse().unwrap();
        assert!(path_and_query < "/c/world&foo=bar".to_string());
        assert!("/c/world&foo=bar".to_string() > path_and_query);
        assert!(path_and_query > "/a/world&foo=bar".to_string());
        assert!("/a/world&foo=bar".to_string() < path_and_query);
    }

    #[test]
    fn ignores_valid_percent_encodings() {
        assert_eq!("/a%20b", pq("/a%20b?r=1").path());
        assert_eq!("qr=%31", pq("/a/b?qr=%31").query().unwrap());
    }

    #[test]
    fn ignores_invalid_percent_encodings() {
        assert_eq!("/a%%b", pq("/a%%b?r=1").path());
        assert_eq!("/aaa%", pq("/aaa%").path());
        assert_eq!("/aaa%", pq("/aaa%?r=1").path());
        assert_eq!("/aa%2", pq("/aa%2").path());
        assert_eq!("/aa%2", pq("/aa%2?r=1").path());
        assert_eq!("qr=%3", pq("/a/b?qr=%3").query().unwrap());
    }

    #[test]
    fn allow_utf8_in_path() {
        assert_eq!("/🍕", pq("/🍕").path());
    }

    #[test]
    fn allow_utf8_in_query() {
        assert_eq!(Some("pizza=🍕"), pq("/test?pizza=🍕").query());
    }

    #[test]
    fn rejects_invalid_utf8_in_path() {
        PathAndQuery::try_from(&[b'/', 0xFF][..]).expect_err("reject invalid utf8");
    }

    #[test]
    fn rejects_invalid_utf8_in_query() {
        PathAndQuery::try_from(&[b'/', b'a', b'?', 0xFF][..]).expect_err("reject invalid utf8");
    }

    #[test]
    fn rejects_empty_string() {
        PathAndQuery::try_from("").expect_err("reject empty str");
    }

    #[test]
    fn requires_starting_with_slash() {
        PathAndQuery::try_from("sneaky").expect_err("reject missing slash");
    }

    #[test]
    fn rejects_del_in_path() {
        PathAndQuery::try_from(&[b'/', 0x7F][..]).expect_err("reject DEL");
    }

    #[test]
    fn rejects_del_in_query() {
        PathAndQuery::try_from(&[b'/', b'a', b'?', 0x7F][..]).expect_err("reject DEL");
    }

    #[test]
    fn rejects_too_long_path_and_query() {
        let path = format!("/{}?query", "a".repeat(MAX_LEN));
        let err = PathAndQuery::try_from(path).expect_err("reject overly long path and query");
        assert_eq!(err.0, ErrorKind::TooLong);
    }

    #[test]
    fn accepts_max_length_path_and_query() {
        let path = format!("/{}?", "a".repeat(MAX_LEN - 2));
        let path_and_query = PathAndQuery::try_from(path).expect("accept maximum length");
        assert_eq!(path_and_query.as_str().len(), MAX_LEN);
        assert_eq!(path_and_query.query(), Some(""));
    }

    #[test]
    fn json_is_fine() {
        assert_eq!(
            r#"/{"bread":"baguette"}"#,
            pq(r#"/{"bread":"baguette"}"#).path()
        );
    }

    fn pq(s: &str) -> PathAndQuery {
        s.parse().expect(&format!("parsing {}", s))
    }
}
