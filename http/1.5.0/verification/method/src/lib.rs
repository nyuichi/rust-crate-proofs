#![allow(dead_code, unexpected_cfgs)]

// Include the production source verbatim. This target narrows translation to
// Method and its private extension representations.
#[path = "../../../src/ascii.rs"]
pub(crate) mod ascii;

#[path = "../../../src/method.rs"]
pub mod method;
