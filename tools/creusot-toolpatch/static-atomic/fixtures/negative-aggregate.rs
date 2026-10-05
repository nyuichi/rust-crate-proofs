#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn bad_aggregate() -> u8 {
    let pair = (&CACHE, 7u8);
    pair.0.load(Ordering::Relaxed)
}
