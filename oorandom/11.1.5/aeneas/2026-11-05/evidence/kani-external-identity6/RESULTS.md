# Completed original Rust external identity contracts

Two Kani jobs passed34 checks each, zero failures. They call real From<Self>,
Borrow<Self>, BorrowMut<Self> for each Rand32 and Rand64 through original path
dependency. Every public from_state state/increment pair and replacement pair
is quantified. Public state() observes exact identity, shared borrow contents,
mutable borrow contents and replacement propagation. No allocation, loop bound,
private observer, injected state, model or modified original body is used.

This covers six saved external entries, with only those observations. It does
not cover Any, CloneToUninit, six generic conditional conversion families, or
all possible downstream trait implementations. Derived std14 stays excluded,
not newly claimed. Only the recorded debug/x86_64/empty feature configuration
was run; empty default/no-default feature equivalence is not a separate run.
Source/harness/lock/log hashes and tool information are in RESULTS.json.
