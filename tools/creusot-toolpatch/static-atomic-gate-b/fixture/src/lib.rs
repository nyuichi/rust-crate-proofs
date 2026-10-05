use creusot_std::prelude::*;
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "partial-predicate")]
#[logic(open, inline)]
pub fn value_invariant(value: u8) -> bool {
    pearlite! { value@ == 0 || value@ / (value@ - 1) == 1 }
}

#[cfg(feature = "prophetic-predicate")]
#[logic(open, inline, prophetic)]
pub fn value_invariant(value: u8) -> bool {
    pearlite! { value@ == 0 || value@ == 1 }
}

#[cfg(feature = "spoof-predicate")]
#[logic]
#[builtin("creusot.int.UInt8$BW$.t'int")]
pub fn fake_u8_view(value: u8) -> Int {
    dead
}

#[cfg(feature = "spoof-predicate")]
#[logic(open, inline)]
pub fn value_invariant(value: u8) -> bool {
    pearlite! { fake_u8_view(value) == 0 || fake_u8_view(value) == 1 }
}

#[cfg(not(any(
    feature = "partial-predicate",
    feature = "prophetic-predicate",
    feature = "spoof-predicate"
)))]
#[logic(open, inline)]
pub fn value_invariant(value: u8) -> bool {
    pearlite! { value@ == 0 || value@ == 1 }
}

#[cfg(feature = "wrong-init")]
static CACHE: AtomicU8 = AtomicU8::new(2);
#[cfg(not(feature = "wrong-init"))]
static CACHE: AtomicU8 = AtomicU8::new(0);

pub fn read_cache() -> u8 {
    let value = CACHE.load(Ordering::Relaxed);
    assert!(value == 0 || value == 1);
    value
}

pub fn store_one() {
    CACHE.store(1, Ordering::Relaxed);
}

#[cfg(feature = "wrong-store")]
pub fn store_invalid() {
    CACHE.store(2, Ordering::Relaxed);
}

pub fn read_twice() -> (u8, u8) {
    (
        CACHE.load(Ordering::Relaxed),
        CACHE.load(Ordering::Relaxed),
    )
}

#[cfg(feature = "skipped-writer")]
#[creusot::no_translate]
pub fn skipped_writer() {
    CACHE.store(2, Ordering::Relaxed);
}

#[cfg(feature = "latest-read")]
pub fn latest_read_is_one() {
    CACHE.store(1, Ordering::Relaxed);
    assert!(CACHE.load(Ordering::Relaxed) == 1);
}

#[cfg(feature = "equal-loads")]
pub fn loads_are_equal() {
    let first = CACHE.load(Ordering::Relaxed);
    let second = CACHE.load(Ordering::Relaxed);
    assert!(first == second);
}

#[cfg(feature = "span-literal-mismatch")]
pub fn span_like_literal_writer() {
    CACHE.store(1, Ordering::Relaxed);
    #[cfg(creusot)]
    panic!("span: src/lib.rs:1:2: 3:4 (#91), escaped \"(#92)\"");
    #[cfg(not(creusot))]
    panic!("span: src/lib.rs:1:2: 3:4 (#93), escaped \"(#94)\"");
}
