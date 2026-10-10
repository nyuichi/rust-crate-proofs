#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static LEFT: AtomicU8 = AtomicU8::new(0);
static RIGHT: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn bad_static_join(choose_left: bool) -> u8 {
    let selected = if choose_left { &LEFT } else { &RIGHT };
    selected.load(Ordering::Relaxed)
}
