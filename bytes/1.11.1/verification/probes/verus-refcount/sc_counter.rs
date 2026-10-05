use vstd::prelude::*;
use vstd::atomic::PAtomicUsize;

verus! {
// Executable SC comparison only. Permission is held exclusively here; this is
// not a concurrent ticket protocol and does not change bytes' native orders.
fn abc_sc_counter() {
    let (count, Tracked(mut permission)) = PAtomicUsize::new(2);
    let a = count.fetch_sub(Tracked(&mut permission), 1);
    assert(a == 2);
    let b_clone = count.fetch_add(Tracked(&mut permission), 1);
    assert(b_clone == 1);
    let b_release = count.fetch_sub(Tracked(&mut permission), 1);
    assert(b_release == 2);
    let c_release = count.fetch_sub(Tracked(&mut permission), 1);
    assert(c_release == 1);
    let final_count = count.load(Tracked(&permission));
    assert(final_count == 0);
}
}
fn main() { abc_sc_counter(); }
