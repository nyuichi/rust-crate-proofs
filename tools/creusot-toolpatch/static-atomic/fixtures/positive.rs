#![allow(unexpected_cfgs)]

use std::sync::atomic::{AtomicU8, Ordering};

static CACHE: AtomicU8 = AtomicU8::new(0);

#[cfg(creusot)]
#[creusot::decl::logic]
fn cache_value_invariant(value: u8) -> bool {
    value == 0 || value == 1
}

pub fn direct_load() -> u8 {
    CACHE.load(Ordering::Relaxed)
}

pub fn alias_reborrow_load() -> u8 {
    let same = &CACHE;
    let alias = &*same;
    alias.load(Ordering::Relaxed)
}

pub fn direct_store(value: u8) {
    CACHE.store(value, Ordering::Relaxed);
}

mod first {
    use std::sync::atomic::{AtomicU8, Ordering};

    pub fn read_local() -> u8 {
        static CACHE: AtomicU8 = AtomicU8::new(0);
        CACHE.load(Ordering::Relaxed)
    }
}

mod second {
    use std::sync::atomic::{AtomicU8, Ordering};

    pub fn read_local() -> u8 {
        static CACHE: AtomicU8 = AtomicU8::new(0);
        CACHE.load(Ordering::Relaxed)
    }
}

pub fn read_two_distinct_locals() -> (u8, u8) {
    (first::read_local(), second::read_local())
}
