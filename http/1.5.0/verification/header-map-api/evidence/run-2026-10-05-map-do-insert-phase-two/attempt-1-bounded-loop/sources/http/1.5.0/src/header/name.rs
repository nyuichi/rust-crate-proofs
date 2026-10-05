use crate::byte_str::ByteStr;
use bytes::{Bytes, BytesMut};

use std::borrow::Borrow;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

#[cfg(creusot)]
#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, variant, DeepModel, Int,
    Invariant, Seq, View,
};
#[cfg(creusot)]
use creusot_std::std::cmp::PartialEq;
#[cfg(creusot)]
use creusot_std::std::ops::{FnExt as _, FnOnceExt as _};
#[cfg(creusot)]
use creusot_std::std::partial_eq::PartialEqModel;
#[cfg(creusot)]
use creusot_std::std::BorrowModel;

/// Represents an HTTP header field name
///
/// Header field names identify the header. Header sets may include multiple
/// headers with the same name. The HTTP specification defines a number of
/// standard headers, but HTTP messages may include non-standard header names as
/// well as long as they adhere to the specification.
///
/// `HeaderName` is used as the [`HeaderMap`] key. Constants are available for
/// all standard header names in the [`header`] module.
///
/// # Representation
///
/// `HeaderName` represents standard header names using an `enum`, as such they
/// will not require an allocation for storage. All custom header names are
/// lower cased upon conversion to a `HeaderName` value. This avoids the
/// overhead of dynamically doing lower case conversion during the hash code
/// computation and the comparison operation.
///
/// [`HeaderMap`]: struct.HeaderMap.html
/// [`header`]: index.html
#[derive(Eq, PartialEq, Hash)]
pub struct HeaderName {
    inner: Repr<Custom>,
}

impl Clone for HeaderName {
    #[cfg_attr(creusot, ensures(result.deep_model() == self.deep_model()))]
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            Repr::Standard(header) => Repr::Standard(*header),
            Repr::Custom(Custom(bytes)) => Repr::Custom(Custom(bytes.clone())),
        };
        HeaderName { inner }
    }
}

// Almost a full `HeaderName`
#[derive(Hash)]
pub struct HdrName<'a> {
    inner: Repr<MaybeLower<'a>>,
}

impl fmt::Debug for HdrName<'_> {
    #[cfg_attr(creusot, ensures(creusot_std::std::fmt::formatter_extends(
        formatter.deep_model(), (^formatter).deep_model()
    )))]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HdrName")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Clone)]
#[cfg_attr(not(creusot), derive(Eq, PartialEq))]
enum Repr<T> {
    Standard(StandardHeader),
    Custom(T),
}

impl<T: fmt::Debug> fmt::Debug for Repr<T> {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            formatter.deep_model(),
            (^formatter).deep_model()
        ))
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Repr::Standard(header) => formatter.debug_tuple("Standard").field(header).finish(),
            Repr::Custom(value) => formatter.debug_tuple("Custom").field(value).finish(),
        }
    }
}

impl<T: Hash> Hash for Repr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Repr::Standard(header) => {
                0isize.hash(state);
                header.hash(state);
            }
            Repr::Custom(value) => {
                1isize.hash(state);
                value.hash(state);
            }
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub enum ReprDeepModel<T> {
    Standard(StandardHeaderModel),
    Custom(T),
}

// Used to hijack the Hash impl
#[derive(Clone, Eq, PartialEq)]
struct Custom(ByteStr);

impl fmt::Debug for Custom {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            formatter.deep_model(),
            (^formatter).deep_model()
        ))
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Custom").field(&self.0).finish()
    }
}

#[derive(Clone)]
// Invariant: If lower then buf is valid UTF-8.
struct MaybeLower<'a> {
    buf: &'a [u8],
    lower: bool,
}

impl fmt::Debug for MaybeLower<'_> {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            formatter.deep_model(),
            (^formatter).deep_model()
        ))
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MaybeLower")
            .field("buf", &self.buf)
            .field("lower", &self.lower)
            .finish()
    }
}

#[cfg(creusot)]
impl DeepModel for MaybeLower<'_> {
    type DeepModelTy = (Seq<u8>, bool);

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { (self.buf@, self.lower) }
    }
}

/// A possible error when converting a `HeaderName` from another type.
pub struct InvalidHeaderName {
    _priv: (),
}

macro_rules! standard_headers {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident, $upcase:ident, $rank:literal, $name_bytes:literal, $name_str:literal,
                [$($name_byte:literal),*], $matcher:ident);
        )+
    ) => {
        #[derive(Clone, Copy)]
        enum StandardHeader {
            $(
                $konst,
            )+
        }

        const STANDARD_HEADER_DEBUG_NAMES: [&str; 81] = [
            $(stringify!($konst),)+
        ];

        $(
            standard_header_byte_matcher!($matcher, [$($name_byte),*]);
        )+

        // Test-only reference types retain rustc's original derive behavior so
        // the manual implementations below can be checked at the hasher-call
        // boundary, including enum tags and callback ordering.
        #[cfg(test)]
        #[derive(Clone, Copy, Debug, Hash)]
        enum LegacyStandardHeader {
            $(
                $konst,
            )+
        }

        #[cfg(test)]
        #[derive(Debug, Hash)]
        enum LegacyRepr<T> {
            Standard(LegacyStandardHeader),
            Custom(T),
        }

        #[cfg(creusot)]
        #[doc(hidden)]
        #[derive(Clone, Copy)]
        #[allow(missing_debug_implementations)]
        pub enum StandardHeaderModel {
            $(
                $konst,
            )+
        }

        #[cfg(creusot)]
        impl StandardHeaderModel {
            #[logic(open(self))]
            fn rank(self) -> Int {
                pearlite! {
                    match self {
                        $(StandardHeaderModel::$konst => $rank,)+
                    }
                }
            }

            #[logic(open(self))]
            #[doc(hidden)]
            pub fn byte_view(self) -> Seq<u8> {
                pearlite! {
                    match self {
                        $(StandardHeaderModel::$konst => seq![$($name_byte),*],)+
                    }
                }
            }
        }

        #[cfg(creusot)]
        #[logic(open)]
        fn standard_header_model_from_rank(rank: Int) -> Option<StandardHeaderModel> {
            pearlite! {
                $( if rank == $rank {
                    Some(StandardHeaderModel::$konst)
                } else )+
                { None }
            }
        }

        // The inverse law is proved for one model value at a time. Equality
        // can then compare compact ranks without a Cartesian enum match.
        #[cfg(creusot)]
        #[logic(opaque)]
        #[ensures(result)]
        #[ensures(result == (standard_header_model_from_rank(model.rank()) == Some(model)))]
        fn standard_header_rank_roundtrip(model: StandardHeaderModel) -> bool {
            match model {
                $(StandardHeaderModel::$konst => {
                    proof_assert! {
                        standard_header_model_from_rank($rank)
                            == Some(StandardHeaderModel::$konst)
                    };
                    true
                },)+
            }
        }

        $(
            $(#[$docs])*
            pub const $upcase: HeaderName = HeaderName {
                inner: Repr::Standard(StandardHeader::$konst),
            };
        )+

        impl StandardHeader {
                #[inline]
                #[cfg_attr(creusot, ensures(result@ == self.deep_model().rank()))]
                #[cfg_attr(creusot, ensures(result@ < 81))]
            fn rank(&self) -> u8 {
                match *self {
                    $(StandardHeader::$konst => $rank,)+
                }
            }

            #[cfg(creusot)]
            #[logic(open(self))]
            fn byte_view(self) -> Seq<u8> {
                self.model().byte_view()
            }

            #[cfg(creusot)]
            #[logic(open(self))]
            fn model(self) -> StandardHeaderModel {
                match self {
                    $(StandardHeader::$konst => StandardHeaderModel::$konst,)+
                }
            }

            #[inline]
            #[cfg_attr(creusot, ensures(result@.to_bytes() == self.byte_view()))]
            fn as_str(&self) -> &'static str {
                match *self {
                    $(StandardHeader::$konst => standard_spellings::$konst(),)+
                }
            }

            #[cfg_attr(creusot, ensures(match result {
                Some(header) => name_bytes@ == header.byte_view(),
                None => forall<header: StandardHeader>
                    name_bytes@ != header.byte_view(),
            }))]
            const fn from_bytes(name_bytes: &[u8]) -> Option<StandardHeader> {
                $(
                    if $matcher(name_bytes) {
                        return Some(StandardHeader::$konst);
                    }
                )+
                #[cfg(creusot)]
                proof_assert! {
                    forall<header: StandardHeader>
                        standard_header_absence_case(name_bytes@, header)
                };
                None
            }
        }

        #[cfg(creusot)]
        #[logic(opaque)]
        #[ensures(result)]
        #[ensures(result == ((
            $(bytes != seq![$($name_byte),*] &&)+ true
        ) ==> bytes != header.byte_view()))]
        fn standard_header_absence_case(bytes: Seq<u8>, header: StandardHeader) -> bool {
            match header {
                $(StandardHeader::$konst => true,)+
            }
        }

        impl fmt::Debug for StandardHeader {
            #[cfg_attr(
                creusot,
                ensures(creusot_std::std::fmt::formatter_extends(
                    formatter.deep_model(),
                    (^formatter).deep_model()
                ))
            )]
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let rank = self.rank() as usize;
                formatter.write_str(STANDARD_HEADER_DEBUG_NAMES.as_slice()[rank])
            }
        }

        // Keep UTF-8 reasoning local to each standard spelling. A single
        // match over all standard names makes the unchecked conversion VC
        // needlessly carry every spelling at once.
        mod standard_spellings {
            #[cfg(creusot)]
            use super::*;

            $(
                #[allow(non_snake_case)]
                #[inline]
                #[cfg_attr(creusot, ensures(result@.to_bytes() == seq![$($name_byte),*]))]
                pub(super) fn $konst() -> &'static str {
                    let bytes: &'static [u8] = &[$($name_byte),*];
                    #[cfg(creusot)]
                    proof_assert! {
                        crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
                        creusot_std::std::string::valid_utf8(bytes@)
                    };
                    // Safety: this spelling's audited byte list is ASCII, and
                    // the proof above establishes UTF-8 validity.
                    unsafe { std::str::from_utf8_unchecked(bytes) }
                }
            )+
        }

        #[cfg(creusot)]
        impl DeepModel for StandardHeader {
            type DeepModelTy = StandardHeaderModel;

            #[logic(open(self))]
            fn deep_model(self) -> Self::DeepModelTy {
                self.model()
            }
        }

        impl PartialEq for StandardHeader {
            #[inline]
            #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
            fn eq(&self, other: &Self) -> bool {
                let self_rank = self.rank();
                let other_rank = other.rank();
                #[cfg(creusot)]
                proof_assert!(standard_header_rank_roundtrip(self.deep_model()));
                #[cfg(creusot)]
                proof_assert!(standard_header_rank_roundtrip(other.deep_model()));
                self_rank == other_rank
            }
        }

        impl Eq for StandardHeader {}

        impl Hash for StandardHeader {
            fn hash<H: Hasher>(&self, state: &mut H) {
                (self.rank() as isize).hash(state)
            }
        }

        #[cfg(test)]
        const TEST_HEADERS: &'static [(StandardHeader, &'static [u8])] = &[
            $(
            (StandardHeader::$konst, $name_bytes),
            )+
        ];

        #[cfg(test)]
        const TEST_HEADER_STRINGS: &'static [(StandardHeader, &'static str)] = &[
            $(
            (StandardHeader::$konst, $name_str),
            )+
        ];

        #[cfg(test)]
        const TEST_HASH_HEADERS: &'static [(StandardHeader, LegacyStandardHeader)] = &[
            $(
            (StandardHeader::$konst, LegacyStandardHeader::$konst),
            )+
        ];

        #[test]
        fn test_standard_header_as_str_values() {
            for &(std, expected) in TEST_HEADER_STRINGS {
                assert_eq!(std.as_str(), expected);
            }
        }

        #[cfg(test)]
        #[test]
        fn test_standard_header_eq_matches_variant_identity() {
            for &(lhs, _) in TEST_HEADERS {
                for &(rhs, _) in TEST_HEADERS {
                    assert_eq!(
                        lhs == rhs,
                        std::mem::discriminant(&lhs) == std::mem::discriminant(&rhs),
                    );
                }
            }
        }

        #[test]
        fn test_parse_standard_headers() {
            for &(std, name_bytes) in TEST_HEADERS {
                // Test lower case
                assert_eq!(HeaderName::from_bytes(name_bytes).unwrap(), HeaderName::from(std));

                // Test upper case
                let upper = std::str::from_utf8(name_bytes).expect("byte string constants are all utf-8").to_uppercase();
                assert_eq!(HeaderName::from_bytes(upper.as_bytes()).unwrap(), HeaderName::from(std));
            }
        }

        #[test]
        fn test_standard_headers_into_bytes() {
            for &(std, name_bytes) in TEST_HEADERS {
                let name = std::str::from_utf8(name_bytes).unwrap();
                let std = HeaderName::from(std);
                // Test lower case
                let bytes: Bytes =
                    HeaderName::from_bytes(name_bytes).unwrap().inner.into();
                assert_eq!(bytes, name);
                assert_eq!(HeaderName::from_bytes(name_bytes).unwrap(), std);

                // Test upper case
                let upper = name.to_uppercase();
                let bytes: Bytes =
                    HeaderName::from_bytes(upper.as_bytes()).unwrap().inner.into();
                assert_eq!(bytes, name_bytes);
                assert_eq!(HeaderName::from_bytes(upper.as_bytes()).unwrap(),
                           std);
            }

        }
    }
}

#[cfg_attr(creusot, ensures(result == (lhs@ == rhs@)))]
const fn bytes_equal(lhs: &[u8], rhs: &[u8]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut i = 0;
    #[cfg_attr(creusot, invariant(lhs@.len() == rhs@.len()))]
    #[cfg_attr(creusot, invariant(i@ <= lhs@.len()))]
    #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < i@ ==> lhs@[j] == rhs@[j]))]
    #[cfg_attr(creusot, variant(lhs@.len() - i@))]
    while i < lhs.len() {
        if lhs[i] != rhs[i] {
            return false;
        }
        i += 1;
    }

    true
}

macro_rules! standard_header_byte_matcher {
    ($matcher:ident, [$($name_byte:literal),*]) => {
        #[cfg_attr(creusot, ensures(result == (name_bytes@ == seq![$($name_byte),*])))]
        const fn $matcher(name_bytes: &[u8]) -> bool {
            bytes_equal(name_bytes, &[$($name_byte),*])
        }
    };
}

// Generate constants for all standard HTTP headers. This includes a static hash
// code for the "fast hash" path. The hash code for static headers *do not* have
// to match the text representation of those headers. This is because header
// strings are always converted to the static values (when they match) before
// being hashed. This means that it is impossible to compare the static hash
// code of CONTENT_LENGTH with "content-length".
standard_headers! {
    /// Advertises which content types the client is able to understand.
    ///
    /// The Accept request HTTP header advertises which content types, expressed
    /// as MIME types, the client is able to understand. Using content
    /// negotiation, the server then selects one of the proposals, uses it and
    /// informs the client of its choice with the Content-Type response header.
    /// Browsers set adequate values for this header depending of the context
    /// where the request is done: when fetching a CSS stylesheet a different
    /// value is set for the request than when fetching an image, video or a
    /// script.
    (Accept, ACCEPT, 0, b"accept", "accept", [97u8, 99u8, 99u8, 101u8, 112u8, 116u8], bytes_equal_accept);

    /// Advertises which character set the client is able to understand.
    ///
    /// The Accept-Charset request HTTP header advertises which character set
    /// the client is able to understand. Using content negotiation, the server
    /// then selects one of the proposals, uses it and informs the client of its
    /// choice within the Content-Type response header. Browsers usually don't
    /// set this header as the default value for each content type is usually
    /// correct and transmitting it would allow easier fingerprinting.
    ///
    /// If the server cannot serve any matching character set, it can
    /// theoretically send back a 406 (Not Acceptable) error code. But, for a
    /// better user experience, this is rarely done and the more common way is
    /// to ignore the Accept-Charset header in this case.
    (AcceptCharset, ACCEPT_CHARSET, 1, b"accept-charset", "accept-charset", [97u8, 99u8, 99u8, 101u8, 112u8, 116u8, 45u8, 99u8, 104u8, 97u8, 114u8, 115u8, 101u8, 116u8], bytes_equal_accept_charset);

    /// Advertises which content encoding the client is able to understand.
    ///
    /// The Accept-Encoding request HTTP header advertises which content
    /// encoding, usually a compression algorithm, the client is able to
    /// understand. Using content negotiation, the server selects one of the
    /// proposals, uses it and informs the client of its choice with the
    /// Content-Encoding response header.
    ///
    /// Even if both the client and the server supports the same compression
    /// algorithms, the server may choose not to compress the body of a
    /// response, if the identity value is also acceptable. Two common cases
    /// lead to this:
    ///
    /// * The data to be sent is already compressed and a second compression
    /// won't lead to smaller data to be transmitted. This may the case with
    /// some image formats;
    ///
    /// * The server is overloaded and cannot afford the computational overhead
    /// induced by the compression requirement. Typically, Microsoft recommends
    /// not to compress if a server use more than 80 % of its computational
    /// power.
    ///
    /// As long as the identity value, meaning no compression, is not explicitly
    /// forbidden, by an identity;q=0 or a *;q=0 without another explicitly set
    /// value for identity, the server must never send back a 406 Not Acceptable
    /// error.
    (AcceptEncoding, ACCEPT_ENCODING, 2, b"accept-encoding", "accept-encoding", [97u8, 99u8, 99u8, 101u8, 112u8, 116u8, 45u8, 101u8, 110u8, 99u8, 111u8, 100u8, 105u8, 110u8, 103u8], bytes_equal_accept_encoding);

    /// Advertises which languages the client is able to understand.
    ///
    /// The Accept-Language request HTTP header advertises which languages the
    /// client is able to understand, and which locale variant is preferred.
    /// Using content negotiation, the server then selects one of the proposals,
    /// uses it and informs the client of its choice with the Content-Language
    /// response header. Browsers set adequate values for this header according
    /// their user interface language and even if a user can change it, this
    /// happens rarely (and is frown upon as it leads to fingerprinting).
    ///
    /// This header is a hint to be used when the server has no way of
    /// determining the language via another way, like a specific URL, that is
    /// controlled by an explicit user decision. It is recommended that the
    /// server never overrides an explicit decision. The content of the
    /// Accept-Language is often out of the control of the user (like when
    /// traveling and using an Internet Cafe in a different country); the user
    /// may also want to visit a page in another language than the locale of
    /// their user interface.
    ///
    /// If the server cannot serve any matching language, it can theoretically
    /// send back a 406 (Not Acceptable) error code. But, for a better user
    /// experience, this is rarely done and more common way is to ignore the
    /// Accept-Language header in this case.
    (AcceptLanguage, ACCEPT_LANGUAGE, 3, b"accept-language", "accept-language", [97u8, 99u8, 99u8, 101u8, 112u8, 116u8, 45u8, 108u8, 97u8, 110u8, 103u8, 117u8, 97u8, 103u8, 101u8], bytes_equal_accept_language);

    /// Marker used by the server to advertise partial request support.
    ///
    /// The Accept-Ranges response HTTP header is a marker used by the server to
    /// advertise its support of partial requests. The value of this field
    /// indicates the unit that can be used to define a range.
    ///
    /// In presence of an Accept-Ranges header, the browser may try to resume an
    /// interrupted download, rather than to start it from the start again.
    (AcceptRanges, ACCEPT_RANGES, 4, b"accept-ranges", "accept-ranges", [97u8, 99u8, 99u8, 101u8, 112u8, 116u8, 45u8, 114u8, 97u8, 110u8, 103u8, 101u8, 115u8], bytes_equal_accept_ranges);

    /// Preflight response indicating if the response to the request can be
    /// exposed to the page.
    ///
    /// The Access-Control-Allow-Credentials response header indicates whether
    /// or not the response to the request can be exposed to the page. It can be
    /// exposed when the true value is returned; it can't in other cases.
    ///
    /// Credentials are cookies, authorization headers or TLS client
    /// certificates.
    ///
    /// When used as part of a response to a preflight request, this indicates
    /// whether or not the actual request can be made using credentials. Note
    /// that simple GET requests are not preflighted, and so if a request is
    /// made for a resource with credentials, if this header is not returned
    /// with the resource, the response is ignored by the browser and not
    /// returned to web content.
    ///
    /// The Access-Control-Allow-Credentials header works in conjunction with
    /// the XMLHttpRequest.withCredentials property or with the credentials
    /// option in the Request() constructor of the Fetch API. Credentials must
    /// be set on both sides (the Access-Control-Allow-Credentials header and in
    /// the XHR or Fetch request) in order for the CORS request with credentials
    /// to succeed.
    (AccessControlAllowCredentials, ACCESS_CONTROL_ALLOW_CREDENTIALS, 5, b"access-control-allow-credentials", "access-control-allow-credentials", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 97u8, 108u8, 108u8, 111u8, 119u8, 45u8, 99u8, 114u8, 101u8, 100u8, 101u8, 110u8, 116u8, 105u8, 97u8, 108u8, 115u8], bytes_equal_access_control_allow_credentials);

    /// Preflight response indicating permitted HTTP headers.
    ///
    /// The Access-Control-Allow-Headers response header is used in response to
    /// a preflight request to indicate which HTTP headers will be available via
    /// Access-Control-Expose-Headers when making the actual request.
    ///
    /// The simple headers, Accept, Accept-Language, Content-Language,
    /// Content-Type (but only with a MIME type of its parsed value (ignoring
    /// parameters) of either application/x-www-form-urlencoded,
    /// multipart/form-data, or text/plain), are always available and don't need
    /// to be listed by this header.
    ///
    /// This header is required if the request has an
    /// Access-Control-Request-Headers header.
    (AccessControlAllowHeaders, ACCESS_CONTROL_ALLOW_HEADERS, 6, b"access-control-allow-headers", "access-control-allow-headers", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 97u8, 108u8, 108u8, 111u8, 119u8, 45u8, 104u8, 101u8, 97u8, 100u8, 101u8, 114u8, 115u8], bytes_equal_access_control_allow_headers);

    /// Preflight header response indicating permitted access methods.
    ///
    /// The Access-Control-Allow-Methods response header specifies the method or
    /// methods allowed when accessing the resource in response to a preflight
    /// request.
    (AccessControlAllowMethods, ACCESS_CONTROL_ALLOW_METHODS, 7, b"access-control-allow-methods", "access-control-allow-methods", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 97u8, 108u8, 108u8, 111u8, 119u8, 45u8, 109u8, 101u8, 116u8, 104u8, 111u8, 100u8, 115u8], bytes_equal_access_control_allow_methods);

    /// Indicates whether the response can be shared with resources with the
    /// given origin.
    (AccessControlAllowOrigin, ACCESS_CONTROL_ALLOW_ORIGIN, 8, b"access-control-allow-origin", "access-control-allow-origin", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 97u8, 108u8, 108u8, 111u8, 119u8, 45u8, 111u8, 114u8, 105u8, 103u8, 105u8, 110u8], bytes_equal_access_control_allow_origin);

    /// Indicates which headers can be exposed as part of the response by
    /// listing their names.
    (AccessControlExposeHeaders, ACCESS_CONTROL_EXPOSE_HEADERS, 9, b"access-control-expose-headers", "access-control-expose-headers", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 101u8, 120u8, 112u8, 111u8, 115u8, 101u8, 45u8, 104u8, 101u8, 97u8, 100u8, 101u8, 114u8, 115u8], bytes_equal_access_control_expose_headers);

    /// Indicates how long the results of a preflight request can be cached.
    (AccessControlMaxAge, ACCESS_CONTROL_MAX_AGE, 10, b"access-control-max-age", "access-control-max-age", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 109u8, 97u8, 120u8, 45u8, 97u8, 103u8, 101u8], bytes_equal_access_control_max_age);

    /// Informs the server which HTTP headers will be used when an actual
    /// request is made.
    (AccessControlRequestHeaders, ACCESS_CONTROL_REQUEST_HEADERS, 11, b"access-control-request-headers", "access-control-request-headers", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 114u8, 101u8, 113u8, 117u8, 101u8, 115u8, 116u8, 45u8, 104u8, 101u8, 97u8, 100u8, 101u8, 114u8, 115u8], bytes_equal_access_control_request_headers);

    /// Informs the server know which HTTP method will be used when the actual
    /// request is made.
    (AccessControlRequestMethod, ACCESS_CONTROL_REQUEST_METHOD, 12, b"access-control-request-method", "access-control-request-method", [97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8, 45u8, 114u8, 101u8, 113u8, 117u8, 101u8, 115u8, 116u8, 45u8, 109u8, 101u8, 116u8, 104u8, 111u8, 100u8], bytes_equal_access_control_request_method);

    /// Indicates the time in seconds the object has been in a proxy cache.
    ///
    /// The Age header is usually close to zero. If it is Age: 0, it was
    /// probably just fetched from the origin server; otherwise It is usually
    /// calculated as a difference between the proxy's current date and the Date
    /// general header included in the HTTP response.
    (Age, AGE, 13, b"age", "age", [97u8, 103u8, 101u8], bytes_equal_age);

    /// Lists the set of methods support by a resource.
    ///
    /// This header must be sent if the server responds with a 405 Method Not
    /// Allowed status code to indicate which request methods can be used. An
    /// empty Allow header indicates that the resource allows no request
    /// methods, which might occur temporarily for a given resource, for
    /// example.
    (Allow, ALLOW, 14, b"allow", "allow", [97u8, 108u8, 108u8, 111u8, 119u8], bytes_equal_allow);

    /// Advertises the availability of alternate services to clients.
    (AltSvc, ALT_SVC, 15, b"alt-svc", "alt-svc", [97u8, 108u8, 116u8, 45u8, 115u8, 118u8, 99u8], bytes_equal_alt_svc);

    /// Contains the credentials to authenticate a user agent with a server.
    ///
    /// Usually this header is included after the server has responded with a
    /// 401 Unauthorized status and the WWW-Authenticate header.
    (Authorization, AUTHORIZATION, 16, b"authorization", "authorization", [97u8, 117u8, 116u8, 104u8, 111u8, 114u8, 105u8, 122u8, 97u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_authorization);

    /// Specifies directives for caching mechanisms in both requests and
    /// responses.
    ///
    /// Caching directives are unidirectional, meaning that a given directive in
    /// a request is not implying that the same directive is to be given in the
    /// response.
    (CacheControl, CACHE_CONTROL, 17, b"cache-control", "cache-control", [99u8, 97u8, 99u8, 104u8, 101u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8], bytes_equal_cache_control);

    /// Indicates how caches have handled a response and its corresponding request.
    ///
    /// See [RFC 9211](https://www.rfc-editor.org/rfc/rfc9211.html).
    (CacheStatus, CACHE_STATUS, 18, b"cache-status", "cache-status", [99u8, 97u8, 99u8, 104u8, 101u8, 45u8, 115u8, 116u8, 97u8, 116u8, 117u8, 115u8], bytes_equal_cache_status);

    /// Specifies directives that allow origin servers to control the behavior of CDN caches
    /// interposed between them and clients separately from other caches that might handle the
    /// response.
    ///
    /// See [RFC 9213](https://www.rfc-editor.org/rfc/rfc9213.html).
    (CdnCacheControl, CDN_CACHE_CONTROL, 19, b"cdn-cache-control", "cdn-cache-control", [99u8, 100u8, 110u8, 45u8, 99u8, 97u8, 99u8, 104u8, 101u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8], bytes_equal_cdn_cache_control);

    /// Controls whether or not the network connection stays open after the
    /// current transaction finishes.
    ///
    /// If the value sent is keep-alive, the connection is persistent and not
    /// closed, allowing for subsequent requests to the same server to be done.
    ///
    /// Except for the standard hop-by-hop headers (Keep-Alive,
    /// Transfer-Encoding, TE, Connection, Trailer, Upgrade, Proxy-Authorization
    /// and Proxy-Authenticate), any hop-by-hop headers used by the message must
    /// be listed in the Connection header, so that the first proxy knows he has
    /// to consume them and not to forward them further. Standard hop-by-hop
    /// headers can be listed too (it is often the case of Keep-Alive, but this
    /// is not mandatory.
    (Connection, CONNECTION, 20, b"connection", "connection", [99u8, 111u8, 110u8, 110u8, 101u8, 99u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_connection);

    /// Indicates if the content is expected to be displayed inline.
    ///
    /// In a regular HTTP response, the Content-Disposition response header is a
    /// header indicating if the content is expected to be displayed inline in
    /// the browser, that is, as a Web page or as part of a Web page, or as an
    /// attachment, that is downloaded and saved locally.
    ///
    /// In a multipart/form-data body, the HTTP Content-Disposition general
    /// header is a header that can be used on the subpart of a multipart body
    /// to give information about the field it applies to. The subpart is
    /// delimited by the boundary defined in the Content-Type header. Used on
    /// the body itself, Content-Disposition has no effect.
    ///
    /// The Content-Disposition header is defined in the larger context of MIME
    /// messages for e-mail, but only a subset of the possible parameters apply
    /// to HTTP forms and POST requests. Only the value form-data, as well as
    /// the optional directive name and filename, can be used in the HTTP
    /// context.
    (ContentDisposition, CONTENT_DISPOSITION, 21, b"content-disposition", "content-disposition", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 100u8, 105u8, 115u8, 112u8, 111u8, 115u8, 105u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_content_disposition);

    /// Used to compress the media-type.
    ///
    /// When present, its value indicates what additional content encoding has
    /// been applied to the entity-body. It lets the client know, how to decode
    /// in order to obtain the media-type referenced by the Content-Type header.
    ///
    /// It is recommended to compress data as much as possible and therefore to
    /// use this field, but some types of resources, like jpeg images, are
    /// already compressed.  Sometimes using additional compression doesn't
    /// reduce payload size and can even make the payload longer.
    (ContentEncoding, CONTENT_ENCODING, 22, b"content-encoding", "content-encoding", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 101u8, 110u8, 99u8, 111u8, 100u8, 105u8, 110u8, 103u8], bytes_equal_content_encoding);

    /// Used to describe the languages intended for the audience.
    ///
    /// This header allows a user to differentiate according to the users' own
    /// preferred language. For example, if "Content-Language: de-DE" is set, it
    /// says that the document is intended for German language speakers
    /// (however, it doesn't indicate the document is written in German. For
    /// example, it might be written in English as part of a language course for
    /// German speakers).
    ///
    /// If no Content-Language is specified, the default is that the content is
    /// intended for all language audiences. Multiple language tags are also
    /// possible, as well as applying the Content-Language header to various
    /// media types and not only to textual documents.
    (ContentLanguage, CONTENT_LANGUAGE, 23, b"content-language", "content-language", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 108u8, 97u8, 110u8, 103u8, 117u8, 97u8, 103u8, 101u8], bytes_equal_content_language);

    /// Indicates the size of the entity-body.
    ///
    /// The header value must be a decimal indicating the number of octets sent
    /// to the recipient.
    (ContentLength, CONTENT_LENGTH, 24, b"content-length", "content-length", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 108u8, 101u8, 110u8, 103u8, 116u8, 104u8], bytes_equal_content_length);

    /// Indicates an alternate location for the returned data.
    ///
    /// The principal use case is to indicate the URL of the resource
    /// transmitted as the result of content negotiation.
    ///
    /// Location and Content-Location are different: Location indicates the
    /// target of a redirection (or the URL of a newly created document), while
    /// Content-Location indicates the direct URL to use to access the resource,
    /// without the need of further content negotiation. Location is a header
    /// associated with the response, while Content-Location is associated with
    /// the entity returned.
    (ContentLocation, CONTENT_LOCATION, 25, b"content-location", "content-location", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 108u8, 111u8, 99u8, 97u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_content_location);

    /// Indicates where in a full body message a partial message belongs.
    (ContentRange, CONTENT_RANGE, 26, b"content-range", "content-range", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 114u8, 97u8, 110u8, 103u8, 101u8], bytes_equal_content_range);

    /// Allows controlling resources the user agent is allowed to load for a
    /// given page.
    ///
    /// With a few exceptions, policies mostly involve specifying server origins
    /// and script endpoints. This helps guard against cross-site scripting
    /// attacks (XSS).
    (ContentSecurityPolicy, CONTENT_SECURITY_POLICY, 27, b"content-security-policy", "content-security-policy", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 115u8, 101u8, 99u8, 117u8, 114u8, 105u8, 116u8, 121u8, 45u8, 112u8, 111u8, 108u8, 105u8, 99u8, 121u8], bytes_equal_content_security_policy);

    /// Allows experimenting with policies by monitoring their effects.
    ///
    /// The HTTP Content-Security-Policy-Report-Only response header allows web
    /// developers to experiment with policies by monitoring (but not enforcing)
    /// their effects. These violation reports consist of JSON documents sent
    /// via an HTTP POST request to the specified URI.
    (ContentSecurityPolicyReportOnly, CONTENT_SECURITY_POLICY_REPORT_ONLY, 28, b"content-security-policy-report-only", "content-security-policy-report-only", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 115u8, 101u8, 99u8, 117u8, 114u8, 105u8, 116u8, 121u8, 45u8, 112u8, 111u8, 108u8, 105u8, 99u8, 121u8, 45u8, 114u8, 101u8, 112u8, 111u8, 114u8, 116u8, 45u8, 111u8, 110u8, 108u8, 121u8], bytes_equal_content_security_policy_report_only);

    /// Used to indicate the media type of the resource.
    ///
    /// In responses, a Content-Type header tells the client what the content
    /// type of the returned content actually is. Browsers will do MIME sniffing
    /// in some cases and will not necessarily follow the value of this header;
    /// to prevent this behavior, the header X-Content-Type-Options can be set
    /// to nosniff.
    ///
    /// In requests, (such as POST or PUT), the client tells the server what
    /// type of data is actually sent.
    (ContentType, CONTENT_TYPE, 29, b"content-type", "content-type", [99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 116u8, 121u8, 112u8, 101u8], bytes_equal_content_type);

    /// Contains stored HTTP cookies previously sent by the server with the
    /// Set-Cookie header.
    ///
    /// The Cookie header might be omitted entirely, if the privacy setting of
    /// the browser are set to block them, for example.
    (Cookie, COOKIE, 30, b"cookie", "cookie", [99u8, 111u8, 111u8, 107u8, 105u8, 101u8], bytes_equal_cookie);

    /// Indicates the client's tracking preference.
    ///
    /// This header lets users indicate whether they would prefer privacy rather
    /// than personalized content.
    (Dnt, DNT, 31, b"dnt", "dnt", [100u8, 110u8, 116u8], bytes_equal_dnt);

    /// Contains the date and time at which the message was originated.
    (Date, DATE, 32, b"date", "date", [100u8, 97u8, 116u8, 101u8], bytes_equal_date);

    /// Identifier for a specific version of a resource.
    ///
    /// This header allows caches to be more efficient, and saves bandwidth, as
    /// a web server does not need to send a full response if the content has
    /// not changed. On the other side, if the content has changed, etags are
    /// useful to help prevent simultaneous updates of a resource from
    /// overwriting each other ("mid-air collisions").
    ///
    /// If the resource at a given URL changes, a new Etag value must be
    /// generated. Etags are therefore similar to fingerprints and might also be
    /// used for tracking purposes by some servers. A comparison of them allows
    /// to quickly determine whether two representations of a resource are the
    /// same, but they might also be set to persist indefinitely by a tracking
    /// server.
    (Etag, ETAG, 33, b"etag", "etag", [101u8, 116u8, 97u8, 103u8], bytes_equal_etag);

    /// Indicates expectations that need to be fulfilled by the server in order
    /// to properly handle the request.
    ///
    /// The only expectation defined in the specification is Expect:
    /// 100-continue, to which the server shall respond with:
    ///
    /// * 100 if the information contained in the header is sufficient to cause
    /// an immediate success,
    ///
    /// * 417 (Expectation Failed) if it cannot meet the expectation; or any
    /// other 4xx status otherwise.
    ///
    /// For example, the server may reject a request if its Content-Length is
    /// too large.
    ///
    /// No common browsers send the Expect header, but some other clients such
    /// as cURL do so by default.
    (Expect, EXPECT, 34, b"expect", "expect", [101u8, 120u8, 112u8, 101u8, 99u8, 116u8], bytes_equal_expect);

    /// Contains the date/time after which the response is considered stale.
    ///
    /// Invalid dates, like the value 0, represent a date in the past and mean
    /// that the resource is already expired.
    ///
    /// If there is a Cache-Control header with the "max-age" or "s-max-age"
    /// directive in the response, the Expires header is ignored.
    (Expires, EXPIRES, 35, b"expires", "expires", [101u8, 120u8, 112u8, 105u8, 114u8, 101u8, 115u8], bytes_equal_expires);

    /// Contains information from the client-facing side of proxy servers that
    /// is altered or lost when a proxy is involved in the path of the request.
    ///
    /// The alternative and de-facto standard versions of this header are the
    /// X-Forwarded-For, X-Forwarded-Host and X-Forwarded-Proto headers.
    ///
    /// This header is used for debugging, statistics, and generating
    /// location-dependent content and by design it exposes privacy sensitive
    /// information, such as the IP address of the client. Therefore the user's
    /// privacy must be kept in mind when deploying this header.
    (Forwarded, FORWARDED, 36, b"forwarded", "forwarded", [102u8, 111u8, 114u8, 119u8, 97u8, 114u8, 100u8, 101u8, 100u8], bytes_equal_forwarded);

    /// Contains an Internet email address for a human user who controls the
    /// requesting user agent.
    ///
    /// If you are running a robotic user agent (e.g. a crawler), the From
    /// header should be sent, so you can be contacted if problems occur on
    /// servers, such as if the robot is sending excessive, unwanted, or invalid
    /// requests.
    (From, FROM, 37, b"from", "from", [102u8, 114u8, 111u8, 109u8], bytes_equal_from);

    /// Specifies the domain name of the server and (optionally) the TCP port
    /// number on which the server is listening.
    ///
    /// If no port is given, the default port for the service requested (e.g.,
    /// "80" for an HTTP URL) is implied.
    ///
    /// A Host header field must be sent in all HTTP/1.1 request messages. A 400
    /// (Bad Request) status code will be sent to any HTTP/1.1 request message
    /// that lacks a Host header field or contains more than one.
    (Host, HOST, 38, b"host", "host", [104u8, 111u8, 115u8, 116u8], bytes_equal_host);

    /// Makes a request conditional based on the E-Tag.
    ///
    /// For GET and HEAD methods, the server will send back the requested
    /// resource only if it matches one of the listed ETags. For PUT and other
    /// non-safe methods, it will only upload the resource in this case.
    ///
    /// The comparison with the stored ETag uses the strong comparison
    /// algorithm, meaning two files are considered identical byte to byte only.
    /// This is weakened when the  W/ prefix is used in front of the ETag.
    ///
    /// There are two common use cases:
    ///
    /// * For GET and HEAD methods, used in combination with an Range header, it
    /// can guarantee that the new ranges requested comes from the same resource
    /// than the previous one. If it doesn't match, then a 416 (Range Not
    /// Satisfiable) response is returned.
    ///
    /// * For other methods, and in particular for PUT, If-Match can be used to
    /// prevent the lost update problem. It can check if the modification of a
    /// resource that the user wants to upload will not override another change
    /// that has been done since the original resource was fetched. If the
    /// request cannot be fulfilled, the 412 (Precondition Failed) response is
    /// returned.
    (IfMatch, IF_MATCH, 39, b"if-match", "if-match", [105u8, 102u8, 45u8, 109u8, 97u8, 116u8, 99u8, 104u8], bytes_equal_if_match);

    /// Makes a request conditional based on the modification date.
    ///
    /// The If-Modified-Since request HTTP header makes the request conditional:
    /// the server will send back the requested resource, with a 200 status,
    /// only if it has been last modified after the given date. If the request
    /// has not been modified since, the response will be a 304 without any
    /// body; the Last-Modified header will contain the date of last
    /// modification. Unlike If-Unmodified-Since, If-Modified-Since can only be
    /// used with a GET or HEAD.
    ///
    /// When used in combination with If-None-Match, it is ignored, unless the
    /// server doesn't support If-None-Match.
    ///
    /// The most common use case is to update a cached entity that has no
    /// associated ETag.
    (IfModifiedSince, IF_MODIFIED_SINCE, 40, b"if-modified-since", "if-modified-since", [105u8, 102u8, 45u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8, 45u8, 115u8, 105u8, 110u8, 99u8, 101u8], bytes_equal_if_modified_since);

    /// Makes a request conditional based on the E-Tag.
    ///
    /// The If-None-Match HTTP request header makes the request conditional. For
    /// GET and HEAD methods, the server will send back the requested resource,
    /// with a 200 status, only if it doesn't have an ETag matching the given
    /// ones. For other methods, the request will be processed only if the
    /// eventually existing resource's ETag doesn't match any of the values
    /// listed.
    ///
    /// When the condition fails for GET and HEAD methods, then the server must
    /// return HTTP status code 304 (Not Modified). For methods that apply
    /// server-side changes, the status code 412 (Precondition Failed) is used.
    /// Note that the server generating a 304 response MUST generate any of the
    /// following header fields that would have been sent in a 200 (OK) response
    /// to the same request: Cache-Control, Content-Location, Date, ETag,
    /// Expires, and Vary.
    ///
    /// The comparison with the stored ETag uses the weak comparison algorithm,
    /// meaning two files are considered identical not only if they are
    /// identical byte to byte, but if the content is equivalent. For example,
    /// two pages that would differ only by the date of generation in the footer
    /// would be considered as identical.
    ///
    /// When used in combination with If-Modified-Since, it has precedence (if
    /// the server supports it).
    ///
    /// There are two common use cases:
    ///
    /// * For `GET` and `HEAD` methods, to update a cached entity that has an associated ETag.
    /// * For other methods, and in particular for `PUT`, `If-None-Match` used with
    /// the `*` value can be used to save a file not known to exist,
    /// guaranteeing that another upload didn't happen before, losing the data
    /// of the previous put; this problems is the variation of the lost update
    /// problem.
    (IfNoneMatch, IF_NONE_MATCH, 41, b"if-none-match", "if-none-match", [105u8, 102u8, 45u8, 110u8, 111u8, 110u8, 101u8, 45u8, 109u8, 97u8, 116u8, 99u8, 104u8], bytes_equal_if_none_match);

    /// Makes a request conditional based on range.
    ///
    /// The If-Range HTTP request header makes a range request conditional: if
    /// the condition is fulfilled, the range request will be issued and the
    /// server sends back a 206 Partial Content answer with the appropriate
    /// body. If the condition is not fulfilled, the full resource is sent back,
    /// with a 200 OK status.
    ///
    /// This header can be used either with a Last-Modified validator, or with
    /// an ETag, but not with both.
    ///
    /// The most common use case is to resume a download, to guarantee that the
    /// stored resource has not been modified since the last fragment has been
    /// received.
    (IfRange, IF_RANGE, 42, b"if-range", "if-range", [105u8, 102u8, 45u8, 114u8, 97u8, 110u8, 103u8, 101u8], bytes_equal_if_range);

    /// Makes the request conditional based on the last modification date.
    ///
    /// The If-Unmodified-Since request HTTP header makes the request
    /// conditional: the server will send back the requested resource, or accept
    /// it in the case of a POST or another non-safe method, only if it has not
    /// been last modified after the given date. If the request has been
    /// modified after the given date, the response will be a 412 (Precondition
    /// Failed) error.
    ///
    /// There are two common use cases:
    ///
    /// * In conjunction non-safe methods, like POST, it can be used to
    /// implement an optimistic concurrency control, like done by some wikis:
    /// editions are rejected if the stored document has been modified since the
    /// original has been retrieved.
    ///
    /// * In conjunction with a range request with a If-Range header, it can be
    /// used to ensure that the new fragment requested comes from an unmodified
    /// document.
    (IfUnmodifiedSince, IF_UNMODIFIED_SINCE, 43, b"if-unmodified-since", "if-unmodified-since", [105u8, 102u8, 45u8, 117u8, 110u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8, 45u8, 115u8, 105u8, 110u8, 99u8, 101u8], bytes_equal_if_unmodified_since);

    /// The Last-Modified header contains the date and time when the origin believes
    /// the resource was last modified.
    ///
    /// The value is a valid Date/Time string defined in [RFC9910](https://datatracker.ietf.org/doc/html/rfc9110#section-5.6.7)
    (LastModified, LAST_MODIFIED, 44, b"last-modified", "last-modified", [108u8, 97u8, 115u8, 116u8, 45u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8], bytes_equal_last_modified);

    /// Allows the server to point an interested client to another resource
    /// containing metadata about the requested resource.
    (Link, LINK, 45, b"link", "link", [108u8, 105u8, 110u8, 107u8], bytes_equal_link);

    /// Indicates the URL to redirect a page to.
    ///
    /// The Location response header indicates the URL to redirect a page to. It
    /// only provides a meaning when served with a 3xx status response.
    ///
    /// The HTTP method used to make the new request to fetch the page pointed
    /// to by Location depends of the original method and of the kind of
    /// redirection:
    ///
    /// * If 303 (See Also) responses always lead to the use of a GET method,
    /// 307 (Temporary Redirect) and 308 (Permanent Redirect) don't change the
    /// method used in the original request;
    ///
    /// * 301 (Permanent Redirect) and 302 (Found) doesn't change the method
    /// most of the time, though older user-agents may (so you basically don't
    /// know).
    ///
    /// All responses with one of these status codes send a Location header.
    ///
    /// Beside redirect response, messages with 201 (Created) status also
    /// include the Location header. It indicates the URL to the newly created
    /// resource.
    ///
    /// Location and Content-Location are different: Location indicates the
    /// target of a redirection (or the URL of a newly created resource), while
    /// Content-Location indicates the direct URL to use to access the resource
    /// when content negotiation happened, without the need of further content
    /// negotiation. Location is a header associated with the response, while
    /// Content-Location is associated with the entity returned.
    (Location, LOCATION, 46, b"location", "location", [108u8, 111u8, 99u8, 97u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_location);

    /// Indicates the max number of intermediaries the request should be sent
    /// through.
    (MaxForwards, MAX_FORWARDS, 47, b"max-forwards", "max-forwards", [109u8, 97u8, 120u8, 45u8, 102u8, 111u8, 114u8, 119u8, 97u8, 114u8, 100u8, 115u8], bytes_equal_max_forwards);

    /// Indicates where a fetch originates from.
    ///
    /// It doesn't include any path information, but only the server name. It is
    /// sent with CORS requests, as well as with POST requests. It is similar to
    /// the Referer header, but, unlike this header, it doesn't disclose the
    /// whole path.
    (Origin, ORIGIN, 48, b"origin", "origin", [111u8, 114u8, 105u8, 103u8, 105u8, 110u8], bytes_equal_origin);

    /// HTTP/1.0 header usually used for backwards compatibility.
    ///
    /// The Pragma HTTP/1.0 general header is an implementation-specific header
    /// that may have various effects along the request-response chain. It is
    /// used for backwards compatibility with HTTP/1.0 caches where the
    /// Cache-Control HTTP/1.1 header is not yet present.
    (Pragma, PRAGMA, 49, b"pragma", "pragma", [112u8, 114u8, 97u8, 103u8, 109u8, 97u8], bytes_equal_pragma);

    /// Defines the authentication method that should be used to gain access to
    /// a proxy.
    ///
    /// Unlike `www-authenticate`, the `proxy-authenticate` header field applies
    /// only to the next outbound client on the response chain. This is because
    /// only the client that chose a given proxy is likely to have the
    /// credentials necessary for authentication. However, when multiple proxies
    /// are used within the same administrative domain, such as office and
    /// regional caching proxies within a large corporate network, it is common
    /// for credentials to be generated by the user agent and passed through the
    /// hierarchy until consumed. Hence, in such a configuration, it will appear
    /// as if Proxy-Authenticate is being forwarded because each proxy will send
    /// the same challenge set.
    ///
    /// The `proxy-authenticate` header is sent along with a `407 Proxy
    /// Authentication Required`.
    (ProxyAuthenticate, PROXY_AUTHENTICATE, 50, b"proxy-authenticate", "proxy-authenticate", [112u8, 114u8, 111u8, 120u8, 121u8, 45u8, 97u8, 117u8, 116u8, 104u8, 101u8, 110u8, 116u8, 105u8, 99u8, 97u8, 116u8, 101u8], bytes_equal_proxy_authenticate);

    /// Contains the credentials to authenticate a user agent to a proxy server.
    ///
    /// This header is usually included after the server has responded with a
    /// 407 Proxy Authentication Required status and the Proxy-Authenticate
    /// header.
    (ProxyAuthorization, PROXY_AUTHORIZATION, 51, b"proxy-authorization", "proxy-authorization", [112u8, 114u8, 111u8, 120u8, 121u8, 45u8, 97u8, 117u8, 116u8, 104u8, 111u8, 114u8, 105u8, 122u8, 97u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_proxy_authorization);

    /// Associates a specific cryptographic public key with a certain server.
    ///
    /// This decreases the risk of MITM attacks with forged certificates. If one
    /// or several keys are pinned and none of them are used by the server, the
    /// browser will not accept the response as legitimate, and will not display
    /// it.
    (PublicKeyPins, PUBLIC_KEY_PINS, 52, b"public-key-pins", "public-key-pins", [112u8, 117u8, 98u8, 108u8, 105u8, 99u8, 45u8, 107u8, 101u8, 121u8, 45u8, 112u8, 105u8, 110u8, 115u8], bytes_equal_public_key_pins);

    /// Sends reports of pinning violation to the report-uri specified in the
    /// header.
    ///
    /// Unlike `Public-Key-Pins`, this header still allows browsers to connect
    /// to the server if the pinning is violated.
    (PublicKeyPinsReportOnly, PUBLIC_KEY_PINS_REPORT_ONLY, 53, b"public-key-pins-report-only", "public-key-pins-report-only", [112u8, 117u8, 98u8, 108u8, 105u8, 99u8, 45u8, 107u8, 101u8, 121u8, 45u8, 112u8, 105u8, 110u8, 115u8, 45u8, 114u8, 101u8, 112u8, 111u8, 114u8, 116u8, 45u8, 111u8, 110u8, 108u8, 121u8], bytes_equal_public_key_pins_report_only);

    /// Indicates the part of a document that the server should return.
    ///
    /// Several parts can be requested with one Range header at once, and the
    /// server may send back these ranges in a multipart document. If the server
    /// sends back ranges, it uses the 206 Partial Content for the response. If
    /// the ranges are invalid, the server returns the 416 Range Not Satisfiable
    /// error. The server can also ignore the Range header and return the whole
    /// document with a 200 status code.
    (Range, RANGE, 54, b"range", "range", [114u8, 97u8, 110u8, 103u8, 101u8], bytes_equal_range);

    /// Contains the address of the previous web page from which a link to the
    /// currently requested page was followed.
    ///
    /// The Referer header allows servers to identify where people are visiting
    /// them from and may use that data for analytics, logging, or optimized
    /// caching, for example.
    (Referer, REFERER, 55, b"referer", "referer", [114u8, 101u8, 102u8, 101u8, 114u8, 101u8, 114u8], bytes_equal_referer);

    /// Governs which referrer information should be included with requests
    /// made.
    (ReferrerPolicy, REFERRER_POLICY, 56, b"referrer-policy", "referrer-policy", [114u8, 101u8, 102u8, 101u8, 114u8, 114u8, 101u8, 114u8, 45u8, 112u8, 111u8, 108u8, 105u8, 99u8, 121u8], bytes_equal_referrer_policy);

    /// Informs the web browser that the current page or frame should be
    /// refreshed.
    (Refresh, REFRESH, 57, b"refresh", "refresh", [114u8, 101u8, 102u8, 114u8, 101u8, 115u8, 104u8], bytes_equal_refresh);

    /// The Retry-After response HTTP header indicates how long the user agent
    /// should wait before making a follow-up request. There are two main cases
    /// this header is used:
    ///
    /// * When sent with a 503 (Service Unavailable) response, it indicates how
    /// long the service is expected to be unavailable.
    ///
    /// * When sent with a redirect response, such as 301 (Moved Permanently),
    /// it indicates the minimum time that the user agent is asked to wait
    /// before issuing the redirected request.
    (RetryAfter, RETRY_AFTER, 58, b"retry-after", "retry-after", [114u8, 101u8, 116u8, 114u8, 121u8, 45u8, 97u8, 102u8, 116u8, 101u8, 114u8], bytes_equal_retry_after);

    /// The |Sec-WebSocket-Accept| header field is used in the WebSocket
    /// opening handshake. It is sent from the server to the client to
    /// confirm that the server is willing to initiate the WebSocket
    /// connection.
    (SecWebSocketAccept, SEC_WEBSOCKET_ACCEPT, 59, b"sec-websocket-accept", "sec-websocket-accept", [115u8, 101u8, 99u8, 45u8, 119u8, 101u8, 98u8, 115u8, 111u8, 99u8, 107u8, 101u8, 116u8, 45u8, 97u8, 99u8, 99u8, 101u8, 112u8, 116u8], bytes_equal_sec_web_socket_accept);

    /// The |Sec-WebSocket-Extensions| header field is used in the WebSocket
    /// opening handshake. It is initially sent from the client to the
    /// server, and then subsequently sent from the server to the client, to
    /// agree on a set of protocol-level extensions to use for the duration
    /// of the connection.
    (SecWebSocketExtensions, SEC_WEBSOCKET_EXTENSIONS, 60, b"sec-websocket-extensions", "sec-websocket-extensions", [115u8, 101u8, 99u8, 45u8, 119u8, 101u8, 98u8, 115u8, 111u8, 99u8, 107u8, 101u8, 116u8, 45u8, 101u8, 120u8, 116u8, 101u8, 110u8, 115u8, 105u8, 111u8, 110u8, 115u8], bytes_equal_sec_web_socket_extensions);

    /// The |Sec-WebSocket-Key| header field is used in the WebSocket opening
    /// handshake. It is sent from the client to the server to provide part
    /// of the information used by the server to prove that it received a
    /// valid WebSocket opening handshake. This helps ensure that the server
    /// does not accept connections from non-WebSocket clients (e.g., HTTP
    /// clients) that are being abused to send data to unsuspecting WebSocket
    /// servers.
    (SecWebSocketKey, SEC_WEBSOCKET_KEY, 61, b"sec-websocket-key", "sec-websocket-key", [115u8, 101u8, 99u8, 45u8, 119u8, 101u8, 98u8, 115u8, 111u8, 99u8, 107u8, 101u8, 116u8, 45u8, 107u8, 101u8, 121u8], bytes_equal_sec_web_socket_key);

    /// The |Sec-WebSocket-Protocol| header field is used in the WebSocket
    /// opening handshake. It is sent from the client to the server and back
    /// from the server to the client to confirm the subprotocol of the
    /// connection.  This enables scripts to both select a subprotocol and be
    /// sure that the server agreed to serve that subprotocol.
    (SecWebSocketProtocol, SEC_WEBSOCKET_PROTOCOL, 62, b"sec-websocket-protocol", "sec-websocket-protocol", [115u8, 101u8, 99u8, 45u8, 119u8, 101u8, 98u8, 115u8, 111u8, 99u8, 107u8, 101u8, 116u8, 45u8, 112u8, 114u8, 111u8, 116u8, 111u8, 99u8, 111u8, 108u8], bytes_equal_sec_web_socket_protocol);

    /// The |Sec-WebSocket-Version| header field is used in the WebSocket
    /// opening handshake.  It is sent from the client to the server to
    /// indicate the protocol version of the connection.  This enables
    /// servers to correctly interpret the opening handshake and subsequent
    /// data being sent from the data, and close the connection if the server
    /// cannot interpret that data in a safe manner.
    (SecWebSocketVersion, SEC_WEBSOCKET_VERSION, 63, b"sec-websocket-version", "sec-websocket-version", [115u8, 101u8, 99u8, 45u8, 119u8, 101u8, 98u8, 115u8, 111u8, 99u8, 107u8, 101u8, 116u8, 45u8, 118u8, 101u8, 114u8, 115u8, 105u8, 111u8, 110u8], bytes_equal_sec_web_socket_version);

    /// Contains information about the software used by the origin server to
    /// handle the request.
    ///
    /// Overly long and detailed Server values should be avoided as they
    /// potentially reveal internal implementation details that might make it
    /// (slightly) easier for attackers to find and exploit known security
    /// holes.
    (Server, SERVER, 64, b"server", "server", [115u8, 101u8, 114u8, 118u8, 101u8, 114u8], bytes_equal_server);

    /// Used to send cookies from the server to the user agent.
    (SetCookie, SET_COOKIE, 65, b"set-cookie", "set-cookie", [115u8, 101u8, 116u8, 45u8, 99u8, 111u8, 111u8, 107u8, 105u8, 101u8], bytes_equal_set_cookie);

    /// Tells the client to communicate with HTTPS instead of using HTTP.
    (StrictTransportSecurity, STRICT_TRANSPORT_SECURITY, 66, b"strict-transport-security", "strict-transport-security", [115u8, 116u8, 114u8, 105u8, 99u8, 116u8, 45u8, 116u8, 114u8, 97u8, 110u8, 115u8, 112u8, 111u8, 114u8, 116u8, 45u8, 115u8, 101u8, 99u8, 117u8, 114u8, 105u8, 116u8, 121u8], bytes_equal_strict_transport_security);

    /// Informs the server of transfer encodings willing to be accepted as part
    /// of the response.
    ///
    /// See also the Transfer-Encoding response header for more details on
    /// transfer encodings. Note that chunked is always acceptable for HTTP/1.1
    /// recipients and you that don't have to specify "chunked" using the TE
    /// header. However, it is useful for setting if the client is accepting
    /// trailer fields in a chunked transfer coding using the "trailers" value.
    (Te, TE, 67, b"te", "te", [116u8, 101u8], bytes_equal_te);

    /// Allows the sender to include additional fields at the end of chunked
    /// messages.
    (Trailer, TRAILER, 68, b"trailer", "trailer", [116u8, 114u8, 97u8, 105u8, 108u8, 101u8, 114u8], bytes_equal_trailer);

    /// Specifies the form of encoding used to safely transfer the entity to the
    /// client.
    ///
    /// `transfer-encoding` is a hop-by-hop header, that is applying to a
    /// message between two nodes, not to a resource itself. Each segment of a
    /// multi-node connection can use different `transfer-encoding` values. If
    /// you want to compress data over the whole connection, use the end-to-end
    /// header `content-encoding` header instead.
    ///
    /// When present on a response to a `HEAD` request that has no body, it
    /// indicates the value that would have applied to the corresponding `GET`
    /// message.
    (TransferEncoding, TRANSFER_ENCODING, 69, b"transfer-encoding", "transfer-encoding", [116u8, 114u8, 97u8, 110u8, 115u8, 102u8, 101u8, 114u8, 45u8, 101u8, 110u8, 99u8, 111u8, 100u8, 105u8, 110u8, 103u8], bytes_equal_transfer_encoding);

    /// Contains a string that allows identifying the requesting client's
    /// software.
    (UserAgent, USER_AGENT, 70, b"user-agent", "user-agent", [117u8, 115u8, 101u8, 114u8, 45u8, 97u8, 103u8, 101u8, 110u8, 116u8], bytes_equal_user_agent);

    /// Used as part of the exchange to upgrade the protocol.
    (Upgrade, UPGRADE, 71, b"upgrade", "upgrade", [117u8, 112u8, 103u8, 114u8, 97u8, 100u8, 101u8], bytes_equal_upgrade);

    /// Sends a signal to the server expressing the client’s preference for an
    /// encrypted and authenticated response.
    (UpgradeInsecureRequests, UPGRADE_INSECURE_REQUESTS, 72, b"upgrade-insecure-requests", "upgrade-insecure-requests", [117u8, 112u8, 103u8, 114u8, 97u8, 100u8, 101u8, 45u8, 105u8, 110u8, 115u8, 101u8, 99u8, 117u8, 114u8, 101u8, 45u8, 114u8, 101u8, 113u8, 117u8, 101u8, 115u8, 116u8, 115u8], bytes_equal_upgrade_insecure_requests);

    /// Determines how to match future requests with cached responses.
    ///
    /// The `vary` HTTP response header determines how to match future request
    /// headers to decide whether a cached response can be used rather than
    /// requesting a fresh one from the origin server. It is used by the server
    /// to indicate which headers it used when selecting a representation of a
    /// resource in a content negotiation algorithm.
    ///
    /// The `vary` header should be set on a 304 Not Modified response exactly
    /// like it would have been set on an equivalent 200 OK response.
    (Vary, VARY, 73, b"vary", "vary", [118u8, 97u8, 114u8, 121u8], bytes_equal_vary);

    /// Added by proxies to track routing.
    ///
    /// The `via` general header is added by proxies, both forward and reverse
    /// proxies, and can appear in the request headers and the response headers.
    /// It is used for tracking message forwards, avoiding request loops, and
    /// identifying the protocol capabilities of senders along the
    /// request/response chain.
    (Via, VIA, 74, b"via", "via", [118u8, 105u8, 97u8], bytes_equal_via);

    /// General HTTP header contains information about possible problems with
    /// the status of the message.
    ///
    /// More than one `warning` header may appear in a response. Warning header
    /// fields can in general be applied to any message, however some warn-codes
    /// are specific to caches and can only be applied to response messages.
    (Warning, WARNING, 75, b"warning", "warning", [119u8, 97u8, 114u8, 110u8, 105u8, 110u8, 103u8], bytes_equal_warning);

    /// Defines the authentication method that should be used to gain access to
    /// a resource.
    (WwwAuthenticate, WWW_AUTHENTICATE, 76, b"www-authenticate", "www-authenticate", [119u8, 119u8, 119u8, 45u8, 97u8, 117u8, 116u8, 104u8, 101u8, 110u8, 116u8, 105u8, 99u8, 97u8, 116u8, 101u8], bytes_equal_www_authenticate);

    /// Marker used by the server to indicate that the MIME types advertised in
    /// the `content-type` headers should not be changed and be followed.
    ///
    /// This allows to opt-out of MIME type sniffing, or, in other words, it is
    /// a way to say that the webmasters knew what they were doing.
    ///
    /// This header was introduced by Microsoft in IE 8 as a way for webmasters
    /// to block content sniffing that was happening and could transform
    /// non-executable MIME types into executable MIME types. Since then, other
    /// browsers have introduced it, even if their MIME sniffing algorithms were
    /// less aggressive.
    ///
    /// Site security testers usually expect this header to be set.
    (XContentTypeOptions, X_CONTENT_TYPE_OPTIONS, 77, b"x-content-type-options", "x-content-type-options", [120u8, 45u8, 99u8, 111u8, 110u8, 116u8, 101u8, 110u8, 116u8, 45u8, 116u8, 121u8, 112u8, 101u8, 45u8, 111u8, 112u8, 116u8, 105u8, 111u8, 110u8, 115u8], bytes_equal_x_content_type_options);

    /// Controls DNS prefetching.
    ///
    /// The `x-dns-prefetch-control` HTTP response header controls DNS
    /// prefetching, a feature by which browsers proactively perform domain name
    /// resolution on both links that the user may choose to follow as well as
    /// URLs for items referenced by the document, including images, CSS,
    /// JavaScript, and so forth.
    ///
    /// This prefetching is performed in the background, so that the DNS is
    /// likely to have been resolved by the time the referenced items are
    /// needed. This reduces latency when the user clicks a link.
    (XDnsPrefetchControl, X_DNS_PREFETCH_CONTROL, 78, b"x-dns-prefetch-control", "x-dns-prefetch-control", [120u8, 45u8, 100u8, 110u8, 115u8, 45u8, 112u8, 114u8, 101u8, 102u8, 101u8, 116u8, 99u8, 104u8, 45u8, 99u8, 111u8, 110u8, 116u8, 114u8, 111u8, 108u8], bytes_equal_x_dns_prefetch_control);

    /// Indicates whether or not a browser should be allowed to render a page in
    /// a frame.
    ///
    /// Sites can use this to avoid clickjacking attacks, by ensuring that their
    /// content is not embedded into other sites.
    ///
    /// The added security is only provided if the user accessing the document
    /// is using a browser supporting `x-frame-options`.
    (XFrameOptions, X_FRAME_OPTIONS, 79, b"x-frame-options", "x-frame-options", [120u8, 45u8, 102u8, 114u8, 97u8, 109u8, 101u8, 45u8, 111u8, 112u8, 116u8, 105u8, 111u8, 110u8, 115u8], bytes_equal_x_frame_options);

    /// Stop pages from loading when an XSS attack is detected.
    ///
    /// The HTTP X-XSS-Protection response header is a feature of Internet
    /// Explorer, Chrome and Safari that stops pages from loading when they
    /// detect reflected cross-site scripting (XSS) attacks. Although these
    /// protections are largely unnecessary in modern browsers when sites
    /// implement a strong Content-Security-Policy that disables the use of
    /// inline JavaScript ('unsafe-inline'), they can still provide protections
    /// for users of older web browsers that don't yet support CSP.
    (XXssProtection, X_XSS_PROTECTION, 80, b"x-xss-protection", "x-xss-protection", [120u8, 45u8, 120u8, 115u8, 115u8, 45u8, 112u8, 114u8, 111u8, 116u8, 101u8, 99u8, 116u8, 105u8, 111u8, 110u8], bytes_equal_x_xss_protection);
}

#[cfg(all(creusot, http_header_name_leaf))]
#[ensures(result)]
fn selected_standard_header_comparator_consumer(name_bytes: &[u8]) -> bool {
    let matches_accept = bytes_equal_accept(name_bytes);
    if !matches_accept {
        proof_assert! {
            name_bytes@ != seq![97u8, 99u8, 99u8, 101u8, 112u8, 116u8]
        };
    }

    let matches_access_control_allow_credentials =
        bytes_equal_access_control_allow_credentials(name_bytes);
    if !matches_access_control_allow_credentials {
        proof_assert! {
            name_bytes@ != seq![
                97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 45u8, 99u8, 111u8, 110u8,
                116u8, 114u8, 111u8, 108u8, 45u8, 97u8, 108u8, 108u8, 111u8, 119u8,
                45u8, 99u8, 114u8, 101u8, 100u8, 101u8, 110u8, 116u8, 105u8, 97u8,
                108u8, 115u8,
            ]
        };
    }

    true
}

#[cfg(creusot)]
impl<T: DeepModel> DeepModel for Repr<T> {
    type DeepModelTy = ReprDeepModel<T::DeepModelTy>;

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! {
            match self {
                Repr::Standard(header) => ReprDeepModel::Standard(header.deep_model()),
                Repr::Custom(value) => ReprDeepModel::Custom(value.deep_model()),
            }
        }
    }
}

#[cfg(creusot)]
impl PartialEq for Repr<Custom> {
    #[ensures(result == self.deep_model().eq_model(rhs.deep_model()))]
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Repr::Standard(lhs), Repr::Standard(rhs)) => lhs == rhs,
            (Repr::Custom(lhs), Repr::Custom(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

#[cfg(creusot)]
impl Eq for Repr<Custom> {}

#[cfg(creusot)]
impl DeepModel for Custom {
    type DeepModelTy = Seq<u8>;

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        pearlite! { self.0@ }
    }
}

#[cfg(creusot)]
#[logic(open(self))]
pub fn header_name_model_bytes(model: ReprDeepModel<Seq<u8>>) -> Seq<u8> {
    pearlite! {
        match model {
            ReprDeepModel::Standard(header) => header.byte_view(),
            ReprDeepModel::Custom(bytes) => bytes,
        }
    }
}

#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn hdr_name_model_bytes(model: ReprDeepModel<(Seq<u8>, bool)>) -> Seq<u8> {
    pearlite! {
        match model {
            ReprDeepModel::Standard(header) => header.byte_view(),
            ReprDeepModel::Custom((bytes, _)) => bytes,
        }
    }
}

#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn hdr_name_conversion_bytes(model: ReprDeepModel<(Seq<u8>, bool)>) -> Seq<Int> {
    pearlite! {
        match model {
            ReprDeepModel::Standard(header) =>
                header.byte_view().map(|byte: u8| byte@),
            ReprDeepModel::Custom((bytes, true)) => bytes.map(|byte: u8| byte@),
            ReprDeepModel::Custom((bytes, false)) => normalize_header_bytes(bytes),
        }
    }
}

/// Valid header name characters
///
/// ```not_rust
///       field-name     = token
///       separators     = "(" | ")" | "<" | ">" | "@"
///                      | "," | ";" | ":" | "\" | <">
///                      | "/" | "[" | "]" | "?" | "="
///                      | "{" | "}" | SP | HT
///       token          = 1*tchar
///       tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*"
///                      / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
///                      / DIGIT / ALPHA
///                      ; any VCHAR, except delimiters
/// ```
// The H1 matcher maps every accepted byte to a valid single-byte UTF-8 codepoint.
// Keep this table for MaybeLower hashing and for matcher parity checks.
#[rustfmt::skip]
const HEADER_CHARS: [u8; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //   x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  1x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  2x
        0,     0,     0,  b'!',     0,  b'#',  b'$',  b'%',  b'&', b'\'', //  3x
        0,     0,  b'*',  b'+',     0,  b'-',  b'.',     0,  b'0',  b'1', //  4x
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',     0,     0, //  5x
        0,     0,     0,     0,     0,  b'a',  b'b',  b'c',  b'd',  b'e', //  6x
     b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm',  b'n',  b'o', //  7x
     b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w',  b'x',  b'y', //  8x
     b'z',     0,     0,     0,  b'^',  b'_',  b'`',  b'a',  b'b',  b'c', //  9x
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm', // 10x
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w', // 11x
     b'x',  b'y',  b'z',     0,  b'|',     0,  b'~',     0,     0,     0, // 12x
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

/// Original HTTP/2.0 and HTTP/3.0 lookup table retained for the all-byte regression test.
#[rustfmt::skip]
#[cfg(test)]
const HEADER_CHARS_H2: [u8; 256] = [
    //  0      1      2      3      4      5      6      7      8      9
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //   x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  1x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  2x
        0,     0,     0,  b'!',  b'"',  b'#',  b'$',  b'%',  b'&', b'\'', //  3x
        0,     0,  b'*',  b'+',     0,  b'-',  b'.',     0,  b'0',  b'1', //  4x
     b'2',  b'3',  b'4',  b'5',  b'6',  b'7',  b'8',  b'9',     0,     0, //  5x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  6x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  7x
        0,     0,     0,     0,     0,     0,     0,     0,     0,     0, //  8x
        0,     0,     0,     0,  b'^',  b'_',  b'`',  b'a',  b'b',  b'c', //  9x
     b'd',  b'e',  b'f',  b'g',  b'h',  b'i',  b'j',  b'k',  b'l',  b'm', // 10x
     b'n',  b'o',  b'p',  b'q',  b'r',  b's',  b't',  b'u',  b'v',  b'w', // 11x
     b'x',  b'y',  b'z',     0,  b'|',     0,  b'~',     0,     0,     0, // 12x
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

#[cfg(creusot)]
impl View for HeaderName {
    type ViewTy = Seq<u8>;

    #[logic]
    fn view(self) -> Self::ViewTy {
        pearlite! {
            match self.inner {
                Repr::Standard(header) => header.byte_view(),
                Repr::Custom(Custom(value)) => value@,
            }
        }
    }
}

#[cfg(creusot)]
impl DeepModel for HeaderName {
    type DeepModelTy = ReprDeepModel<Seq<u8>>;

    #[logic(open(self))]
    #[ensures(header_name_model_bytes(result) == self@)]
    fn deep_model(self) -> Self::DeepModelTy {
        self.inner.deep_model()
    }
}

#[cfg(creusot)]
impl DeepModel for HdrName<'_> {
    type DeepModelTy = ReprDeepModel<(Seq<u8>, bool)>;

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        self.inner.deep_model()
    }
}

#[cfg(creusot)]
impl Invariant for HdrName<'_> {
    #[logic(open)]
    fn invariant(self) -> bool {
        pearlite! {
            hdr_name_model_bytes(self.deep_model()).len() <= isize::MAX@
                && match self.deep_model() {
                    ReprDeepModel::Standard(_) => true,
                    ReprDeepModel::Custom((bytes, lower)) =>
                        !lower || creusot_std::std::string::valid_utf8(bytes),
                }
        }
    }
}

#[cfg(creusot)]
#[logic(open(self))]
fn header_chars_lower_tchar(byte: u8) -> bool {
    pearlite! {
        byte@ == 33 || byte@ == 35 || byte@ == 36 || byte@ == 37
            || byte@ == 38 || byte@ == 39 || byte@ == 42 || byte@ == 43
            || byte@ == 45 || byte@ == 46 || (48 <= byte@ && byte@ <= 57)
            || (94 <= byte@ && byte@ <= 96) || (97 <= byte@ && byte@ <= 122)
            || byte@ == 124 || byte@ == 126
    }
}

#[cfg(creusot)]
#[logic(open(self))]
pub fn header_chars_byte(byte: u8) -> Int {
    pearlite! {
        if 65 <= byte@ && byte@ <= 90 { byte@ + 32 }
        else if header_chars_lower_tchar(byte) { byte@ }
        else { 0 }
    }
}

#[cfg(creusot)]
#[logic(open(self))]
pub fn header_chars_h2_byte(byte: u8) -> Int {
    pearlite! {
        if byte@ == 34 || header_chars_lower_tchar(byte) { byte@ }
        else { 0 }
    }
}

#[cfg(creusot)]
#[logic(open(self))]
pub fn header_name_matches_text(name: ReprDeepModel<Seq<u8>>, text: Seq<char>) -> bool {
    let name_bytes = header_name_model_bytes(name);
    let text_bytes = text.to_bytes();
    pearlite! {
        name_bytes.len() == text_bytes.len()
            && forall<i: Int> 0 <= i && i < name_bytes.len()
                ==> name_bytes[i]@ == header_chars_byte(text_bytes[i])
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn header_name_matches_hdr_name(
    name: ReprDeepModel<Seq<u8>>,
    other: ReprDeepModel<(Seq<u8>, bool)>,
) -> bool {
    pearlite! {
        match (name, other) {
            (ReprDeepModel::Standard(lhs), ReprDeepModel::Standard(rhs)) => lhs == rhs,
            (ReprDeepModel::Custom(lhs), ReprDeepModel::Custom((rhs, lower))) =>
                if lower { lhs == rhs } else {
                    lhs.len() == rhs.len()
                        && forall<i: Int> 0 <= i && i < lhs.len()
                            ==> lhs[i]@ == header_chars_byte(rhs[i])
                },
            _ => false,
        }
    }
}

#[cfg(creusot)]
impl PartialEqModel<ReprDeepModel<(Seq<u8>, bool)>> for ReprDeepModel<Seq<u8>> {
    #[logic(open)]
    fn eq_model(self, rhs: ReprDeepModel<(Seq<u8>, bool)>) -> bool {
        pearlite! { header_name_matches_hdr_name(self, rhs) }
    }
}

#[cfg(creusot)]
impl PartialEqModel<Seq<char>> for ReprDeepModel<Seq<u8>> {
    #[logic(open(self))]
    fn eq_model(self, rhs: Seq<char>) -> bool {
        pearlite! { header_name_matches_text(self, rhs) }
    }
}

#[cfg(creusot)]
impl BorrowModel<Seq<char>> for ReprDeepModel<Seq<u8>> {
    #[logic(open(self))]
    fn borrowed_model(self, rhs: Seq<char>) -> bool {
        pearlite! { header_name_model_bytes(self) == rhs.to_bytes() }
    }
}

#[cfg(creusot)]
impl PartialEqModel<ReprDeepModel<Seq<u8>>> for Seq<char> {
    #[logic(open(self))]
    fn eq_model(self, rhs: ReprDeepModel<Seq<u8>>) -> bool {
        pearlite! { header_name_matches_text(rhs, self) }
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn normalize_header_bytes(bytes: Seq<u8>) -> Seq<Int> {
    pearlite! { bytes.map(|byte: u8| header_chars_byte(byte)) }
}

#[cfg(creusot)]
#[logic(open)]
pub fn header_bytes_allowed(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i> 0 <= i && i < bytes.len()
            ==> header_chars_byte(bytes[i]) != 0
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn lowercase_header_bytes_allowed(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i> 0 <= i && i < bytes.len()
            ==> bytes[i]@ != 0 && header_chars_h2_byte(bytes[i]) == bytes[i]@
    }
}

#[cfg(creusot)]
#[logic(open)]
fn header_chars_h2_nonzero_is_identity(byte: u8) -> bool {
    pearlite! {
        header_chars_h2_byte(byte) != 0
            ==> byte@ != 0 && header_chars_h2_byte(byte) == byte@
    }
}

#[inline]
#[cfg_attr(creusot, ensures(result@ == header_chars_byte(byte)))]
#[cfg_attr(creusot, ensures(result@ < 128))]
fn header_chars_byte_value(byte: u8) -> u8 {
    match byte {
        b'A'..=b'Z' => byte + 32,
        33
        | 35
        | 36
        | 37
        | 38
        | 39
        | 42
        | 43
        | 45
        | 46
        | 48..=57
        | 94..=96
        | 97..=122
        | 124
        | 126 => byte,
        _ => 0,
    }
}

#[inline]
#[cfg_attr(creusot, ensures(result@ == header_chars_h2_byte(byte)))]
#[cfg_attr(creusot, ensures(result@ < 128))]
const fn header_chars_h2_byte_value(byte: u8) -> u8 {
    match byte {
        b'"'
        | 33
        | 35
        | 36
        | 37
        | 38
        | 39
        | 42
        | 43
        | 45
        | 46
        | 48..=57
        | 94..=96
        | 97..=122
        | 124
        | 126 => byte,
        _ => 0,
    }
}

#[inline]
#[cfg_attr(creusot, ensures(result@ == if h2 {
    header_chars_h2_byte(byte)
} else {
    header_chars_byte(byte)
}))]
#[cfg_attr(creusot, ensures(result@ < 128))]
fn header_chars_value(byte: u8, h2: bool) -> u8 {
    if h2 {
        header_chars_h2_byte_value(byte)
    } else {
        header_chars_byte_value(byte)
    }
}

#[cfg(creusot)]
#[logic(open)]
#[doc(hidden)]
pub fn hdr_parse_success(data: Seq<u8>, model: ReprDeepModel<(Seq<u8>, bool)>) -> bool {
    pearlite! {
        0 < data.len() && data.len() <= 65535
            && match model {
                ReprDeepModel::Standard(_) => data.len() <= SCRATCH_BUF_SIZE@,
                ReprDeepModel::Custom((bytes, lower)) =>
                    lower == (data.len() <= SCRATCH_BUF_SIZE@)
                        && (!lower ==> bytes == data),
            }
            && (if data.len() <= SCRATCH_BUF_SIZE@ {
                header_bytes_allowed(data)
                    && hdr_name_model_bytes(model).len() == data.len()
                    && forall<i: Int> 0 <= i && i < data.len() ==>
                        hdr_name_model_bytes(model)[i]@ == header_chars_byte(data[i])
            } else {
                hdr_name_model_bytes(model) == data
            })
    }
}

#[cfg_attr(creusot, ensures(match result {
    Ok(name) => 0 < data@.len() && data@.len() <= 65535
        && match name.deep_model() {
            ReprDeepModel::Standard(_) => data@.len() <= SCRATCH_BUF_SIZE@,
            ReprDeepModel::Custom((bytes, lower)) =>
                lower == (data@.len() <= SCRATCH_BUF_SIZE@)
                    && (!lower ==> bytes == data@),
        }
        && (if data@.len() <= SCRATCH_BUF_SIZE@ {
            (if h2 {
                lowercase_header_bytes_allowed(data@)
            } else {
                header_bytes_allowed(data@)
            })
                && hdr_name_model_bytes(name.deep_model()).len() == data@.len()
                && forall<i: Int> 0 <= i && i < data@.len() ==>
                    hdr_name_model_bytes(name.deep_model())[i]@ == if h2 {
                        header_chars_h2_byte(data@[i])
                    } else {
                        header_chars_byte(data@[i])
                    }
        } else {
            hdr_name_model_bytes(name.deep_model()) == data@
        }),
    Err(_) => data@.len() == 0 || data@.len() > 65535
        || (data@.len() <= SCRATCH_BUF_SIZE@ && (if h2 {
            !lowercase_header_bytes_allowed(data@)
        } else {
            !header_bytes_allowed(data@)
        })),
}))]
fn parse_hdr<'a>(
    data: &'a [u8],
    b: &'a mut [u8; SCRATCH_BUF_SIZE],
    h2: bool,
) -> Result<HdrName<'a>, InvalidHeaderName> {
    match data.len() {
        0 => Err(InvalidHeaderName::new()),
        len @ 1..=SCRATCH_BUF_SIZE => {
            // Normalize into the fixed scratch buffer before checking standard names.
            let mut i = 0;
            let mut all_nonzero = true;
            #[cfg_attr(creusot, invariant(i@ <= data@.len()))]
            #[cfg_attr(creusot, invariant(all_nonzero == forall<j: Int>
                0 <= j && j < i@ ==>
                    (if h2 { header_chars_h2_byte(data@[j]) }
                     else { header_chars_byte(data@[j]) }) != 0))]
            #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < i@ ==>
                b@[j]@ == if h2 { header_chars_h2_byte(data@[j]) }
                          else { header_chars_byte(data@[j]) }))]
            #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < i@ ==> b@[j]@ < 128))]
            #[cfg_attr(creusot, variant(data@.len() - i@))]
            while i < len {
                let normalized = header_chars_value(data[i], h2);
                b[i] = normalized;
                all_nonzero &= normalized != 0;
                i += 1;
            }

            if !all_nonzero {
                return Err(InvalidHeaderName::new());
            }

            #[cfg(creusot)]
            if h2 {
                proof_assert! {
                    forall<j: Int> 0 <= j && j < data@.len() ==>
                        header_chars_h2_nonzero_is_identity(data@[j]);
                    lowercase_header_bytes_allowed(data@)
                };
            } else {
                proof_assert! { header_bytes_allowed(data@) };
            }

            let name: &'a [u8] = &b.as_slice()[0..len];
            #[cfg(creusot)]
            proof_assert! {
                name@.len() == data@.len();
                forall<j: Int> 0 <= j && j < data@.len() ==>
                    name@[j]@ == if h2 {
                        header_chars_h2_byte(data@[j])
                    } else {
                        header_chars_byte(data@[j])
                    }
            };
            match StandardHeader::from_bytes(name) {
                Some(sh) => Ok(sh.into()),
                None => {
                    #[cfg(creusot)]
                    proof_assert! {
                        forall<j: Int> 0 <= j && j < name@.len() ==> name@[j]@ < 128;
                        crate::ascii::ascii_bytes_are_valid_utf8(name@);
                        creusot_std::std::string::valid_utf8(name@)
                    };
                    Ok(HdrName::custom(name, true))
                }
            }
        }
        SCRATCH_BUF_OVERFLOW..=super::MAX_HEADER_NAME_LEN => Ok(HdrName::custom(data, false)),
        _ => Err(InvalidHeaderName::new()),
    }
}

impl<'a> From<StandardHeader> for HdrName<'a> {
    #[cfg_attr(
        creusot,
        ensures(result.deep_model() == ReprDeepModel::Standard(hdr.deep_model()))
    )]
    fn from(hdr: StandardHeader) -> HdrName<'a> {
        HdrName {
            inner: Repr::Standard(hdr),
        }
    }
}

impl HeaderName {
    /// Converts a slice of bytes to an HTTP header name.
    ///
    /// This function normalizes the input.
    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(src@)
            && 0 < src@.len() && src@.len() <= 65535
            && header_bytes_allowed(src@),
        Err(_) => src@.len() == 0 || src@.len() > 65535
            || !header_bytes_allowed(src@),
    }))]
    pub fn from_bytes(src: &[u8]) -> Result<HeaderName, InvalidHeaderName> {
        let mut buf = [0u8; SCRATCH_BUF_SIZE];
        match parse_hdr(src, &mut buf, false)?.inner {
            Repr::Standard(std) => Ok(std.into()),
            Repr::Custom(MaybeLower { buf, lower: true }) => {
                let buf = Bytes::copy_from_slice(buf);
                // Safety: the invariant on MaybeLower ensures buf is valid UTF-8.
                let val = unsafe { ByteStr::from_utf8_unchecked(buf) };
                Ok(Custom(val).into())
            }
            Repr::Custom(MaybeLower { buf, lower: false }) => {
                let mut dst = BytesMut::with_capacity(buf.len());

                let mut i = 0;
                #[cfg_attr(creusot, invariant(i@ <= buf@.len()))]
                #[cfg_attr(creusot, invariant(crate::bytes_model::bytes_mut_seq(dst).len() == i@))]
                #[cfg_attr(creusot, invariant(forall<j: Int>
                    0 <= j && j < i@ ==>
                        crate::bytes_model::bytes_mut_seq(dst)[j]@ == header_chars_byte(buf@[j])
                ))]
                #[cfg_attr(creusot, invariant(forall<j: Int>
                    0 <= j && j < i@ ==> header_chars_byte(buf@[j]) != 0
                ))]
                #[cfg_attr(creusot, invariant(forall<j: Int>
                    0 <= j && j < i@ ==>
                        crate::bytes_model::bytes_mut_seq(dst)[j]@ < 128
                ))]
                #[cfg_attr(creusot, variant(buf@.len() - i@))]
                while i < buf.len() {
                    // The H1 matcher maps accepted bytes to valid single-byte UTF-8
                    let b = header_chars_byte_value(buf[i]);

                    if b == 0 {
                        return Err(InvalidHeaderName::new());
                    }

                    dst.extend_from_slice(std::slice::from_ref(&b));
                    i += 1;
                }

                #[cfg(creusot)]
                proof_assert! {
                    forall<j: Int> 0 <= j
                        && j < crate::bytes_model::bytes_mut_seq(dst).len() ==>
                        crate::bytes_model::bytes_mut_seq(dst)[j]@ < 128;
                    crate::ascii::ascii_bytes_are_valid_utf8(
                        crate::bytes_model::bytes_mut_seq(dst)
                    );
                    creusot_std::std::string::valid_utf8(
                        crate::bytes_model::bytes_mut_seq(dst)
                    )
                };

                // Safety: the loop above maps all bytes in buf to valid single byte
                // UTF-8 before copying them into dst. This means that dst (and hence
                // dst.freeze()) is valid UTF-8.
                let val = unsafe { ByteStr::from_utf8_unchecked(dst.freeze()) };

                Ok(Custom(val).into())
            }
        }
    }

    /// Converts a slice of bytes to an HTTP header name.
    ///
    /// This function expects the input to only contain lowercase characters.
    /// This is useful when decoding HTTP/2.0 or HTTP/3.0 headers. Both
    /// require that all headers be represented in lower case.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::*;
    ///
    /// // Parsing a lower case header
    /// let hdr = HeaderName::from_lowercase(b"content-length").unwrap();
    /// assert_eq!(CONTENT_LENGTH, hdr);
    ///
    /// // Parsing a header that contains uppercase characters
    /// assert!(HeaderName::from_lowercase(b"Content-Length").is_err());
    /// ```
    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@ == src@ && 0 < src@.len()
            && src@.len() <= 65535 && lowercase_header_bytes_allowed(src@),
        Err(_) => src@.len() == 0 || src@.len() > 65535
            || !lowercase_header_bytes_allowed(src@),
    }))]
    pub fn from_lowercase(src: &[u8]) -> Result<HeaderName, InvalidHeaderName> {
        let mut buf = [0u8; SCRATCH_BUF_SIZE];
        match parse_hdr(src, &mut buf, true)?.inner {
            Repr::Standard(std) => Ok(std.into()),
            Repr::Custom(MaybeLower { buf, lower: true }) => {
                let buf = Bytes::copy_from_slice(buf);
                // Safety: the invariant on MaybeLower ensures buf is valid UTF-8.
                let val = unsafe { ByteStr::from_utf8_unchecked(buf) };
                Ok(Custom(val).into())
            }
            Repr::Custom(MaybeLower { buf, lower: false }) => {
                let mut i = 0;
                #[cfg_attr(creusot, invariant(i@ <= buf@.len()))]
                #[cfg_attr(creusot, invariant(forall<j: Int>
                    0 <= j && j < i@ ==> header_chars_h2_byte(buf@[j]) != 0
                ))]
                #[cfg_attr(creusot, variant(buf@.len() - i@))]
                while i < buf.len() {
                    // The H2 matcher maps bytes outside its accepted set to 0;
                    // the quote byte remains accepted to preserve existing behavior.
                    if header_chars_h2_byte_value(buf[i]) == 0 {
                        return Err(InvalidHeaderName::new());
                    }
                    i += 1;
                }

                #[cfg(creusot)]
                proof_assert! {
                    forall<j: Int> 0 <= j && j < buf@.len() ==> buf@[j]@ < 128;
                    crate::ascii::ascii_bytes_are_valid_utf8(buf@);
                    creusot_std::std::string::valid_utf8(buf@)
                };

                let buf = Bytes::copy_from_slice(buf);
                // Safety: the loop above checks that each byte of buf (either
                // version) is valid UTF-8.
                let val = unsafe { ByteStr::from_utf8_unchecked(buf) };
                Ok(Custom(val).into())
            }
        }
    }

    /// Converts a static string to a HTTP header name.
    ///
    /// This function requires the static string to only contain lowercase
    /// characters, numerals and symbols, as per the HTTP/2.0 specification
    /// and header names internal representation within this library.
    ///
    /// # Panics
    ///
    /// This function panics when the static string is a invalid header.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::*;
    /// // Parsing a standard header
    /// let hdr = HeaderName::from_static("content-length");
    /// assert_eq!(CONTENT_LENGTH, hdr);
    ///
    /// // Parsing a custom header
    /// let CUSTOM_HEADER: &'static str = "custom-header";
    ///
    /// let a = HeaderName::from_lowercase(b"custom-header").unwrap();
    /// let b = HeaderName::from_static(CUSTOM_HEADER);
    /// assert_eq!(a, b);
    /// ```
    ///
    /// ```should_panic
    /// # use http::header::*;
    /// #
    /// // Parsing a header that contains invalid symbols:
    /// HeaderName::from_static("content{}{}length"); // This line panics!
    ///
    /// // Parsing a header that contains invalid uppercase characters.
    /// let a = HeaderName::from_static("foobar");
    /// let b = HeaderName::from_static("FOOBAR"); // This line panics!
    /// ```
    #[cfg_attr(creusot, requires(0 < src@.to_bytes().len()
        && src@.to_bytes().len() <= 65535
        && lowercase_header_bytes_allowed(src@.to_bytes())))]
    #[cfg_attr(creusot, ensures(result@ == src@.to_bytes()))]
    pub const fn from_static(src: &'static str) -> HeaderName {
        let name_bytes = src.as_bytes();
        if let Some(standard) = StandardHeader::from_bytes(name_bytes) {
            return HeaderName {
                inner: Repr::Standard(standard),
            };
        }

        if name_bytes.is_empty() || name_bytes.len() > super::MAX_HEADER_NAME_LEN || {
            let mut i = 0;
            #[cfg_attr(creusot, invariant(i@ <= name_bytes@.len()))]
            #[cfg_attr(creusot, invariant(forall<j: Int>
                0 <= j && j < i@ ==> header_chars_h2_byte(name_bytes@[j]) != 0
            ))]
            #[cfg_attr(creusot, variant(name_bytes@.len() - i@))]
            loop {
                if i >= name_bytes.len() {
                    break false;
                } else if header_chars_h2_byte_value(name_bytes[i]) == 0 {
                    break true;
                }
                i += 1;
            }
        } {
            // Invalid header name
            panic!("HeaderName::from_static with invalid bytes")
        }

        HeaderName {
            inner: Repr::Custom(Custom(ByteStr::from_static(src))),
        }
    }

    /// Returns a `str` representation of the header.
    ///
    /// The returned string will always be lower case.
    #[inline]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@))]
    pub fn as_str(&self) -> &str {
        match self.inner {
            Repr::Standard(v) => v.as_str(),
            Repr::Custom(ref v) => &v.0,
        }
    }

    #[cfg_attr(creusot, ensures(crate::bytes_model::bytes_seq(result) == self@))]
    pub(super) fn into_bytes(self) -> Bytes {
        match self.inner {
            Repr::Standard(header) => Bytes::from_static(header.as_str().as_bytes()),
            Repr::Custom(Custom(bytes)) => Bytes::from(bytes),
        }
    }
}

impl FromStr for HeaderName {
    type Err = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(s@.to_bytes())
            && 0 < s@.to_bytes().len() && s@.to_bytes().len() <= 65535
            && header_bytes_allowed(s@.to_bytes()),
        Err(_) => s@.to_bytes().len() == 0 || s@.to_bytes().len() > 65535
            || !header_bytes_allowed(s@.to_bytes()),
    }))]
    fn from_str(s: &str) -> Result<HeaderName, InvalidHeaderName> {
        match HeaderName::from_bytes(s.as_bytes()) {
            Ok(name) => Ok(name),
            Err(_) => Err(InvalidHeaderName { _priv: () }),
        }
    }
}

impl AsRef<str> for HeaderName {
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@))]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for HeaderName {
    #[cfg_attr(creusot, ensures(result@ == self@))]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl Borrow<str> for HeaderName {
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self@))]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

// These consistency probes deliberately have an unprovable postcondition. They
// are only included in the isolated header verification harness when checking
// that its open HeaderName model and standard-name constants do not make an
// arbitrary false goal provable.
#[cfg(all(creusot, feature = "negative-model-probes"))]
#[allow(dead_code, unused_variables)]
#[requires(header_name_model_bytes(value.deep_model()) == value@)]
#[ensures(false)]
fn header_same_module_program_negative(value: &HeaderName) {}

#[cfg(all(creusot, feature = "negative-model-probes"))]
#[allow(dead_code, unused_variables)]
#[logic]
#[requires(header_name_model_bytes(value.deep_model()) == value@)]
#[ensures(false)]
fn header_same_module_logic_negative(value: &HeaderName) {}

#[cfg(all(creusot, feature = "negative-model-probes"))]
#[allow(dead_code)]
#[logic]
#[ensures(false)]
fn header_no_const_logic_negative() {}

#[cfg(all(creusot, feature = "negative-model-probes"))]
const HEADER_LOGIC_PROBE_TEXT: &str = "probe";

#[cfg(all(creusot, feature = "negative-model-probes"))]
#[allow(dead_code)]
#[logic]
#[requires(HEADER_LOGIC_PROBE_TEXT@ == HEADER_LOGIC_PROBE_TEXT@)]
#[ensures(false)]
fn header_one_const_logic_negative() {}

impl fmt::Debug for HeaderName {
    #[cfg_attr(creusot, ensures(creusot_std::std::fmt::formatter_extends(
        fmt.deep_model(), (^fmt).deep_model()
    )))]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), fmt)
    }
}

impl fmt::Display for HeaderName {
    #[cfg_attr(creusot, ensures(creusot_std::std::fmt::formatter_extends(
        fmt.deep_model(), (^fmt).deep_model()
    )))]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}

impl InvalidHeaderName {
    pub(super) fn new() -> InvalidHeaderName {
        InvalidHeaderName { _priv: () }
    }
}

impl From<&HeaderName> for HeaderName {
    #[cfg_attr(creusot, ensures(result.deep_model() == src.deep_model()))]
    #[cfg_attr(creusot, ensures(result@ == src@))]
    fn from(src: &HeaderName) -> HeaderName {
        src.clone()
    }
}

#[doc(hidden)]
impl<T> From<Repr<T>> for Bytes
where
    T: Into<Bytes>,
{
    #[cfg_attr(creusot, requires(match repr {
        Repr::Standard(_) => true,
        Repr::Custom(value) => <T as Into<Bytes>>::into.precondition((value,)),
    }))]
    #[cfg_attr(creusot, ensures(match repr {
        Repr::Standard(header) => crate::bytes_model::bytes_seq(result) == header.deep_model().byte_view(),
        Repr::Custom(value) => <T as Into<Bytes>>::into.postcondition((value,), result),
    }))]
    fn from(repr: Repr<T>) -> Bytes {
        match repr {
            Repr::Standard(header) => Bytes::from_static(header.as_str().as_bytes()),
            Repr::Custom(header) => header.into(),
        }
    }
}

impl From<Custom> for Bytes {
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(crate::bytes_model::bytes_seq(result) == src.deep_model())
    )]
    fn from(src: Custom) -> Bytes {
        let Custom(inner) = src;
        Bytes::from(inner)
    }
}

impl TryFrom<&str> for HeaderName {
    type Error = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(s@.to_bytes())
            && 0 < s@.to_bytes().len() && s@.to_bytes().len() <= 65535
            && header_bytes_allowed(s@.to_bytes()),
        Err(_) => s@.to_bytes().len() == 0 || s@.to_bytes().len() > 65535
            || !header_bytes_allowed(s@.to_bytes()),
    }))]
    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_bytes(s.as_bytes())
    }
}

impl TryFrom<&String> for HeaderName {
    type Error = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(s@.to_bytes())
            && 0 < s@.to_bytes().len() && s@.to_bytes().len() <= 65535
            && header_bytes_allowed(s@.to_bytes()),
        Err(_) => s@.to_bytes().len() == 0 || s@.to_bytes().len() > 65535
            || !header_bytes_allowed(s@.to_bytes()),
    }))]
    #[inline]
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Self::from_bytes(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for HeaderName {
    type Error = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(s@)
            && 0 < s@.len() && s@.len() <= 65535 && header_bytes_allowed(s@),
        Err(_) => s@.len() == 0 || s@.len() > 65535 || !header_bytes_allowed(s@),
    }))]
    #[inline]
    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(s)
    }
}

impl TryFrom<String> for HeaderName {
    type Error = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(s@.to_bytes())
            && 0 < s@.to_bytes().len() && s@.to_bytes().len() <= 65535
            && header_bytes_allowed(s@.to_bytes()),
        Err(_) => s@.to_bytes().len() == 0 || s@.to_bytes().len() > 65535
            || !header_bytes_allowed(s@.to_bytes()),
    }))]
    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_bytes(s.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for HeaderName {
    type Error = InvalidHeaderName;

    #[cfg_attr(creusot, ensures(match result {
        Ok(name) => name@.map(|byte: u8| byte@) == normalize_header_bytes(vec@)
            && 0 < vec@.len() && vec@.len() <= 65535 && header_bytes_allowed(vec@),
        Err(_) => vec@.len() == 0 || vec@.len() > 65535 || !header_bytes_allowed(vec@),
    }))]
    #[inline]
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(&vec)
    }
}

#[doc(hidden)]
impl From<StandardHeader> for HeaderName {
    #[cfg_attr(
        creusot,
        ensures(result.deep_model() == ReprDeepModel::Standard(src.deep_model()))
    )]
    #[cfg_attr(creusot, ensures(result@ == src.deep_model().byte_view()))]
    fn from(src: StandardHeader) -> HeaderName {
        HeaderName {
            inner: Repr::Standard(src),
        }
    }
}

#[doc(hidden)]
impl From<Custom> for HeaderName {
    #[cfg_attr(
        creusot,
        ensures(result.deep_model() == ReprDeepModel::Custom(src.deep_model()))
    )]
    #[cfg_attr(creusot, ensures(result@ == src.deep_model()))]
    fn from(src: Custom) -> HeaderName {
        HeaderName {
            inner: Repr::Custom(src),
        }
    }
}

impl PartialEq<&HeaderName> for HeaderName {
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == (self.deep_model() == other.deep_model()))
    )]
    fn eq(&self, other: &&HeaderName) -> bool {
        *self == **other
    }
}

impl PartialEq<HeaderName> for &HeaderName {
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == (self.deep_model() == other.deep_model()))
    )]
    fn eq(&self, other: &HeaderName) -> bool {
        *other == *self
    }
}

impl PartialEq<str> for HeaderName {
    /// Performs a case-insensitive comparison of the string against the header
    /// name
    ///
    /// # Examples
    ///
    /// ```
    /// use http::header::CONTENT_LENGTH;
    ///
    /// assert_eq!(CONTENT_LENGTH, "content-length");
    /// assert_eq!(CONTENT_LENGTH, "Content-Length");
    /// assert_ne!(CONTENT_LENGTH, "content length");
    /// ```
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == header_name_matches_text(self.deep_model(), other@))
    )]
    fn eq(&self, other: &str) -> bool {
        eq_ignore_ascii_case(self.as_ref(), other.as_bytes())
    }
}

impl PartialEq<HeaderName> for str {
    /// Performs a case-insensitive comparison of the string against the header
    /// name
    ///
    /// # Examples
    ///
    /// ```
    /// use http::header::CONTENT_LENGTH;
    ///
    /// assert_eq!(CONTENT_LENGTH, "content-length");
    /// assert_eq!(CONTENT_LENGTH, "Content-Length");
    /// assert_ne!(CONTENT_LENGTH, "content length");
    /// ```
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == header_name_matches_text(other.deep_model(), self@))
    )]
    fn eq(&self, other: &HeaderName) -> bool {
        *other == *self
    }
}

impl PartialEq<&str> for HeaderName {
    /// Performs a case-insensitive comparison of the string against the header
    /// name
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == header_name_matches_text(self.deep_model(), (**other)@))
    )]
    fn eq(&self, other: &&str) -> bool {
        *self == **other
    }
}

impl PartialEq<HeaderName> for &str {
    /// Performs a case-insensitive comparison of the string against the header
    /// name
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == header_name_matches_text(other.deep_model(), (**self)@))
    )]
    fn eq(&self, other: &HeaderName) -> bool {
        *other == *self
    }
}

impl fmt::Debug for InvalidHeaderName {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidHeaderName")
            // skip _priv noise
            .finish()
    }
}

impl fmt::Display for InvalidHeaderName {
    #[cfg_attr(
        creusot,
        ensures(creusot_std::std::fmt::formatter_extends(
            f.deep_model(),
            (^f).deep_model()
        ))
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid HTTP header name")
    }
}

impl Error for InvalidHeaderName {}

// ===== HdrName =====

impl<'a> HdrName<'a> {
    // Precondition: if lower then buf is valid UTF-8
    #[cfg_attr(creusot, requires(buf@.len() <= isize::MAX@))]
    #[cfg_attr(creusot, requires(!lower || creusot_std::std::string::valid_utf8(buf@)))]
    #[cfg_attr(
        creusot,
        ensures(result.deep_model() == ReprDeepModel::Custom((buf@, lower)))
    )]
    fn custom(buf: &'a [u8], lower: bool) -> HdrName<'a> {
        HdrName {
            // Invariant (on MaybeLower): follows from the precondition
            inner: Repr::Custom(MaybeLower { buf, lower }),
        }
    }

    #[cfg_attr(creusot, requires(forall<parsed: HdrName<'_>>
        creusot_std::invariant::inv(parsed)
            && hdr_parse_success(hdr@, parsed.deep_model())
            ==> f.precondition((parsed,))))]
    #[cfg_attr(creusot, ensures(match result {
        Ok(output) => exists<parsed: HdrName<'_>>
            creusot_std::invariant::inv(parsed)
                && hdr_parse_success(hdr@, parsed.deep_model())
                && f.postcondition_once((parsed,), output),
        Err(_) => hdr@.len() == 0 || hdr@.len() > 65535
            || (hdr@.len() <= SCRATCH_BUF_SIZE@ && !header_bytes_allowed(hdr@)),
    }))]
    pub fn from_bytes<F, U>(hdr: &[u8], f: F) -> Result<U, InvalidHeaderName>
    where
        F: FnOnce(HdrName<'_>) -> U,
    {
        let mut buf = [0u8; SCRATCH_BUF_SIZE];
        let hdr = parse_hdr(hdr, &mut buf, false)?;
        Ok(f(hdr))
    }

    #[cfg_attr(creusot, requires(
        0 < hdr@.to_bytes().len()
            && hdr@.to_bytes().len() <= 65535
            && (hdr@.to_bytes().len() > SCRATCH_BUF_SIZE@
                || header_bytes_allowed(hdr@.to_bytes()))
    ))]
    #[cfg_attr(creusot, requires(forall<parsed: HdrName<'_>>
        creusot_std::invariant::inv(parsed)
            && hdr_parse_success(hdr@.to_bytes(), parsed.deep_model())
            ==> f.precondition((parsed,))))]
    #[cfg_attr(creusot, ensures(exists<parsed: HdrName<'_>>
        creusot_std::invariant::inv(parsed)
            && hdr_parse_success(hdr@.to_bytes(), parsed.deep_model())
            && f.postcondition_once((parsed,), result)
    ))]
    pub fn from_static<F, U>(hdr: &'static str, f: F) -> U
    where
        F: FnOnce(HdrName<'_>) -> U,
    {
        let mut buf = [0u8; SCRATCH_BUF_SIZE];
        let hdr = parse_hdr(hdr.as_bytes(), &mut buf, false).expect("static str is invalid name");
        f(hdr)
    }
}

#[doc(hidden)]
impl<'a> From<HdrName<'a>> for HeaderName {
    #[cfg_attr(
        creusot,
        requires(hdr_name_model_bytes(src.deep_model()).len() <= isize::MAX@)
    )]
    #[cfg_attr(
        creusot,
        requires(match src.deep_model() {
            ReprDeepModel::Standard(_) => true,
            ReprDeepModel::Custom((bytes, lower)) =>
                !lower || creusot_std::std::string::valid_utf8(bytes),
        })
    )]
    #[cfg_attr(
        creusot,
        ensures(result@.map(|byte: u8| byte@) == hdr_name_conversion_bytes(src.deep_model()))
    )]
    fn from(src: HdrName<'a>) -> HeaderName {
        match src.inner {
            Repr::Standard(s) => HeaderName {
                inner: Repr::Standard(s),
            },
            Repr::Custom(maybe_lower) => {
                if maybe_lower.lower {
                    let buf = Bytes::copy_from_slice(maybe_lower.buf);
                    // Safety: the invariant on MaybeLower ensures buf is valid UTF-8.
                    let byte_str = unsafe { ByteStr::from_utf8_unchecked(buf) };

                    HeaderName {
                        inner: Repr::Custom(Custom(byte_str)),
                    }
                } else {
                    let mut dst = BytesMut::with_capacity(maybe_lower.buf.len());

                    let mut i = 0;
                    #[cfg_attr(creusot, invariant(i@ <= maybe_lower.buf@.len()))]
                    #[cfg_attr(creusot, invariant(crate::bytes_model::bytes_mut_seq(dst).len() == i@))]
                    #[cfg_attr(creusot, invariant(forall<j: Int>
                        0 <= j && j < i@ ==>
                            crate::bytes_model::bytes_mut_seq(dst)[j]@ ==
                                header_chars_byte(maybe_lower.buf@[j])
                    ))]
                    #[cfg_attr(creusot, invariant(forall<j: Int>
                        0 <= j && j < i@ ==>
                            crate::bytes_model::bytes_mut_seq(dst)[j]@ < 128
                    ))]
                    #[cfg_attr(creusot, variant(maybe_lower.buf@.len() - i@))]
                    while i < maybe_lower.buf.len() {
                        // The H1 matcher maps each byte to a valid single-byte UTF-8
                        // codepoint.
                        let normalized = header_chars_byte_value(maybe_lower.buf[i]);
                        dst.extend_from_slice(std::slice::from_ref(&normalized));
                        i += 1;
                    }

                    #[cfg(creusot)]
                    proof_assert! {
                        crate::bytes_model::bytes_mut_seq(dst).len()
                            == maybe_lower.buf@.len();
                        forall<j: Int> 0 <= j
                            && j < crate::bytes_model::bytes_mut_seq(dst).len() ==>
                            crate::bytes_model::bytes_mut_seq(dst)[j]@
                                == header_chars_byte(maybe_lower.buf@[j]);
                        crate::bytes_model::bytes_mut_seq(dst)
                            .map(|byte: u8| byte@)
                            .ext_eq(normalize_header_bytes(maybe_lower.buf@))
                    };

                    #[cfg(creusot)]
                    proof_assert! {
                        forall<j: Int> 0 <= j
                            && j < crate::bytes_model::bytes_mut_seq(dst).len() ==>
                            crate::bytes_model::bytes_mut_seq(dst)[j]@ < 128;
                        crate::ascii::ascii_bytes_are_valid_utf8(
                            crate::bytes_model::bytes_mut_seq(dst)
                        );
                        creusot_std::std::string::valid_utf8(
                            crate::bytes_model::bytes_mut_seq(dst)
                        )
                    };

                    // Safety: the loop above maps each byte of maybe_lower.buf to a
                    // valid single-byte UTF-8 codepoint before copying it into dst.
                    // dst (and hence dst.freeze()) is thus valid UTF-8.
                    let buf = unsafe { ByteStr::from_utf8_unchecked(dst.freeze()) };

                    HeaderName {
                        inner: Repr::Custom(Custom(buf)),
                    }
                }
            }
        }
    }
}

#[doc(hidden)]
impl<'a> PartialEq<HdrName<'a>> for HeaderName {
    #[inline]
    #[cfg_attr(
        creusot,
        ensures(result == header_name_matches_hdr_name(self.deep_model(), other.deep_model()))
    )]
    fn eq(&self, other: &HdrName<'a>) -> bool {
        match self.inner {
            Repr::Standard(a) => match other.inner {
                Repr::Standard(b) => a == b,
                _ => false,
            },
            Repr::Custom(Custom(ref a)) => match other.inner {
                Repr::Custom(ref b) => {
                    if b.lower {
                        let lhs = a.as_bytes();
                        let equal = lhs == b.buf;

                        #[cfg(creusot)]
                        if equal {
                            proof_assert! { lhs@.len() == b.buf@.len() };
                            proof_assert! {
                                forall<i: Int> 0 <= i && i < lhs@.len() ==>
                                    lhs@[i]@ == b.buf@[i]@
                            };
                            proof_assert! { lhs@.ext_eq(b.buf@) };
                        }

                        equal
                    } else {
                        eq_ignore_ascii_case(a.as_bytes(), b.buf)
                    }
                }
                _ => false,
            },
        }
    }
}

// ===== Custom =====

impl Hash for Custom {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write(self.0.as_bytes())
    }
}

// ===== MaybeLower =====

impl<'a> Hash for MaybeLower<'a> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        if self.lower {
            hasher.write(self.buf);
        } else {
            for &b in self.buf {
                hasher.write(&[HEADER_CHARS[b as usize]]);
            }
        }
    }
}

// Assumes that the left hand side is already lower case
#[inline]
#[cfg_attr(creusot, ensures(result == (lower@.len() == s@.len()
    && forall<i: Int> 0 <= i && i < lower@.len()
        ==> lower@[i]@ == header_chars_byte(s@[i]))))]
fn eq_ignore_ascii_case(lower: &[u8], s: &[u8]) -> bool {
    if lower.len() != s.len() {
        return false;
    }

    let mut i = 0;
    let mut matches = true;
    #[cfg_attr(creusot, invariant(i@ <= lower@.len()))]
    #[cfg_attr(creusot, invariant(lower@.len() == s@.len()))]
    #[cfg_attr(creusot, invariant(matches == (forall<j: Int>
        0 <= j && j < i@ ==> lower@[j]@ == header_chars_byte(s@[j]))))]
    #[cfg_attr(creusot, variant(lower@.len() - i@))]
    while i < lower.len() {
        matches &= lower[i] == header_chars_byte_value(s[i]);
        i += 1;
    }

    matches
}

#[doc(hidden)]
pub const SCRATCH_BUF_SIZE: usize = 64;
const SCRATCH_BUF_OVERFLOW: usize = SCRATCH_BUF_SIZE + 1;

#[cfg(test)]
mod tests {
    use self::StandardHeader::Vary;
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug)]
    struct DerivedHdrName<'a> {
        inner: Repr<MaybeLower<'a>>,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct LegacyCustom(ByteStr);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct LegacyMaybeLower<'a> {
        buf: &'a [u8],
        lower: bool,
    }

    fn assert_hdr_name_debug_matches_derive(inner: Repr<MaybeLower<'_>>) {
        let actual = format!(
            "{:?}",
            HdrName {
                inner: inner.clone()
            }
        );
        let derived = format!("{:?}", DerivedHdrName { inner });
        let expected = derived.replacen("DerivedHdrName", "HdrName", 1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn hdr_name_debug_preserves_derived_format() {
        assert_hdr_name_debug_matches_derive(Repr::Standard(StandardHeader::Accept));
        assert_hdr_name_debug_matches_derive(Repr::Custom(MaybeLower {
            buf: b"custom-name",
            lower: true,
        }));
        assert_hdr_name_debug_matches_derive(Repr::Custom(MaybeLower {
            buf: b"X\"Name",
            lower: false,
        }));
    }

    #[test]
    fn manual_debug_implementations_match_the_original_derives() {
        for &(header, legacy_header) in TEST_HASH_HEADERS {
            assert_eq!(format!("{header:?}"), format!("{legacy_header:?}"));
            assert_eq!(
                format!("{:?}", Repr::<Custom>::Standard(header)),
                format!("{:?}", LegacyRepr::<LegacyCustom>::Standard(legacy_header)),
            );
        }

        let bytes = ByteStr::from_static("custom-name");
        assert_eq!(
            format!("{:?}", Custom(bytes.clone())),
            format!("{:?}", LegacyCustom(bytes.clone())).replacen("LegacyCustom", "Custom", 1),
        );
        assert_eq!(
            format!("{:?}", Repr::Custom(Custom(bytes.clone()))),
            format!("{:?}", LegacyRepr::Custom(LegacyCustom(bytes))).replacen(
                "LegacyCustom",
                "Custom",
                1
            ),
        );

        for &(buf, lower) in &[(b"custom-name" as &[u8], true), (b"X\"Name", false)] {
            assert_eq!(
                format!("{:?}", MaybeLower { buf, lower }),
                format!("{:?}", LegacyMaybeLower { buf, lower }).replacen(
                    "LegacyMaybeLower",
                    "MaybeLower",
                    1
                ),
            );
            assert_eq!(
                format!("{:?}", Repr::Custom(MaybeLower { buf, lower })),
                format!("{:?}", LegacyRepr::Custom(LegacyMaybeLower { buf, lower })).replacen(
                    "LegacyMaybeLower",
                    "MaybeLower",
                    1
                ),
            );
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum HashCall {
        Write(Vec<u8>),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        Usize(usize),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        Isize(isize),
    }

    #[derive(Default)]
    struct TraceHasher(Vec<HashCall>);

    impl Hasher for TraceHasher {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.push(HashCall::Write(bytes.to_vec()));
        }

        fn write_u8(&mut self, value: u8) {
            self.0.push(HashCall::U8(value));
        }

        fn write_u16(&mut self, value: u16) {
            self.0.push(HashCall::U16(value));
        }

        fn write_u32(&mut self, value: u32) {
            self.0.push(HashCall::U32(value));
        }

        fn write_u64(&mut self, value: u64) {
            self.0.push(HashCall::U64(value));
        }

        fn write_u128(&mut self, value: u128) {
            self.0.push(HashCall::U128(value));
        }

        fn write_usize(&mut self, value: usize) {
            self.0.push(HashCall::Usize(value));
        }

        fn write_i8(&mut self, value: i8) {
            self.0.push(HashCall::I8(value));
        }

        fn write_i16(&mut self, value: i16) {
            self.0.push(HashCall::I16(value));
        }

        fn write_i32(&mut self, value: i32) {
            self.0.push(HashCall::I32(value));
        }

        fn write_i64(&mut self, value: i64) {
            self.0.push(HashCall::I64(value));
        }

        fn write_i128(&mut self, value: i128) {
            self.0.push(HashCall::I128(value));
        }

        fn write_isize(&mut self, value: isize) {
            self.0.push(HashCall::Isize(value));
        }
    }

    fn hash_trace<T: Hash>(value: &T) -> Vec<HashCall> {
        let mut state = TraceHasher::default();
        value.hash(&mut state);
        state.0
    }

    #[test]
    fn numeric_header_character_helpers_match_original_tables() {
        for byte in 0u8..=u8::MAX {
            assert_eq!(header_chars_byte_value(byte), HEADER_CHARS[byte as usize]);
            assert_eq!(
                header_chars_h2_byte_value(byte),
                HEADER_CHARS_H2[byte as usize],
            );
        }

        assert_eq!(header_chars_byte_value(b'"'), 0);
        assert_eq!(header_chars_h2_byte_value(b'"'), b'"');
    }

    #[test]
    fn header_name_hash_matches_original_derived_trace() {
        for &(current, legacy) in TEST_HASH_HEADERS {
            assert_eq!(hash_trace(&current), hash_trace(&legacy));
            let current = HeaderName {
                inner: Repr::Standard(current),
            };
            let legacy = LegacyRepr::<Custom>::Standard(legacy);
            assert_eq!(hash_trace(&current), hash_trace(&legacy));
        }

        let custom = Custom(ByteStr::from_static("x-custom"));
        let current = HeaderName {
            inner: Repr::Custom(custom.clone()),
        };
        let legacy = LegacyRepr::Custom(custom);
        assert_eq!(hash_trace(&current), hash_trace(&legacy));

        let fixtures: &[(&[u8], bool)] = &[
            (b"A", false),
            (b"a", false),
            (b"\"", false),
            (b"[", false),
            (b"~", false),
            (&[0x80], false),
            (b"\"", true),
        ];
        for &(bytes, lower) in fixtures {
            let current = HdrName {
                inner: Repr::Custom(MaybeLower { buf: bytes, lower }),
            };
            let legacy = LegacyRepr::Custom(MaybeLower { buf: bytes, lower });
            assert_eq!(hash_trace(&current), hash_trace(&legacy));
        }

        for (byte, mapped) in [
            (b'A', b'a'),
            (b'a', b'a'),
            (b'"', 0),
            (b'[', 0),
            (b'~', b'~'),
            (0x80, 0),
        ] {
            let current = HdrName {
                inner: Repr::Custom(MaybeLower {
                    buf: std::slice::from_ref(&byte),
                    lower: false,
                }),
            };
            assert_eq!(
                hash_trace(&current),
                vec![HashCall::Isize(1), HashCall::Write(vec![mapped])],
            );
        }

        let quote = HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"\"",
                lower: true,
            }),
        };
        assert_eq!(
            hash_trace(&quote),
            vec![HashCall::Isize(1), HashCall::Write(vec![b'"'])],
        );
    }

    #[test]
    fn test_bounds() {
        fn check_bounds<T: Sync + Send>() {}
        check_bounds::<HeaderName>();
    }

    #[test]
    fn test_parse_invalid_headers() {
        for i in 0..128 {
            let hdr = vec![1u8; i];
            assert!(
                HeaderName::from_bytes(&hdr).is_err(),
                "{} invalid header chars did not fail",
                i
            );
        }
    }

    const ONE_TOO_LONG: &[u8] = &[b'a'; super::super::MAX_HEADER_NAME_LEN + 1];

    #[test]
    fn test_invalid_name_lengths() {
        assert!(
            HeaderName::from_bytes(&[]).is_err(),
            "zero-length header name is an error",
        );

        let long = &ONE_TOO_LONG[0..super::super::MAX_HEADER_NAME_LEN];

        let long_str = std::str::from_utf8(long).unwrap();
        assert_eq!(HeaderName::from_static(long_str), long_str); // shouldn't panic!

        assert!(
            HeaderName::from_bytes(long).is_ok(),
            "max header name length is ok",
        );
        assert!(
            HeaderName::from_bytes(ONE_TOO_LONG).is_err(),
            "longer than max header name length is an error",
        );
    }

    #[test]
    #[should_panic]
    fn test_static_invalid_name_lengths() {
        // Safety: ONE_TOO_LONG contains only the UTF-8 safe, single-byte codepoint b'a'.
        let _ = HeaderName::from_static(unsafe { std::str::from_utf8_unchecked(ONE_TOO_LONG) });
    }

    #[test]
    fn test_from_hdr_name() {
        use self::StandardHeader::Vary;

        let name = HeaderName::from(HdrName {
            inner: Repr::Standard(Vary),
        });

        assert_eq!(name.inner, Repr::Standard(Vary));

        let name = HeaderName::from(HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"hello-world",
                lower: true,
            }),
        });

        assert_eq!(
            name.inner,
            Repr::Custom(Custom(ByteStr::from_static("hello-world")))
        );

        let name = HeaderName::from(HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"Hello-World",
                lower: false,
            }),
        });

        assert_eq!(
            name.inner,
            Repr::Custom(Custom(ByteStr::from_static("hello-world")))
        );
    }

    #[test]
    fn test_eq_hdr_name() {
        use self::StandardHeader::Vary;

        let a = HeaderName {
            inner: Repr::Standard(Vary),
        };
        let b = HdrName {
            inner: Repr::Standard(Vary),
        };

        assert_eq!(a, b);

        let a = HeaderName {
            inner: Repr::Custom(Custom(ByteStr::from_static("vaary"))),
        };
        assert_ne!(a, b);

        let b = HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"vaary",
                lower: true,
            }),
        };

        assert_eq!(a, b);

        let b = HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"vaary",
                lower: false,
            }),
        };

        assert_eq!(a, b);

        let b = HdrName {
            inner: Repr::Custom(MaybeLower {
                buf: b"VAARY",
                lower: false,
            }),
        };

        assert_eq!(a, b);

        let a = HeaderName {
            inner: Repr::Standard(Vary),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_from_static_std() {
        let a = HeaderName {
            inner: Repr::Standard(Vary),
        };

        let b = HeaderName::from_static("vary");
        assert_eq!(a, b);

        let b = HeaderName::from_static("vaary");
        assert_ne!(a, b);
    }

    #[test]
    #[should_panic]
    fn test_from_static_std_uppercase() {
        HeaderName::from_static("Vary");
    }

    #[test]
    #[should_panic]
    fn test_from_static_std_symbol() {
        HeaderName::from_static("vary{}");
    }

    // MaybeLower { lower: true }
    #[test]
    fn test_from_static_custom_short() {
        let a = HeaderName {
            inner: Repr::Custom(Custom(ByteStr::from_static("customheader"))),
        };
        let b = HeaderName::from_static("customheader");
        assert_eq!(a, b);
    }

    #[test]
    #[should_panic]
    fn test_from_static_custom_short_uppercase() {
        HeaderName::from_static("custom header");
    }

    #[test]
    #[should_panic]
    fn test_from_static_custom_short_symbol() {
        HeaderName::from_static("CustomHeader");
    }

    // MaybeLower { lower: false }
    #[test]
    fn test_from_static_custom_long() {
        let a = HeaderName {
            inner: Repr::Custom(Custom(ByteStr::from_static(
                "longer-than-63--thisheaderislongerthansixtythreecharactersandthushandleddifferent",
            ))),
        };
        let b = HeaderName::from_static(
            "longer-than-63--thisheaderislongerthansixtythreecharactersandthushandleddifferent",
        );
        assert_eq!(a, b);
    }

    #[test]
    #[should_panic]
    fn test_from_static_custom_long_uppercase() {
        HeaderName::from_static(
            "Longer-Than-63--ThisHeaderIsLongerThanSixtyThreeCharactersAndThusHandledDifferent",
        );
    }

    #[test]
    #[should_panic]
    fn test_from_static_custom_long_symbol() {
        HeaderName::from_static(
            "longer-than-63--thisheader{}{}{}{}islongerthansixtythreecharactersandthushandleddifferent"
        );
    }

    #[test]
    fn test_from_static_custom_single_char() {
        let a = HeaderName {
            inner: Repr::Custom(Custom(ByteStr::from_static("a"))),
        };
        let b = HeaderName::from_static("a");
        assert_eq!(a, b);
    }

    #[test]
    #[should_panic]
    fn test_from_static_empty() {
        HeaderName::from_static("");
    }

    #[test]
    fn test_all_tokens() {
        HeaderName::from_static("!#$%&'*+-.^_`|~0123456789abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_from_lowercase() {
        HeaderName::from_lowercase(&[0; 10]).unwrap_err();
        HeaderName::from_lowercase(&[b'A'; 10]).unwrap_err();
        HeaderName::from_lowercase(&[0x1; 10]).unwrap_err();
        HeaderName::from_lowercase(&[0xFF; 10]).unwrap_err();
        //HeaderName::from_lowercase(&[0; 100]).unwrap_err();
        HeaderName::from_lowercase(&[b'A'; 100]).unwrap_err();
        HeaderName::from_lowercase(&[0x1; 100]).unwrap_err();
        HeaderName::from_lowercase(&[0xFF; 100]).unwrap_err();
    }
}
