#![cfg_attr(creusot, feature(core_intrinsics))]

use creusot_std::prelude::{bitwise_proof, ensures};

include!("../../../../src/error.rs");

#[derive(PartialEq, Eq)]
#[cfg_attr(creusot, derive(creusot_std::prelude::DeepModel))]
#[repr(i8)]
enum SignedI8 {
    Negative = -7,
    Positive = 20,
}

#[derive(PartialEq, Eq)]
#[cfg_attr(creusot, derive(creusot_std::prelude::DeepModel))]
#[repr(i16)]
enum SignedI16 {
    Negative = -300,
    Positive = 1000,
}

#[cfg(creusot)]
#[ensures(result@ == "invalid header name"@)]
pub fn actual_error_description() -> &'static str {
    Error::HeaderName.description_str()
}

#[cfg(creusot)]
#[ensures(result)]
pub fn error_equal_same_variant() -> bool {
    Error::HeaderName == Error::HeaderName
}

#[cfg(creusot)]
#[ensures(!result)]
pub fn error_equal_different_variants() -> bool {
    Error::HeaderName == Error::HeaderValue
}

#[cfg(creusot)]
#[ensures(result)]
pub fn repr_i8_equal_same_variant() -> bool {
    SignedI8::Negative == SignedI8::Negative
}

#[cfg(creusot)]
#[ensures(!result)]
pub fn repr_i8_equal_different_variants() -> bool {
    SignedI8::Negative == SignedI8::Positive
}

#[cfg(creusot)]
#[ensures(result)]
pub fn repr_i16_equal_same_variant() -> bool {
    SignedI16::Negative == SignedI16::Negative
}

#[cfg(creusot)]
#[ensures(!result)]
pub fn repr_i16_equal_different_variants() -> bool {
    SignedI16::Negative == SignedI16::Positive
}

#[cfg(creusot)]
#[ensures(result == -7i8)]
pub fn repr_i8_discriminant_math() -> i8 {
    unsafe { core::intrinsics::discriminant_value(&SignedI8::Negative) }
}

#[cfg(creusot)]
#[bitwise_proof]
#[ensures(result == -7i8)]
pub fn repr_i8_discriminant_bitwise() -> i8 {
    unsafe { core::intrinsics::discriminant_value(&SignedI8::Negative) }
}

#[cfg(creusot)]
#[ensures(result == -300i16)]
pub fn repr_i16_discriminant_math() -> i16 {
    unsafe { core::intrinsics::discriminant_value(&SignedI16::Negative) }
}

#[cfg(creusot)]
#[bitwise_proof]
#[ensures(result == -300i16)]
pub fn repr_i16_discriminant_bitwise() -> i16 {
    unsafe { core::intrinsics::discriminant_value(&SignedI16::Negative) }
}
