#![allow(dead_code)]

extern crate creusot_std;

use creusot_std::prelude::{ensures, requires, View};

#[path = "../../../../src/verification/model.rs"]
mod verification_model;

#[path = "../../../../src/verification/spaces.rs"]
mod verification_spaces;

#[path = "../../../../src/iter.rs"]
pub mod iter;

use crate::iter::Bytes;

#[macro_use]
#[path = "../../../../src/macros.rs"]
mod macros;

include!("../../../../src/status.rs");
include!("../../../../src/error.rs");

pub type Result<T> = core::result::Result<Status<T>, Error>;

include!("../../../../src/skip_spaces.rs");
