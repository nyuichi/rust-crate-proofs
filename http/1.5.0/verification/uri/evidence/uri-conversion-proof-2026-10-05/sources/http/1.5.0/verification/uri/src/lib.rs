#![allow(unexpected_cfgs)]

#[cfg(creusot)]
use creusot_std::prelude::DeepModel;

// Use the production UTF-8 byte wrapper and the single bytes-model facade.
#[path = "../../../src/ascii.rs"]
mod ascii;
#[path = "../../../src/byte_str.rs"]
mod byte_str;
#[path = "../../../src/bytes_model.rs"]
mod bytes_model;

// The URI support module includes the exact production error and scheme
// modules. No replacement error or byte-string types are used here.
#[cfg(not(any(http_uri_parts_leaf, http_uri_default_leaf, http_uri_compare_leaf)))]
#[path = "scheme_support.rs"]
pub mod uri;
#[cfg(any(http_uri_parts_leaf, http_uri_default_leaf, http_uri_compare_leaf))]
#[path = "../../../src/uri/mod.rs"]
pub mod uri;

// The Builder proof profile uses the production Error core and the actual
// payload types for unrelated HTTP error variants. These types are carried
// through the URI builder's error state; the harness does not replace them.
#[cfg(http_uri_builder_leaf)]
pub mod header {
    pub use runtime_http::header::{InvalidHeaderName, InvalidHeaderValue, MaxSizeReached};
}

#[cfg(http_uri_builder_leaf)]
pub mod method {
    pub use runtime_http::method::InvalidMethod;
}

#[cfg(http_uri_builder_leaf)]
pub mod status {
    pub use runtime_http::status::InvalidStatusCode;
}

#[cfg(http_uri_builder_leaf)]
#[path = "../../../src/error/core.rs"]
mod error_core;

#[cfg(http_uri_builder_leaf)]
pub use error_core::{Error, Result};

#[cfg(all(creusot, http_uri_builder_leaf, not(http_composition_leaf)))]
pub use error_core::{error_is_empty_uri, error_model, error_model_ref, ErrorModel, ErrorModelRef};

#[cfg(http_uri_builder_leaf)]
pub use uri::Uri;

// Instantiate the generic Port text-view relay with the real `&str` AsRef
// implementation. This is a harness lemma, not a claim that numeric and text
// representations are otherwise consistent for every `Port<T>`.
#[cfg(creusot)]
#[creusot_std::prelude::ensures(result@ == (*value.repr_model())@)]
fn borrowed_port_text<'a>(value: &'a uri::port::Port<&'a str>) -> &'a str {
    value.as_str()
}

#[cfg(creusot)]
#[creusot_std::prelude::ensures(match result {
    Ok(port) => creusot_std::std::string::parse_u16_model(value@.to_bytes())
        == Some(port@@),
    Err(error) => error.deep_model() == 3
        && creusot_std::std::string::parse_u16_model(value@.to_bytes()) == None,
})]
fn parse_borrowed_port(
    value: &str,
) -> std::result::Result<uri::port::Port<&str>, uri::InvalidUri> {
    uri::port::Port::from_str(value)
}

// Check that an external proof caller can use the production domain predicate,
// including the existing behavior that `from_static` scans only the prefix
// before the first authority delimiter.
#[cfg(creusot)]
#[creusot_std::prelude::logic(open)]
fn example_authority_static_bytes() -> creusot_std::logic::Seq<u8> {
    seq![101u8, 120u8, 97u8, 109u8, 112u8, 108u8, 101u8, 46u8,
        99u8, 111u8, 109u8, 47u8, 112u8, 97u8, 116u8, 104u8]
}

#[cfg(creusot)]
#[creusot_std::prelude::ensures(
    uri::authority_static_input_is_valid(example_authority_static_bytes())
)]
fn accepted_authority_static_domain() {
    creusot_std::prelude::proof_assert! {
        example_authority_static_bytes()[11]@ == 47
    };
    creusot_std::prelude::proof_assert! {
        uri::authority::authority_first_delimiter_same_after_empty_prefix(
            example_authority_static_bytes(), 0, 11,
        )
    };
    creusot_std::prelude::proof_assert! {
        uri::authority::authority_first_delimiter_from(
            example_authority_static_bytes(), 0
        ) == 11
    };
    creusot_std::prelude::proof_assert! {
        forall<j: creusot_std::logic::Int> 0 <= j && j < 11 ==>
            !uri::authority::authority_delimiter(example_authority_static_bytes()[j]@)
                && (uri::authority_chars::uri_char_model(
                example_authority_static_bytes()[j]@
            ) != 0 || example_authority_static_bytes()[j]@ == 37)
                && example_authority_static_bytes()[j]@ < 128
                && example_authority_static_bytes()[j]@ != 37
                && example_authority_static_bytes()[j]@ != 58
                && example_authority_static_bytes()[j]@ != 64
                && example_authority_static_bytes()[j]@ != 91
                && example_authority_static_bytes()[j]@ != 93
    };
    creusot_std::prelude::proof_assert! {
        uri::authority::authority_neutral_prefix_is_accepted(
            example_authority_static_bytes(), 11,
        )
    };
    creusot_std::prelude::proof_assert! {
        uri::authority_static_input_is_valid(example_authority_static_bytes())
    };
}

#[cfg(creusot)]
#[creusot_std::prelude::ensures(result@.to_bytes() == example_authority_static_bytes())]
fn example_authority_static_text() -> &'static str {
    let bytes: &'static [u8] = &[
        101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109, 47, 112, 97, 116, 104,
    ];
    creusot_std::prelude::proof_assert! {
        crate::ascii::ascii_bytes_are_valid_utf8(bytes@);
        creusot_std::std::string::valid_utf8(bytes@)
    };
    // Safety: the byte list is ASCII, and the proof establishes its UTF-8 validity.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

#[cfg(creusot)]
#[creusot_std::prelude::ensures(result@ == example_authority_static_bytes())]
fn construct_static_authority_example() -> uri::authority::Authority {
    accepted_authority_static_domain();
    uri::authority::Authority::from_static(example_authority_static_text())
}

#[cfg(all(creusot, http_uri_parts_leaf))]
#[creusot_std::prelude::ensures(
    result@.scheme == uri::uri_parts_model(value@).scheme
        && result@.authority == uri::uri_parts_model(value@).authority
        && result@.path_and_query == uri::uri_parts_model(value@).path_and_query
)]
fn actual_uri_into_parts(value: uri::Uri) -> uri::Parts {
    value.into_parts()
}

#[cfg(all(creusot, http_uri_parts_leaf))]
#[creusot_std::prelude::ensures(match result {
    Ok(uri) => uri@.scheme == uri::uri_model_from_parts(value@).scheme
        && uri@.authority == uri::uri_model_from_parts(value@).authority
        && uri@.path_and_query == uri::uri_model_from_parts(value@).path_and_query,
    Err(error) => uri::uri_parts_error_matches(value@, error.deep_model()),
})]
fn actual_uri_from_parts(
    value: uri::Parts,
) -> std::result::Result<uri::Uri, uri::InvalidUriParts> {
    uri::Uri::from_parts(value)
}
