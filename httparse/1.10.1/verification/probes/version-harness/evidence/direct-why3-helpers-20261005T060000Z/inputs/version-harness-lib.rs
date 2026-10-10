#![allow(dead_code)]
#![allow(unexpected_cfgs)]

extern crate creusot_std;

#[allow(unused_imports)]
use creusot_std::prelude::{check, ensures, ghost, requires, snapshot, View};

#[path = "../../../../src/verification/model.rs"]
mod verification_model;

#[path = "../../../../src/verification/version.rs"]
mod verification_version;

#[path = "../../../../src/iter.rs"]
pub mod iter;

use crate::iter::Bytes;

#[macro_use]
#[path = "../../../../src/macros.rs"]
mod macros;

include!("../../../../src/status.rs");
include!("../../../../src/error.rs");

pub type Result<T> = core::result::Result<Status<T>, Error>;

include!("../../../../src/parse_version.rs");
