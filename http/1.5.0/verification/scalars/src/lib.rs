#![cfg_attr(creusot, feature(nonzero_internals))]
#![allow(dead_code, unexpected_cfgs)]

// These modules are the upstream runtime implementations. This harness only
// narrows the translated crate so scalar proofs can be developed independently
// of the rest of http's unrelated pointer and type-erasure code.
#[path = "../../../src/ascii.rs"]
pub(crate) mod ascii;

#[cfg(feature = "status")]
#[path = "../../../src/status.rs"]
pub mod status;

#[path = "../../../src/version.rs"]
pub mod version;

// Small translation probe: check whether numeric array constants preserve
// their contents in the model. This is isolated harness code, not an HTTP
// runtime replacement or a production specification.
#[cfg(all(feature = "status", creusot))]
use creusot_std::prelude::ensures;

#[cfg(all(feature = "status", creusot))]
pub const STATUS_ARRAY_MODEL_PROBE: [u8; 3] = [50, 48, 48];

#[cfg(all(feature = "status", creusot))]
#[ensures(
    STATUS_ARRAY_MODEL_PROBE@[0]@ == 50
        && STATUS_ARRAY_MODEL_PROBE@[1]@ == 48
        && STATUS_ARRAY_MODEL_PROBE@[2]@ == 48
)]
pub fn status_array_model_probe() {}

// Narrow parser probe for the representation used by the static status digit
// table: the array constant is by value, and the function returns a promoted
// reference to it. This separates constant-value support from static-pointer
// support.
#[cfg(all(feature = "status", creusot))]
#[ensures(
    result@[0]@ == 50
        && result@[1]@ == 48
        && result@[2]@ == 48
)]
pub fn status_array_model_borrowed() -> &'static [u8; 3] {
    &STATUS_ARRAY_MODEL_PROBE
}
