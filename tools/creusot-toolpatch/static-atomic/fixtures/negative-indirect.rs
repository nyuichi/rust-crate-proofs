#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn bad_indirect_call() -> u8 {
    let load: fn(&AtomicU8, Ordering) -> u8 = AtomicU8::load;
    load(&CACHE, Ordering::Relaxed)
}
