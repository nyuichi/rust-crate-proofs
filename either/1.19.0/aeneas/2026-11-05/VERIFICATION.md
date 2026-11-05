# Either value and borrowed-view operations

Fourteen original methods are proved for default features on x86_64: seven previous branch predicates/projections/swap methods plus left_and/right_and, as_ref/as_mut and factor_none/factor_first/factor_second. All fourteen roots are extracted together into one unchanged Generated.Either module; one Proofs module contains fourteen API theorems and flip_twice.

The added contracts establish branch selection, exact optional/tuple payload factoring, unchanged borrowed views and same-variant mutable-payload writeback in Aeneas's backward-function model. They do not establish all crate APIs: callbacks, iterators, trait implementations, serde and other inventory entries remain open. No all-features proof is claimed.

Run verify-all.bash for archive provenance, original tests, one union extraction/hash comparison and all fifteen theorem audits. into_inner remains blocked by a generated Lean duplicate binder; factor_err/factor_ok remain blocked by opaque standard Result.map. No original Rust changes are used.

## Original callback API composition

A single union extracts all 20 original public targets. Six new APIs are proved against an explicitly supplied mathematical callback function, assuming normal modeled callback results on every argument. Eighteen audited theorems cover normal results, unchanged selected callback failure, and unconditional non-selected branch results. The proof is generic over the modeled FnOnce dictionary and both payload types. It does not prove an arbitrary concrete callback terminates, has no panic or external side effects; Rust Drop/unwinding boundaries remain as previously stated.

## Twelve conditional callback APIs

Six further original APIs add mapped-default, lazy-default and branch-and-then composition. Normal results use an explicit total-success mathematical callback premise. Companion theorems propagate selected callback failure exactly and certify nonselected branch behavior unconditionally. The union now extracts 26 original bodies and audits 51 theorems. Arbitrary callbacks, captured effects and Drop/unwinding remain outside those assumptions.

## Original IntoEither provided defaults and real blanket dispatch

The actual bodies are src/into_either.rs29-35 and55-61. The impl<T> IntoEither for T at64 has no overrides and is separately extracted as metadata, not invented as a handwritten body. Its generated real dictionary field is definitionally the proved original into_either default. Specialization to that dictionary removes the conversion-law hypothesis from the new public into_either_with theorem. Only the explicit normal predicate call premise remains, with unchanged modeled callback failure proved separately. Generic default-body conditional theorems are retained as helpers and distinguished from the actual blanket theorem. The union has28 public callable targets and one additional trait-implementation root, with60 audited theorems and complete machine-module simultaneous import. Arbitrary callback termination/effects and independent Rust memory-model claims remain outside scope.

## Original handwritten From conversions

Source lib.rs1438-1443 converts Result<R,L> to Either<L,R> with Ok→Right and Err→Left, preserving payloads. Source1448-1453 converts back with Left→Err and Right→Ok. Its actual foreign self remains core::result::Result<R,L>. Both normal-return contracts cover all variants and generic modeled sized payloads without callback assumptions. Both roundtrip orders are proved, including every error payload. The union has30 public callables plus the actual blanket-implementation metadata root (31selectors), with64audited theorems and all machine modules jointly imported. Old28 guarantees and their boundaries are retained.

## Default fallback branch/delegation

Original left_or_default/right_or_default preserve the matching payload without any default-result premise. The opposite branch delegates the supplied Default computation in the value model; the main theorem includes its whole abstract outcome without predicting its value or termination. Normal value, modeled failure and modeled divergence are proved separately under the respective explicit premises. These are not proofs of an arbitrary Default implementation, nor real Rust Drop panic/nontermination/external effects or unwinding/return ordering for discarded payloads. The union has32 public callables plus the original blanket metadata root,74audited main/helper theorems, and all machine modules are imported together. All old30 mappings and their prior guarantees remain.
