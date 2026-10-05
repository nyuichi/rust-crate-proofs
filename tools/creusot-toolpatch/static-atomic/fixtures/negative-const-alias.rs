#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);
const ALIAS: &AtomicU8 = &CACHE;

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn bad_const_alias() -> u8 {
    ALIAS.load(Ordering::Relaxed)
}
