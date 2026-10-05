#![allow(unexpected_cfgs)]

// This focused crate includes the production header implementation directly.
// Its exact `Bytes` contract is conditional on the separate bytes proof.
#[path = "../../../src/bytes_model.rs"]
pub(crate) mod bytes_model;

#[path = "../../../src/ascii.rs"]
pub(crate) mod ascii;

#[macro_use]
#[path = "../../../src/convert.rs"]
mod convert;

#[cfg(not(http_header_value_leaf))]
#[path = "../../../src/byte_str.rs"]
mod byte_str;

pub mod header;

#[cfg(all(creusot, http_header_name_leaf, feature = "negative-model-probes"))]
mod negative_model_probes {
    use crate::header::name::{header_name_model_bytes, HeaderName};
    use creusot_std::prelude::*;

    #[requires(header_name_model_bytes(value.deep_model()) == value@)]
    #[ensures(false)]
    fn header_model_contradiction_probe(value: &HeaderName) {}

    const PROBE_TEXT: &str = "probe";

    #[requires(PROBE_TEXT@ == PROBE_TEXT@)]
    #[ensures(result@ == PROBE_TEXT@)]
    #[ensures(false)]
    fn constant_model_contradiction_probe() -> &'static str {
        PROBE_TEXT
    }
}
