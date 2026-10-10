#![cfg_attr(creusot, feature(core_intrinsics))]

use creusot_std::prelude::ensures;

include!("../../../../../src/error.rs");

#[derive(PartialEq, Eq)]
#[cfg_attr(creusot, derive(creusot_std::prelude::DeepModel))]
#[repr(i8)]
enum SignedI8 {
    Negative = -7,
    Positive = 20,
}

#[cfg(creusot)]
#[ensures(result)]
pub fn deliberately_wrong_error_eq() -> bool {
    Error::HeaderName == Error::HeaderValue
}

#[cfg(creusot)]
#[ensures(result == 0i8)]
pub fn deliberately_wrong_discriminant_tag() -> i8 {
    unsafe { core::intrinsics::discriminant_value(&SignedI8::Negative) }
}
