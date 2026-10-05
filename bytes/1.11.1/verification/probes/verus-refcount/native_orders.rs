use vstd::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

verus! {
// The actual operations/orders from bytes_mut.rs::increment_shared and
// release_shared, reduced to their atomic slice. No external-body specification.
fn clone_native(ref_count: &AtomicUsize) -> usize {
    ref_count.fetch_add(1, Ordering::Relaxed)
}
fn release_native(ref_count: &AtomicUsize) -> bool {
    let old = ref_count.fetch_sub(1, Ordering::Release);
    if old != 1 { return false; }
    ref_count.load(Ordering::Acquire);
    true
}

#[cfg(negative_native_value)]
fn private_final_release() {
    let count = AtomicUsize::new(1);
    let old = count.fetch_sub(1, Ordering::Release);
    count.load(Ordering::Acquire);
    assert(old == 1);
}
}
fn main() {}
