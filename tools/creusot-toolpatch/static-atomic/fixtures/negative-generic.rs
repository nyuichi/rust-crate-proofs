#![allow(unexpected_cfgs)]

use std::ops::Deref;
use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

fn generic_read<T: Deref<Target = AtomicU8>>(cache: T) -> u8 {
    cache.load(Ordering::Relaxed)
}

pub fn bad_generic_call() -> u8 {
    generic_read(&CACHE)
}
