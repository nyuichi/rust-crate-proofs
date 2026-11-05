# Either value and borrowed-view operations

Fourteen original methods are proved for default features on x86_64: seven previous branch predicates/projections/swap methods plus left_and/right_and, as_ref/as_mut and factor_none/factor_first/factor_second. All fourteen roots are extracted together into one unchanged Generated.Either module; one Proofs module contains fourteen API theorems and flip_twice.

The added contracts establish branch selection, exact optional/tuple payload factoring, unchanged borrowed views and same-variant mutable-payload writeback in Aeneas's backward-function model. They do not establish all crate APIs: callbacks, iterators, trait implementations, serde and other inventory entries remain open. No all-features proof is claimed.

Run verify-all.bash for archive provenance, original tests, one union extraction/hash comparison and all fifteen theorem audits. into_inner remains blocked by a generated Lean duplicate binder; factor_err/factor_ok remain blocked by opaque standard Result.map. No original Rust changes are used.

## Original callback API composition

A single union extracts all 20 original public targets. Six new APIs are proved against an explicitly supplied mathematical callback function, assuming normal modeled callback results on every argument. Eighteen audited theorems cover normal results, unchanged selected callback failure, and unconditional non-selected branch results. The proof is generic over the modeled FnOnce dictionary and both payload types. It does not prove an arbitrary concrete callback terminates, has no panic or external side effects; Rust Drop/unwinding boundaries remain as previously stated.

## Twelve conditional callback APIs

Six further original APIs add mapped-default, lazy-default and branch-and-then composition. Normal results use an explicit total-success mathematical callback premise. Companion theorems propagate selected callback failure exactly and certify nonselected branch behavior unconditionally. The union now extracts 26 original bodies and audits 51 theorems. Arbitrary callbacks, captured effects and Drop/unwinding remain outside those assumptions.
