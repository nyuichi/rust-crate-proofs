#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU16, Ordering};

static CACHE: AtomicU16 = AtomicU16::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn direct_load() -> u16 {
    CACHE.load(Ordering::Relaxed)
}
