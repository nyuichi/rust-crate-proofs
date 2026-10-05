//! The HTTP request method
//!
//! This module contains HTTP-method related structs and errors and such. The
//! main type of this module, `Method`, is also reexported at the root of the
//! crate as `http::Method` and is intended for import through that location
//! primarily.
//!
//! # Examples
//!
//! ```
//! use http::Method;
//!
//! assert_eq!(Method::GET, Method::from_bytes(b"GET").unwrap());
//! assert!(Method::GET.is_idempotent());
//! assert_eq!(Method::POST.as_str(), "POST");
//! ```

use self::extension::{AllocatedExtension, InlineExtension};
use self::Inner::*;

use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;
use std::{fmt, str};

#[allow(unused_imports)]
use creusot_std::prelude::{
    ensures, invariant, logic, pearlite, proof_assert, requires, variant, DeepModel, Int,
    Invariant, Seq,
};
#[cfg(creusot)]
use creusot_std::logic::OrdLogic;

/// Whether a byte is one of the ASCII characters allowed by HTTP's `tchar`.
#[cfg(creusot)]
#[logic(open(super))]
pub fn is_method_char_model(byte: u8) -> bool {
    pearlite! {
        (33 <= byte@ && byte@ <= 126)
            && byte@ != 34 && byte@ != 40 && byte@ != 41
            && byte@ != 44 && byte@ != 47 && byte@ != 58
            && byte@ != 59 && byte@ != 60 && byte@ != 61
            && byte@ != 62 && byte@ != 63 && byte@ != 64
            && byte@ != 91 && byte@ != 92 && byte@ != 93
            && byte@ != 123 && byte@ != 125
    }
}

/// Every byte in a sequence is an HTTP method character.
#[cfg(creusot)]
#[logic(open(super))]
pub fn all_method_chars_valid(bytes: Seq<u8>) -> bool {
    pearlite! {
        forall<i: Int> 0 <= i && i < bytes.len() ==> is_method_char_model(bytes[i])
    }
}

/// The shared ASCII character mapping preserves every byte value.
#[cfg(creusot)]
#[logic(open(crate))]
#[requires(forall<i: Int> 0 <= i && i < bytes.len() ==> bytes[i]@ < 128)]
#[ensures(forall<i: Int> 0 <= i && i < bytes.len()
    ==> crate::ascii::ascii_chars(bytes)[i]@ == bytes[i]@)]
#[variant(bytes.len())]
pub(crate) fn method_ascii_chars_preserve_byte_values(bytes: Seq<u8>) {
    if bytes.len() > 0 {
        let head = crate::ascii::ascii_char(bytes[0]);
        proof_assert! { bytes[0]@ < 128 };
        proof_assert! { head@ == bytes[0]@ };
        method_ascii_chars_preserve_byte_values(bytes.tail());
        proof_assert! {
            forall<i: Int> 0 <= i && i < bytes.len()
                ==> crate::ascii::ascii_chars(bytes)[i]@ == bytes[i]@
        };
    }
}

/// Transport lexicographic order between ASCII method bytes and their
/// character encoding. The helper is opaque to callers once its exact
/// comparator relation has been proved.
#[cfg(creusot)]
#[logic(opaque)]
#[requires(forall<i: Int> 0 <= i && i < left.len() ==> left[i]@ < 128)]
#[requires(forall<i: Int> 0 <= i && i < right.len() ==> right[i]@ < 128)]
#[ensures(result)]
#[ensures(result == (
    crate::ascii::ascii_chars(left).cmp_log(crate::ascii::ascii_chars(right))
        == left.cmp_log(right)
))]
pub(crate) fn method_ascii_cmp_preserved(left: Seq<u8>, right: Seq<u8>) -> bool {
    method_ascii_chars_preserve_byte_values(left);
    method_ascii_chars_preserve_byte_values(right);
    creusot_std::std::seq_ord::seq_cmp_char_u8_transport(
        crate::ascii::ascii_chars(left),
        crate::ascii::ascii_chars(right),
        left,
        right,
    );
    proof_assert! {
        crate::ascii::ascii_chars(left).cmp_log(crate::ascii::ascii_chars(right))
            == left.cmp_log(right)
    };
    true
}

/// Runtime counterpart of the RFC `tchar` byte predicate.
#[ensures(result == is_method_char_model(byte))]
const fn is_method_char(byte: u8) -> bool {
    (b'!' <= byte && byte <= b'~')
        && byte != b'"' && byte != b'(' && byte != b')'
        && byte != b',' && byte != b'/' && byte != b':'
        && byte != b';' && byte != b'<' && byte != b'='
        && byte != b'>' && byte != b'?' && byte != b'@'
        && byte != b'[' && byte != b'\\' && byte != b']'
        && byte != b'{' && byte != b'}'
}

/// Reinterpret ASCII method bytes as UTF-8 while preserving their exact text.
#[requires(forall<i: Int> 0 <= i && i < bytes@.len() ==> bytes@[i]@ < 128)]
#[cfg_attr(creusot, ensures(result@.to_bytes() == bytes@))]
fn method_str(bytes: &[u8]) -> &str {
    #[cfg(creusot)]
    proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
        creusot_std::std::string::valid_utf8(bytes@)
    };
    // Safety: the verified ASCII witness establishes valid UTF-8 for exactly
    // this byte sequence.
    unsafe { str::from_utf8_unchecked(bytes) }
}

/// Return a nonzero tag exactly when `src` spells one of the built-in methods.
/// This classifier does not construct a `Method`, so callers can separate the
/// byte-recognition proof from the representation invariant on `Method`.
#[cfg_attr(creusot, ensures(result@ <= 10))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag(src: &[u8]) -> u8 {
    match src.len() {
        3 => known_method_tag_3(src),
        4 => known_method_tag_4(src),
        5 => known_method_tag_5(src),
        6 => known_method_tag_6(src),
        7 => known_method_tag_7(src),
        _ => 0,
    }
}

#[cfg_attr(creusot, requires(src@.len() == 3))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag_3(src: &[u8]) -> u8 {
    if src[0] == 71 && src[1] == 69 && src[2] == 84 {
        2
    } else if src[0] == 80 && src[1] == 85 && src[2] == 84 {
        4
    } else {
        0
    }
}

#[cfg_attr(creusot, requires(src@.len() == 4))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag_4(src: &[u8]) -> u8 {
    if src[0] == 80 && src[1] == 79 && src[2] == 83 && src[3] == 84 {
        3
    } else if src[0] == 72 && src[1] == 69 && src[2] == 65 && src[3] == 68 {
        6
    } else {
        0
    }
}

#[cfg_attr(creusot, requires(src@.len() == 5))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag_5(src: &[u8]) -> u8 {
    if src[0] == 80 && src[1] == 65 && src[2] == 84 && src[3] == 67 && src[4] == 72 {
        9
    } else if src[0] == 84 && src[1] == 82 && src[2] == 65 && src[3] == 67 && src[4] == 69 {
        7
    } else if src[0] == 81 && src[1] == 85 && src[2] == 69 && src[3] == 82 && src[4] == 89 {
        10
    } else {
        0
    }
}

#[cfg_attr(creusot, requires(src@.len() == 6))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag_6(src: &[u8]) -> u8 {
    if src[0] == 68 && src[1] == 69 && src[2] == 76 && src[3] == 69 && src[4] == 84 && src[5] == 69 {
        5
    } else {
        0
    }
}

#[cfg_attr(creusot, requires(src@.len() == 7))]
#[cfg_attr(creusot, ensures((result@ == 0) == !is_known_method_text(src@)))]
#[cfg_attr(creusot, ensures(result@ > 0 ==>
    method_text_matches(src@, known_method_tag_text(result@))
        && all_method_chars_valid(src@)
))]
fn known_method_tag_7(src: &[u8]) -> u8 {
    if src[0] == 79 && src[1] == 80 && src[2] == 84 && src[3] == 73 && src[4] == 79 && src[5] == 78 && src[6] == 83 {
        1
    } else if src[0] == 67 && src[1] == 79 && src[2] == 78 && src[3] == 78 && src[4] == 69 && src[5] == 67 && src[6] == 84 {
        8
    } else {
        0
    }
}

/// The Request Method (VERB)
///
/// This type also contains constants for a number of common HTTP methods such
/// as GET, POST, etc.
///
/// Currently includes 8 variants representing the 8 methods defined in
/// [RFC 7230](https://tools.ietf.org/html/rfc7231#section-4.1), plus PATCH,
/// and an Extension variant for all extensions.
///
/// # Examples
///
/// ```
/// use http::Method;
///
/// assert_eq!(Method::GET, Method::from_bytes(b"GET").unwrap());
/// assert!(Method::GET.is_idempotent());
/// assert_eq!(Method::POST.as_str(), "POST");
/// ```
#[derive(Eq, Hash)]
pub struct Method(Inner);

// Keep the verifier's public model distinct from the private storage enum so
// the public DeepModel implementation does not expose that representation.
#[cfg(creusot)]
#[doc(hidden)]
// This is a logical model type; deriving runtime Debug would impose a Debug
// implementation on the verifier's logical Seq fields.
#[allow(missing_debug_implementations)]
#[derive(Copy, Clone)]
pub enum MethodModel {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Query,
    ExtensionInline(Seq<u8>, Int),
    ExtensionAllocated(Seq<u8>),
}

#[cfg(creusot)]
#[logic(open)]
pub fn method_model_text(method: MethodModel) -> Seq<u8> {
    pearlite! {
        match method {
            MethodModel::Options => seq![79u8, 80u8, 84u8, 73u8, 79u8, 78u8, 83u8],
            MethodModel::Get => seq![71u8, 69u8, 84u8],
            MethodModel::Post => seq![80u8, 79u8, 83u8, 84u8],
            MethodModel::Put => seq![80u8, 85u8, 84u8],
            MethodModel::Delete => seq![68u8, 69u8, 76u8, 69u8, 84u8, 69u8],
            MethodModel::Head => seq![72u8, 69u8, 65u8, 68u8],
            MethodModel::Trace => seq![84u8, 82u8, 65u8, 67u8, 69u8],
            MethodModel::Connect => seq![67u8, 79u8, 78u8, 78u8, 69u8, 67u8, 84u8],
            MethodModel::Patch => seq![80u8, 65u8, 84u8, 67u8, 72u8],
            MethodModel::Query => seq![81u8, 85u8, 69u8, 82u8, 89u8],
            MethodModel::ExtensionInline(data, len) => data.subsequence(0, len),
            MethodModel::ExtensionAllocated(data) => data,
        }
    }
}

/// Exact elementwise equality for two byte sequences.
#[cfg(creusot)]
#[logic(open)]
pub fn method_text_matches(left: Seq<u8>, right: Seq<u8>) -> bool {
    pearlite! {
        left.len() == right.len()
            && forall<i: Int> 0 <= i && i < left.len() ==> left[i] == right[i]
    }
}

/// Exact byte model for each of the ten built-in methods, indexed by their
/// private classifier tag (1 through 10). Tag zero denotes an extension.
#[cfg(creusot)]
#[logic(open)]
pub fn known_method_tag_text(tag: Int) -> Seq<u8> {
    pearlite! {
        if tag == 1 { method_model_text(MethodModel::Options) }
        else if tag == 2 { method_model_text(MethodModel::Get) }
        else if tag == 3 { method_model_text(MethodModel::Post) }
        else if tag == 4 { method_model_text(MethodModel::Put) }
        else if tag == 5 { method_model_text(MethodModel::Delete) }
        else if tag == 6 { method_model_text(MethodModel::Head) }
        else if tag == 7 { method_model_text(MethodModel::Trace) }
        else if tag == 8 { method_model_text(MethodModel::Connect) }
        else if tag == 9 { method_model_text(MethodModel::Patch) }
        else if tag == 10 { method_model_text(MethodModel::Query) }
        else { Seq::empty() }
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn is_known_method_text(bytes: Seq<u8>) -> bool {
    pearlite! {
        if bytes.len() == 3 {
            method_text_matches(bytes, method_model_text(MethodModel::Get))
                || method_text_matches(bytes, method_model_text(MethodModel::Put))
        } else if bytes.len() == 4 {
            method_text_matches(bytes, method_model_text(MethodModel::Post))
                || method_text_matches(bytes, method_model_text(MethodModel::Head))
        } else if bytes.len() == 5 {
            method_text_matches(bytes, method_model_text(MethodModel::Patch))
                || method_text_matches(bytes, method_model_text(MethodModel::Trace))
                || method_text_matches(bytes, method_model_text(MethodModel::Query))
        } else if bytes.len() == 6 {
            method_text_matches(bytes, method_model_text(MethodModel::Delete))
        } else if bytes.len() == 7 {
            method_text_matches(bytes, method_model_text(MethodModel::Options))
                || method_text_matches(bytes, method_model_text(MethodModel::Connect))
        } else {
            false
        }
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn method_model_is_safe(method: MethodModel) -> bool {
    pearlite! {
        match method {
            MethodModel::Get | MethodModel::Head | MethodModel::Options
                | MethodModel::Trace | MethodModel::Query => true,
            _ => false,
        }
    }
}

#[cfg(creusot)]
#[logic(open)]
pub fn method_model_is_idempotent(method: MethodModel) -> bool {
    pearlite! {
        match method {
            MethodModel::Get | MethodModel::Head | MethodModel::Options
                | MethodModel::Trace | MethodModel::Query
                | MethodModel::Put | MethodModel::Delete => true,
            _ => false,
        }
    }
}

/// Canonical representation conditions for a reachable method model.
#[cfg(creusot)]
#[logic(open)]
pub fn method_model_is_canonical(method: MethodModel) -> bool {
    pearlite! {
        match method {
            MethodModel::ExtensionInline(data, len) =>
                data.len() == InlineExtension::MAX@
                    && 0 < len
                    && len <= InlineExtension::MAX@
                    && all_method_chars_valid(data.subsequence(0, len))
                    && (forall<i: Int> len <= i && i < data.len() ==> data[i] == 0u8)
                    && !is_known_method_text(data.subsequence(0, len)),
            MethodModel::ExtensionAllocated(data) =>
                data.len() > InlineExtension::MAX@
                    && all_method_chars_valid(data)
                    && !is_known_method_text(data),
            _ => true,
        }
    }
}

/// Equal canonical inline text has equal length and identical zero-padded
/// storage. This is the representation step used by structural Eq.
#[cfg(creusot)]
#[logic(open)]
#[requires(data1.len() == InlineExtension::MAX@)]
#[requires(data2.len() == InlineExtension::MAX@)]
#[requires(0 < len1 && len1 <= InlineExtension::MAX@)]
#[requires(0 < len2 && len2 <= InlineExtension::MAX@)]
#[requires(forall<i: Int> len1 <= i && i < data1.len() ==> data1[i] == 0u8)]
#[requires(forall<i: Int> len2 <= i && i < data2.len() ==> data2[i] == 0u8)]
#[ensures(
    (data1 == data2 && len1 == len2)
        == (data1.subsequence(0, len1) == data2.subsequence(0, len2))
)]
pub fn equal_canonical_inline_storage(
    data1: Seq<u8>,
    len1: Int,
    data2: Seq<u8>,
    len2: Int,
) {
    proof_assert! {
        (data1 == data2 && len1 == len2)
            == (data1.subsequence(0, len1) == data2.subsequence(0, len2))
    };
}

/// On reachable Method models, exact text is an injective observer of the
/// structural representation. This preserves structural Eq semantics.
#[cfg(creusot)]
#[logic(open)]
#[requires(method_model_is_canonical(left))]
#[requires(method_model_is_canonical(right))]
#[ensures((left == right) == (method_model_text(left) == method_model_text(right)))]
fn canonical_method_text_is_injective(left: MethodModel, right: MethodModel) {
    match (left, right) {
        (
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
        ) => {
            proof_assert! {
                (left == right)
                    == (method_model_text(left) == method_model_text(right))
            };
        }
        (
            MethodModel::ExtensionInline(data1, len1),
            MethodModel::ExtensionInline(data2, len2),
        ) => {
            equal_canonical_inline_storage(data1, len1, data2, len2);
            proof_assert! {
                (MethodModel::ExtensionInline(data1, len1)
                    == MethodModel::ExtensionInline(data2, len2))
                    == (data1.subsequence(0, len1) == data2.subsequence(0, len2))
            };
        }
        (
            MethodModel::ExtensionAllocated(data1),
            MethodModel::ExtensionAllocated(data2),
        ) => {
            proof_assert! { (data1 == data2) == (data1 == data2) };
        }
        (
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
            MethodModel::ExtensionInline(data, len),
        ) => {
            proof_assert! { is_known_method_text(method_model_text(left)) };
            proof_assert! { !is_known_method_text(data.subsequence(0, len)) };
            proof_assert! {
                (left == MethodModel::ExtensionInline(data, len))
                    == (method_model_text(left) == data.subsequence(0, len))
            };
        }
        (
            MethodModel::ExtensionInline(data, len),
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
        ) => {
            proof_assert! { is_known_method_text(method_model_text(right)) };
            proof_assert! { !is_known_method_text(data.subsequence(0, len)) };
            proof_assert! {
                (MethodModel::ExtensionInline(data, len) == right)
                    == (data.subsequence(0, len) == method_model_text(right))
            };
        }
        (
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
            MethodModel::ExtensionAllocated(data),
        ) => {
            proof_assert! { is_known_method_text(method_model_text(left)) };
            proof_assert! { !is_known_method_text(data) };
            proof_assert! {
                (left == MethodModel::ExtensionAllocated(data))
                    == (method_model_text(left) == data)
            };
        }
        (
            MethodModel::ExtensionAllocated(data),
            MethodModel::Options
            | MethodModel::Get
            | MethodModel::Post
            | MethodModel::Put
            | MethodModel::Delete
            | MethodModel::Head
            | MethodModel::Trace
            | MethodModel::Connect
            | MethodModel::Patch
            | MethodModel::Query,
        ) => {
            proof_assert! { is_known_method_text(method_model_text(right)) };
            proof_assert! { !is_known_method_text(data) };
            proof_assert! {
                (MethodModel::ExtensionAllocated(data) == right)
                    == (data == method_model_text(right))
            };
        }
        (
            MethodModel::ExtensionInline(data, len),
            MethodModel::ExtensionAllocated(allocated),
        )
        | (
            MethodModel::ExtensionAllocated(allocated),
            MethodModel::ExtensionInline(data, len),
        ) => {
            proof_assert! { len != allocated.len() };
            proof_assert! {
                (data.subsequence(0, len) == allocated) == false
            };
        }
    }
}

/// Every canonical method's public text consists of ASCII bytes.
#[cfg(creusot)]
#[logic(open)]
#[requires(method_model_is_canonical(method))]
#[ensures(forall<i: Int> 0 <= i && i < method_model_text(method).len()
    ==> method_model_text(method)[i]@ < 128)]
fn canonical_method_text_is_ascii(method: MethodModel) {
    match method {
        MethodModel::Options
        | MethodModel::Get
        | MethodModel::Post
        | MethodModel::Put
        | MethodModel::Delete
        | MethodModel::Head
        | MethodModel::Trace
        | MethodModel::Connect
        | MethodModel::Patch
        | MethodModel::Query => {
            proof_assert! {
                forall<i: Int> 0 <= i && i < method_model_text(method).len()
                    ==> method_model_text(method)[i]@ < 128
            };
        }
        MethodModel::ExtensionInline(data, len) => {
            proof_assert! {
                forall<i: Int> 0 <= i && i < data.subsequence(0, len).len()
                    ==> data.subsequence(0, len)[i]@ < 128
            };
        }
        MethodModel::ExtensionAllocated(data) => {
            proof_assert! {
                forall<i: Int> 0 <= i && i < data.len() ==> data[i]@ < 128
            };
        }
    }
}

#[cfg(creusot)]
#[doc(hidden)]
#[logic]
pub fn method_repr(value: &Method) -> MethodModel {
        <&Inner as DeepModel>::deep_model(&value.0)
}

#[cfg(creusot)]
impl DeepModel for Inner {
    type DeepModelTy = MethodModel;

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        match self {
            Options => MethodModel::Options,
            Get => MethodModel::Get,
            Post => MethodModel::Post,
            Put => MethodModel::Put,
            Delete => MethodModel::Delete,
            Head => MethodModel::Head,
            Trace => MethodModel::Trace,
            Connect => MethodModel::Connect,
            Patch => MethodModel::Patch,
            Query => MethodModel::Query,
            ExtensionInline(value) => {
                let model = value.deep_model();
                MethodModel::ExtensionInline(model.0, model.1)
            }
            ExtensionAllocated(value) => MethodModel::ExtensionAllocated(value.deep_model()),
        }
    }
}

#[cfg(creusot)]
impl DeepModel for Method {
    type DeepModelTy = Seq<u8>;

    #[logic(open(self))]
    fn deep_model(self) -> Self::DeepModelTy {
        method_model_text(self.0.deep_model())
    }
}

/// A possible error value when converting `Method` from bytes.
pub struct InvalidMethod {
    _priv: (),
}

#[derive(Clone, Eq)]
#[cfg_attr(creusot, derive(creusot_std::std::cmp::PartialEq))]
#[cfg_attr(not(creusot), derive(PartialEq))]
enum Inner {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Query,
    // If the extension is short enough, store it inline
    ExtensionInline(InlineExtension),
    // Otherwise, allocate it
    ExtensionAllocated(AllocatedExtension),
}

// Match the pinned derive(Hash) callback sequence: hash the enum discriminant
// as an isize, then hash the payload for the two data-carrying variants. The
// implicit discriminants follow declaration order, from Options = 0 through
// ExtensionAllocated = 11.
impl std::hash::Hash for Inner {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let discriminant = match self {
            Options => 0isize,
            Get => 1isize,
            Post => 2isize,
            Put => 3isize,
            Delete => 4isize,
            Head => 5isize,
            Trace => 6isize,
            Connect => 7isize,
            Patch => 8isize,
            Query => 9isize,
            ExtensionInline(_) => 10isize,
            ExtensionAllocated(_) => 11isize,
        };
        std::hash::Hash::hash(&discriminant, state);

        match self {
            ExtensionInline(value) => std::hash::Hash::hash(value, state),
            ExtensionAllocated(value) => std::hash::Hash::hash(value, state),
            _ => {}
        }
    }
}

// Keep Clone's upstream value semantics while exposing its exact effect to
// verification. The inline representation is copied field by field; allocated
// extensions use their exact content-copy implementation below.
impl Clone for Method {
    #[cfg_attr(creusot, requires(self.invariant()))]
    #[cfg_attr(creusot, ensures(result.deep_model() == self.deep_model()))]
    #[cfg_attr(creusot, ensures(result.invariant()))]
    fn clone(&self) -> Self {
        let inner = match &self.0 {
            Options => Options,
            Get => Get,
            Post => Post,
            Put => Put,
            Delete => Delete,
            Head => Head,
            Trace => Trace,
            Connect => Connect,
            Patch => Patch,
            Query => Query,
            ExtensionInline(value) => ExtensionInline(InlineExtension(value.0, value.1)),
            ExtensionAllocated(value) => ExtensionAllocated(value.clone()),
        };
        Method(inner)
    }
}

impl PartialEq for Method {
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
    fn eq(&self, other: &Self) -> bool {
        let result = self.0 == other.0;
        #[cfg(creusot)]
        proof_assert! {
            canonical_method_text_is_injective(method_repr(self), method_repr(other));
            result == (self.deep_model() == other.deep_model())
        };
        result
    }
}

#[cfg(creusot)]
impl Invariant for Method {
    #[logic(open)]
    fn invariant(self) -> bool {
        method_model_is_canonical(method_repr(&self))
    }
}

impl Method {
    /// GET
    pub const GET: Method = Method(Get);

    /// POST
    pub const POST: Method = Method(Post);

    /// PUT
    pub const PUT: Method = Method(Put);

    /// DELETE
    pub const DELETE: Method = Method(Delete);

    /// HEAD
    pub const HEAD: Method = Method(Head);

    /// OPTIONS
    pub const OPTIONS: Method = Method(Options);

    /// CONNECT
    pub const CONNECT: Method = Method(Connect);

    /// PATCH
    pub const PATCH: Method = Method(Patch);

    /// TRACE
    pub const TRACE: Method = Method(Trace);

    /// QUERY
    pub const QUERY: Method = Method(Query);

    /// Converts a slice of bytes to an HTTP method.
    #[cfg_attr(creusot, ensures(match result {
        Ok(method) => src@.len() > 0
            && all_method_chars_valid(src@)
            && method_text_matches(method.deep_model(), src@)
            && method.invariant(),
        Err(_) => src@.len() == 0 || !all_method_chars_valid(src@),
    }))]
    pub fn from_bytes(src: &[u8]) -> Result<Method, InvalidMethod> {
        let tag = known_method_tag(src);
        if tag != 0 {
            Ok(Method::from_known_tag(tag))
        } else if src.is_empty() {
            Err(InvalidMethod::new())
        } else if src.len() <= InlineExtension::MAX {
            Method::extension_inline(src)
        } else {
            let allocated = AllocatedExtension::new(src)?;
            Ok(Method(ExtensionAllocated(allocated)))
        }
    }

    /// Materialize the built-in variant selected by `known_method_tag`.
    #[requires(1 <= tag@ && tag@ <= 10)]
    #[cfg_attr(creusot, ensures(method_text_matches(result.deep_model(), known_method_tag_text(tag@))))]
    #[cfg_attr(creusot, ensures(result.invariant()))]
    fn from_known_tag(tag: u8) -> Method {
        match tag {
            1 => Method(Options),
            2 => Method(Get),
            3 => Method(Post),
            4 => Method(Put),
            5 => Method(Delete),
            6 => Method(Head),
            7 => Method(Trace),
            8 => Method(Connect),
            9 => Method(Patch),
            10 => Method(Query),
            _ => unreachable!(),
        }
    }

    #[requires(src@.len() <= InlineExtension::MAX@)]
    #[requires(src@.len() > 0)]
    #[cfg_attr(creusot, requires(!is_known_method_text(src@)))]
    #[cfg_attr(creusot, ensures(match result {
        Ok(method) => all_method_chars_valid(src@)
            && method_text_matches(method.deep_model(), src@)
            && method.invariant(),
        Err(_) => !all_method_chars_valid(src@),
    }))]
    fn extension_inline(src: &[u8]) -> Result<Method, InvalidMethod> {
        let inline = InlineExtension::new(src)?;

        Ok(Method(ExtensionInline(inline)))
    }

    /// Whether a method is considered "safe", meaning the request is
    /// essentially read-only.
    ///
    /// See [the spec](https://tools.ietf.org/html/rfc7231#section-4.2.1)
    /// for more words.
    #[cfg_attr(creusot, ensures(result == method_model_is_safe(method_repr(self))))]
    pub fn is_safe(&self) -> bool {
        matches!(self.0, Get | Head | Options | Trace | Query)
    }

    /// Whether a method is considered "idempotent", meaning the request has
    /// the same result if executed multiple times.
    ///
    /// See [the spec](https://tools.ietf.org/html/rfc7231#section-4.2.2) for
    /// more words.
    #[cfg_attr(creusot, ensures(
        result == method_model_is_idempotent(method_repr(self))
    ))]
    pub fn is_idempotent(&self) -> bool {
        match self.0 {
            Put | Delete => true,
            _ => self.is_safe(),
        }
    }

    /// Return a &str representation of the HTTP method
    #[inline]
    #[cfg_attr(creusot, ensures(
        method_text_matches(result@.to_bytes(), self.deep_model())
    ))]
    pub fn as_str(&self) -> &str {
        match self.0 {
            Options => method_str(&[79, 80, 84, 73, 79, 78, 83]),
            Get => method_str(&[71, 69, 84]),
            Post => method_str(&[80, 79, 83, 84]),
            Put => method_str(&[80, 85, 84]),
            Delete => method_str(&[68, 69, 76, 69, 84, 69]),
            Head => method_str(&[72, 69, 65, 68]),
            Trace => method_str(&[84, 82, 65, 67, 69]),
            Connect => method_str(&[67, 79, 78, 78, 69, 67, 84]),
            Patch => method_str(&[80, 65, 84, 67, 72]),
            Query => method_str(&[81, 85, 69, 82, 89]),
            ExtensionInline(ref inline) => inline.as_str(),
            ExtensionAllocated(ref allocated) => allocated.as_str(),
        }
    }
}

impl AsRef<str> for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result@.to_bytes() == self.deep_model()))]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Ord for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(
        result == self.deep_model().cmp_log(other.deep_model())
    ))]
    fn cmp(&self, other: &Method) -> std::cmp::Ordering {
        let left = self.as_ref();
        let right = other.as_ref();
        let result = left.cmp(right);
        #[cfg(creusot)]
        {
            proof_assert! {
                canonical_method_text_is_ascii(method_repr(self));
                forall<i: Int> 0 <= i
                    && i < method_model_text(method_repr(self)).len()
                    ==> method_model_text(method_repr(self))[i]@ < 128
            };
            proof_assert! {
                canonical_method_text_is_ascii(method_repr(other));
                forall<i: Int> 0 <= i
                    && i < method_model_text(method_repr(other)).len()
                    ==> method_model_text(method_repr(other))[i]@ < 128
            };
            proof_assert! {
                crate::ascii::ascii_bytes_are_valid_utf8(
                    method_model_text(method_repr(self))
                );
                crate::ascii::ascii_bytes_are_valid_utf8(
                    method_model_text(method_repr(other))
                );
                creusot_std::std::string::injective_to_bytes();
                true
            };
            proof_assert! {
                left@ == crate::ascii::ascii_chars(method_model_text(method_repr(self)))
            };
            proof_assert! {
                right@ == crate::ascii::ascii_chars(method_model_text(method_repr(other)))
            };
            proof_assert! {
                method_ascii_cmp_preserved(
                    method_model_text(method_repr(self)),
                    method_model_text(method_repr(other)),
                )
            };
            proof_assert! {
                result == method_model_text(method_repr(self))
                    .cmp_log(method_model_text(method_repr(other)))
            };
        }
        result
    }
}

impl PartialOrd for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(
        result == Some(self.deep_model().cmp_log(other.deep_model()))
    ))]
    fn partial_cmp(&self, other: &Method) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<&Method> for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
    fn eq(&self, other: &&Method) -> bool {
        self == *other
    }
}

impl PartialEq<Method> for &Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
    fn eq(&self, other: &Method) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == other@.to_bytes())))]
    fn eq(&self, other: &str) -> bool {
        let result = self.as_ref() == other;
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::string::injective_to_bytes();
            result == (self.deep_model() == other@.to_bytes())
        };
        result
    }
}

impl PartialEq<Method> for str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self@.to_bytes() == other.deep_model())))]
    fn eq(&self, other: &Method) -> bool {
        let result = self == other.as_ref();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::string::injective_to_bytes();
            result == (self@.to_bytes() == other.deep_model())
        };
        result
    }
}

impl PartialEq<&str> for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.deep_model() == (**other)@.to_bytes())))]
    fn eq(&self, other: &&str) -> bool {
        let result = self.as_ref() == *other;
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::string::injective_to_bytes();
            result == (self.deep_model() == (**other)@.to_bytes())
        };
        result
    }
}

impl PartialEq<Method> for &str {
    #[inline]
    #[cfg_attr(creusot, ensures(result == ((**self)@.to_bytes() == other.deep_model())))]
    fn eq(&self, other: &Method) -> bool {
        let result = *self == other.as_ref();
        #[cfg(creusot)]
        proof_assert! {
            creusot_std::std::string::injective_to_bytes();
            result == ((**self)@.to_bytes() == other.deep_model())
        };
        result
    }
}

impl fmt::Debug for Method {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^f).deep_model() == f.deep_model().concat(
            self.deep_model().map(|byte: u8| byte@)
        ),
        Err(_) => creusot_std::std::fmt::formatter_extends(
            f.deep_model(), (^f).deep_model()
        ),
    }))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl fmt::Display for Method {
    #[cfg_attr(creusot, ensures(match result {
        Ok(_) => (^fmt).deep_model() == fmt.deep_model().concat(
            self.deep_model().map(|byte: u8| byte@)
        ),
        Err(_) => creusot_std::std::fmt::formatter_extends(
            fmt.deep_model(), (^fmt).deep_model()
        ),
    }))]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_ref())
    }
}

impl Default for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Get)
            && result.invariant()
    ))]
    fn default() -> Method {
        Method::GET
    }
}

impl From<&Method> for Method {
    #[inline]
    #[cfg_attr(creusot, ensures(result.deep_model() == t.deep_model()))]
    #[cfg_attr(creusot, ensures(result.invariant()))]
    fn from(t: &Method) -> Self {
        t.clone()
    }
}

impl TryFrom<&[u8]> for Method {
    type Error = InvalidMethod;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(method) => method.deep_model() == t@ && method.invariant(),
        Err(_) => t@.len() == 0 || !all_method_chars_valid(t@),
    }))]
    fn try_from(t: &[u8]) -> Result<Self, Self::Error> {
        Method::from_bytes(t)
    }
}

impl TryFrom<&str> for Method {
    type Error = InvalidMethod;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(method) => method.deep_model() == t@.to_bytes() && method.invariant(),
        Err(_) => t@.to_bytes().len() == 0
            || !all_method_chars_valid(t@.to_bytes()),
    }))]
    fn try_from(t: &str) -> Result<Self, Self::Error> {
        Method::from_bytes(t.as_bytes())
    }
}

impl FromStr for Method {
    type Err = InvalidMethod;

    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Ok(method) => method.deep_model() == t@.to_bytes() && method.invariant(),
        Err(_) => t@.to_bytes().len() == 0
            || !all_method_chars_valid(t@.to_bytes()),
    }))]
    fn from_str(t: &str) -> Result<Self, Self::Err> {
        Method::from_bytes(t.as_bytes())
    }
}

impl InvalidMethod {
    fn new() -> InvalidMethod {
        InvalidMethod { _priv: () }
    }
}

impl fmt::Debug for InvalidMethod {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidMethod")
            // skip _priv noise
            .finish()
    }
}

impl fmt::Display for InvalidMethod {
    #[cfg_attr(creusot, ensures(
        creusot_std::std::fmt::formatter_extends(f.deep_model(), (^f).deep_model())
    ))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid HTTP method")
    }
}

impl Error for InvalidMethod {}

mod extension {
    use super::{is_method_char, InvalidMethod};
    #[cfg(creusot)]
    use super::{all_method_chars_valid, is_method_char_model};
    #[allow(unused_imports)]
    use creusot_std::prelude::{
        ensures, invariant, logic, pearlite, proof_assert, requires, DeepModel, Int, Invariant,
        Seq, View,
    };
    use std::str;

    #[derive(Clone, Eq, Hash)]
    pub struct InlineExtension(pub [u8; InlineExtension::MAX], pub u8);

    // Keep the derived runtime comparison order (the complete fixed array,
    // then its length), while exposing the byte-view equality needed to
    // connect the array's integer deep model to InlineExtension's byte model.
    impl PartialEq for InlineExtension {
        #[cfg_attr(creusot, ensures(result == (self.deep_model() == other.deep_model())))]
        fn eq(&self, other: &Self) -> bool {
            let arrays_equal = self.0 == other.0;
            let result = arrays_equal && self.1 == other.1;

            #[cfg(creusot)]
            if arrays_equal {
                proof_assert! { self.0@.len() == other.0@.len() };
                proof_assert! {
                    forall<i: Int> 0 <= i && i < self.0@.len() ==>
                        self.0@[i]@ == other.0@[i]@
                };
                proof_assert! { self.0@.ext_eq(other.0@) };
            }

            result
        }
    }

    #[derive(Eq, Hash)]
    #[cfg_attr(creusot, derive(creusot_std::std::cmp::PartialEq))]
    #[cfg_attr(not(creusot), derive(PartialEq))]
    pub struct AllocatedExtension(pub Box<[u8]>);

    impl Invariant for InlineExtension {
        #[logic(open)]
        fn invariant(self) -> bool {
            pearlite! {
                self.1@ <= InlineExtension::MAX@
                    && all_method_chars_valid(self.0@.subsequence(0, self.1@))
            }
        }
    }

    impl Invariant for AllocatedExtension {
        #[logic(open)]
        fn invariant(self) -> bool {
            pearlite! { all_method_chars_valid(self.0@) }
        }
    }

    #[cfg(creusot)]
    impl DeepModel for InlineExtension {
        type DeepModelTy = (Seq<u8>, Int);

        #[logic(open(super))]
        fn deep_model(self) -> Self::DeepModelTy {
            pearlite! { (self.0@, self.1@) }
        }
    }

    #[cfg(creusot)]
    impl DeepModel for AllocatedExtension {
        type DeepModelTy = Seq<u8>;

        #[logic(open(super))]
        fn deep_model(self) -> Self::DeepModelTy {
            pearlite! { self.0@ }
        }
    }

    impl InlineExtension {
        // Method::from_bytes() assumes this is at least 7
        pub const MAX: usize = 15;

        #[requires(src@.len() <= InlineExtension::MAX@)]
        #[cfg_attr(creusot, ensures(match result {
            Ok(value) => all_method_chars_valid(src@)
                && value.0@.subsequence(0, value.1@) == src@
                && value.1@ == src@.len()
                && forall<i: Int> value.1@ <= i && i < value.0@.len()
                    ==> value.0@[i] == 0u8,
            Err(_) => !all_method_chars_valid(src@),
        }))]
        pub fn new(src: &[u8]) -> Result<InlineExtension, InvalidMethod> {
            let mut data: [u8; InlineExtension::MAX] = [0u8; InlineExtension::MAX];

            write_checked(src, &mut data)?;

            // Invariant: write_checked ensures that the first src.len() bytes
            // of data are valid UTF-8.
            Ok(InlineExtension(data, src.len() as u8))
        }

        #[cfg_attr(creusot, ensures(
            result@.to_bytes() == self.0@.subsequence(0, self.1@)
        ))]
        pub fn as_str(&self) -> &str {
            let data = self.0.as_slice();
            let len = self.1 as usize;
            let bytes = &data[..len];
            super::method_str(bytes)
        }
    }

    impl AllocatedExtension {
        #[cfg_attr(creusot, ensures(match result {
            Ok(value) => all_method_chars_valid(src@) && value.0@ == src@,
            Err(_) => !all_method_chars_valid(src@),
        }))]
        pub fn new(src: &[u8]) -> Result<AllocatedExtension, InvalidMethod> {
            let mut data: Vec<u8> = vec![0; src.len()];

            write_checked(src, &mut data)?;

            // Invariant: data is exactly src.len() long and write_checked
            // ensures that the first src.len() bytes of data are valid UTF-8.
            Ok(AllocatedExtension(data.into_boxed_slice()))
        }

        #[cfg_attr(creusot, ensures(result@.to_bytes() == self.0@))]
        pub fn as_str(&self) -> &str {
            let bytes = &self.0;
            super::method_str(bytes)
        }
    }

    impl Clone for AllocatedExtension {
        #[requires(self.invariant())]
        #[ensures(result.0@ == self.0@)]
        #[ensures(result.invariant())]
        fn clone(&self) -> Self {
            let src: &[u8] = self.0.as_ref();
            let mut copy = Vec::with_capacity(src.len());
            let mut i = 0;

            #[invariant(i@ <= src@.len())]
            #[invariant(copy@.len() == i@)]
            #[invariant(forall<j: Int> 0 <= j && j < i@ ==> copy@[j] == src@[j])]
            #[variant(src@.len() - i@)]
            while i < src.len() {
                copy.push(src[i]);
                i += 1;
            }

            AllocatedExtension(copy.into_boxed_slice())
        }
    }

    // From the RFC 9110 HTTP Semantics, section 9.1, the HTTP method is case-sensitive and can
    // contain the following characters:
    //
    // ```
    // method = token
    // token = 1*tchar
    // tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." /
    //     "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
    // ```
    //
    // https://datatracker.ietf.org/doc/html/rfc9110#section-9.1
    //
    // Note that this definition means that any &[u8] that consists solely of valid
    // characters is also valid UTF-8 because the valid method characters are a
    // subset of the valid 1 byte UTF-8 encoding.
    // write_checked ensures (among other things) that the first src.len() bytes
    // of dst are valid UTF-8. On invalid input it leaves dst unchanged; callers
    // discard the private buffer on error, so this validation-first ordering is
    // observationally equivalent to the old partial-write loop.
    #[requires(src@.len() <= dst@.len())]
    #[ensures(match result {
        Ok(()) => all_method_chars_valid(src@)
            && (^dst)@ == src@.concat(dst@.subsequence(src@.len(), dst@.len())),
        Err(_) => !all_method_chars_valid(src@) && (^dst)@ == dst@,
    })]
    fn write_checked(src: &[u8], dst: &mut [u8]) -> Result<(), InvalidMethod> {
        let mut i = 0;

        #[invariant(i@ <= src@.len())]
        #[invariant(src@.len() <= dst@.len())]
        #[invariant(forall<j: Int> 0 <= j && j < i@ ==>
            is_method_char_model(src@[j]))]
        #[variant(src@.len() - i@)]
        while i < src.len() {
            if !is_method_char(src[i]) {
                return Err(InvalidMethod::new());
            }
            i += 1;
        }

        dst[..src.len()].copy_from_slice(src);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_method_eq() {
        assert_eq!(Method::GET, Method::GET);
        assert_eq!(Method::GET, "GET");
        assert_eq!(&Method::GET, "GET");

        assert_eq!("GET", Method::GET);
        assert_eq!("GET", &Method::GET);

        assert_eq!(&Method::GET, Method::GET);
        assert_eq!(Method::GET, &Method::GET);
    }

    #[test]
    fn test_invalid_method() {
        assert!(Method::from_str("").is_err());
        assert!(Method::from_bytes(b"").is_err());
        assert!(Method::from_bytes(&[0xC0]).is_err()); // invalid utf-8
        assert!(Method::from_bytes(&[0x10]).is_err()); // invalid method characters
    }

    #[test]
    fn test_is_idempotent() {
        assert!(Method::OPTIONS.is_idempotent());
        assert!(Method::GET.is_idempotent());
        assert!(Method::PUT.is_idempotent());
        assert!(Method::DELETE.is_idempotent());
        assert!(Method::HEAD.is_idempotent());
        assert!(Method::TRACE.is_idempotent());
        assert!(Method::QUERY.is_idempotent());

        assert!(!Method::POST.is_idempotent());
        assert!(!Method::CONNECT.is_idempotent());
        assert!(!Method::PATCH.is_idempotent());
    }

    #[test]
    fn test_extension_method() {
        assert_eq!(Method::from_str("WOW").unwrap(), "WOW");
        assert_eq!(Method::from_str("wOw!!").unwrap(), "wOw!!");

        let long_method = "This_is_a_very_long_method.It_is_valid_but_unlikely.";
        assert_eq!(Method::from_str(long_method).unwrap(), long_method);

        let longest_inline_method = [b'A'; InlineExtension::MAX];
        assert_eq!(
            Method::from_bytes(&longest_inline_method).unwrap(),
            Method(ExtensionInline(
                InlineExtension::new(&longest_inline_method).unwrap()
            ))
        );
        let shortest_allocated_method = [b'A'; InlineExtension::MAX + 1];
        assert_eq!(
            Method::from_bytes(&shortest_allocated_method).unwrap(),
            Method(ExtensionAllocated(
                AllocatedExtension::new(&shortest_allocated_method).unwrap()
            ))
        );
    }

    #[test]
    fn test_extension_method_chars() {
        const VALID_METHOD_CHARS: &str =
            "!#$%&'*+-.^_`|~0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        for c in VALID_METHOD_CHARS.chars() {
            let c = c.to_string();

            assert_eq!(
                Method::from_str(&c).unwrap(),
                c.as_str(),
                "testing {c} is a valid method character"
            );
        }
    }
}

// Builder wrappers use these constructors instead of relying on constant
// lowering across the request/method module boundary. Each has the exact
// public model of its built-in method.
impl Method {
    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Get)
            && result.invariant()
    ))]
    pub(crate) const fn builder_get() -> Method {
        Method(Get)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Connect)
            && result.invariant()
    ))]
    pub(crate) const fn builder_connect() -> Method {
        Method(Connect)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Post)
            && result.invariant()
    ))]
    pub(crate) const fn builder_post() -> Method {
        Method(Post)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Put)
            && result.invariant()
    ))]
    pub(crate) const fn builder_put() -> Method {
        Method(Put)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Delete)
            && result.invariant()
    ))]
    pub(crate) const fn builder_delete() -> Method {
        Method(Delete)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Head)
            && result.invariant()
    ))]
    pub(crate) const fn builder_head() -> Method {
        Method(Head)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Options)
            && result.invariant()
    ))]
    pub(crate) const fn builder_options() -> Method {
        Method(Options)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Patch)
            && result.invariant()
    ))]
    pub(crate) const fn builder_patch() -> Method {
        Method(Patch)
    }

    #[cfg_attr(creusot, ensures(
        result.deep_model() == method_model_text(MethodModel::Trace)
            && result.invariant()
    ))]
    pub(crate) const fn builder_trace() -> Method {
        Method(Trace)
    }
}
