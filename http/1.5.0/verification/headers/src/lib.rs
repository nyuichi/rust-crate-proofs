#![allow(unexpected_cfgs)]

// Verify the production byte classifiers directly while excluding unrelated
// HeaderValue representation and external Bytes behavior from these leaf VCs.
#[path = "../../../src/header/value_validation.rs"]
mod value_validation;
