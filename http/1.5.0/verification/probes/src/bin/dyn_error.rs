#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;
use std::error;

pub fn error_source<'a>(
    value: &'a (dyn error::Error + 'static),
) -> Option<&'a (dyn error::Error + 'static)> {
    value.source()
}

pub fn error_type_is<E: error::Error + 'static>(
    value: &(dyn error::Error + 'static),
) -> bool {
    value.is::<E>()
}

fn main() {}
