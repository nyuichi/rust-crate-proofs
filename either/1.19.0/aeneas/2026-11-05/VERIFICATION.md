# Either value and borrowed-view operations

Fourteen original methods are proved for default features on x86_64: seven previous branch predicates/projections/swap methods plus left_and/right_and, as_ref/as_mut and factor_none/factor_first/factor_second. All fourteen roots are extracted together into one unchanged Generated.Either module; one Proofs module contains fourteen API theorems and flip_twice.

The added contracts establish branch selection, exact optional/tuple payload factoring, unchanged borrowed views and same-variant mutable-payload writeback in Aeneas's backward-function model. They do not establish all crate APIs: callbacks, iterators, trait implementations, serde and other inventory entries remain open. No all-features proof is claimed.

Run verify-all.bash for archive provenance, original tests, one union extraction/hash comparison and all fifteen theorem audits. into_inner remains blocked by a generated Lean duplicate binder; factor_err/factor_ok remain blocked by opaque standard Result.map. No original Rust changes are used.
