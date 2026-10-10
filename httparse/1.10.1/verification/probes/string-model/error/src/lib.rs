use creusot_std::prelude::ensures;

include!("../../../../../src/error.rs");

#[cfg(creusot)]
#[ensures(result@ == "invalid header name"@)]
pub fn actual_error_description() -> &'static str {
    Error::HeaderName.description_str()
}
