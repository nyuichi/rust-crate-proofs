#![cfg_attr(not(feature = "std"), no_std)]

extern crate creusot_std;

use creusot_std::prelude::ensures;

include!("../../../../src/message.rs");

/// Call the runtime constructor through its proved contract.
#[cfg(creusot)]
#[ensures(result.method == None)]
#[ensures(result.path == None)]
#[ensures(result.version == None)]
#[ensures(result.headers@ == headers@)]
pub fn request_constructor_caller<'h, 'b>(headers: &'h mut [Header<'b>]) -> Request<'h, 'b> {
    Request::new(headers)
}

/// Call the runtime constructor through its proved contract.
#[cfg(creusot)]
#[ensures(result.version == None)]
#[ensures(result.code == None)]
#[ensures(result.reason == None)]
#[ensures(result.headers@ == headers@)]
pub fn response_constructor_caller<'h, 'b>(headers: &'h mut [Header<'b>]) -> Response<'h, 'b> {
    Response::new(headers)
}

/// Check the byte-slice component of the published empty-header constant.
#[cfg(creusot)]
#[ensures(result.value@.len() == 0)]
pub fn empty_header_value_caller() -> Header<'static> {
    EMPTY_HEADER
}
