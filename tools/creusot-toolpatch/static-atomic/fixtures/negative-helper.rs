#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

fn unregistered_helper(cache: &AtomicU8) -> u8 {
    cache.load(Ordering::Relaxed)
}

pub fn bad_helper_call() -> u8 {
    unregistered_helper(&CACHE)
}
