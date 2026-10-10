use creusot_std::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "wrong-signature")]
#[logic(open, inline)]
pub fn value_invariant(value: u8) -> bool {
    pearlite! { value@ == 0 }
}

#[cfg(not(feature = "wrong-signature"))]
#[logic(open, inline)]
pub fn value_invariant(value: u8, avx2: bool, sse42: bool) -> bool {
    pearlite! {
        value@ == 0
            || (value@ == 1 && avx2)
            || (value@ == 2 && sse42)
            || value@ == 3
    }
}

// A same-name local item must not control the compiler-generated global
// capability symbols used in the static invariant applications.
#[allow(dead_code)]
fn avx2_usable() -> bool {
    false
}

static CACHE: AtomicU8 = AtomicU8::new(0);

pub fn read_cache() -> u8 {
    CACHE.load(Ordering::Relaxed)
}

pub fn store_zero() {
    CACHE.store(0, Ordering::Relaxed);
}

pub fn store_one() {
    CACHE.store(1, Ordering::Relaxed);
}

pub fn store_two() {
    CACHE.store(2, Ordering::Relaxed);
}

pub fn store_three() {
    CACHE.store(3, Ordering::Relaxed);
}

pub fn read_twice() -> (u8, u8) {
    (CACHE.load(Ordering::Relaxed), CACHE.load(Ordering::Relaxed))
}
