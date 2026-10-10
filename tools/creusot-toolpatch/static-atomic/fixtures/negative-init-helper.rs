#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

const fn make_cache() -> AtomicU8 {
    AtomicU8::new(0)
}

static CACHE: AtomicU8 = make_cache();

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn direct_load() -> u8 {
    CACHE.load(Ordering::Relaxed)
}
