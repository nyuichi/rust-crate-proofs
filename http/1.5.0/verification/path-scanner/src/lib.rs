#![allow(unexpected_cfgs)]
#![allow(dead_code)]

// Include the actual error representation, the actual URI length limit, and
// the exact production scanner. The PathAndQuery wrapper is outside this
// focused target because its formatting/indexing and dyn Any optimization are
// separate verification obligations.
#[path = "../../../src/uri/error.rs"]
mod error;
use error::ErrorKind;

#[path = "../../../src/uri/limits.rs"]
mod limits;
use limits::MAX_LEN;

#[path = "../../../src/uri/path_scan.rs"]
mod path_scan;
