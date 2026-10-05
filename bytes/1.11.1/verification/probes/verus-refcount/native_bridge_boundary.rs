use vstd::prelude::*;
use vstd::invariant::*;
use std::sync::atomic::{AtomicUsize, Ordering};

verus! {
// EXPERIMENTAL TCB ONLY. Ordinary native atomic semantics under exclusive
// permission. No bytes protocol or physical resource appears in these clauses.
#[verifier::external_body]
struct NativeCounter { atomic: AtomicUsize }
#[verifier::external_body]
tracked struct CounterPermission { no_copy: NoCopy }
impl CounterPermission {
    uninterp spec fn id(self) -> int;
    uninterp spec fn value(self) -> usize;
}
impl NativeCounter {
    uninterp spec fn id(&self) -> int;
    #[verifier::external_body]
    fn new(value: usize) -> (result: (Self, Tracked<CounterPermission>))
        ensures result.1@.id() == result.0.id(), result.1@.value() == value,
    {
        (Self { atomic: AtomicUsize::new(value) }, Tracked::assume_new())
    }
    // Deliberately NOT certified atomic for stock AtomicInvariant. Turning on
    // rejected_sc_rule demonstrates why that addition is not harmless.
    #[cfg_attr(rejected_sc_rule, verifier::atomic)]
    #[verifier::external_body]
    fn sub_release(&self, Tracked(p): Tracked<&mut CounterPermission>) -> (previous: usize)
        requires old(p).id() == self.id(), old(p).value() > 0,
        ensures final(p).id() == self.id(), previous == old(p).value(),
            final(p).value() + 1 == old(p).value(),
        opens_invariants none
        no_unwind
    { self.atomic.fetch_sub(1, Ordering::Release) }
    #[cfg_attr(rejected_sc_rule, verifier::atomic)]
    #[verifier::external_body]
    fn swap_release(&self, Tracked(p): Tracked<&mut CounterPermission>) -> (previous: usize)
        requires old(p).id() == self.id(),
        ensures final(p).id() == self.id(), previous == old(p).value(), final(p).value() == 0,
        opens_invariants none
        no_unwind
    { self.atomic.swap(0, Ordering::Release) }
    #[verifier::external_body]
    fn load_acquire(&self, Tracked(p): Tracked<&CounterPermission>) -> (value: usize)
        requires p.id() == self.id(),
        ensures value == p.value(),
        opens_invariants none
        no_unwind
    { self.atomic.load(Ordering::Acquire) }
}

fn private_native_bridge() {
    let (counter, Tracked(mut p)) = NativeCounter::new(1);
    let old = counter.sub_release(Tracked(&mut p));
    let zero = counter.load_acquire(Tracked(&p));
    assert(old == 1 && zero == 0);
}

type CellPerm = vstd::cell::pcell::PointsTo<u64>;
tracked struct State {
    tracked counter: CounterPermission,
    tracked resource: Option<CellPerm>,
}
struct Pred;
impl InvariantPredicate<(int, vstd::cell::CellId), State> for Pred {
    closed spec fn inv(k: (int, vstd::cell::CellId), s: State) -> bool {
        &&& s.counter.id() == k.0
        &&& s.counter.value() <= 1
        &&& s.counter.value() == 1 ==> s.resource is Some
        &&& s.resource is Some ==> s.resource->0.id() == k.1
    }
}

// This is intentionally an inadmissible weak-memory resource transfer: it
// exports a PCell permission after Release alone, with no Acquire anywhere.
// Default verifier must reject opening the SC invariant around swap_release.
// The rejected_sc_rule switch is a DIAGNOSTIC of an overstrong trusted rule.
#[cfg(any(attempt_weak_invariant, rejected_sc_rule))]
fn extract_without_acquire(
    counter: &NativeCounter,
    Tracked(inv): Tracked<&AtomicInvariant<(int, vstd::cell::CellId), State, Pred>>,
) -> (result: Tracked<Option<CellPerm>>)
    requires inv.constant().0 == counter.id(),
{
    let tracked mut output = None;
    open_atomic_invariant!(inv => state => {
        let old = counter.swap_release(Tracked(&mut state.counter));
        proof {
            if old == 1 {
                output = state.resource;
                state.resource = None;
            }
        }
    });
    Tracked(output)
}
}
fn main() { private_native_bridge(); }
