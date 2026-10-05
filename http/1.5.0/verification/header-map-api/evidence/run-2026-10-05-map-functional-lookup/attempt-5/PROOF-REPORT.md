# Attempt 5 bounded Map pilot proof

**Result: passed.** The frozen five-COMA pilot produced 10 direct roots: 3 own function-body roots, 2 own trait-refinement roots, and 5 imported literal-true support stubs. All five fresh proof JSON root sets exactly match the independently printed goal sets, and all roots are direct successful prover leaves. The proof command and raw log are saved next to this report; fresh proof JSON is copied under `proofs/`.

| Target COMA | Own roots | Imported stub roots | Total |
|---|---:|---:|---:|
| `header/map/impl_HeaderMap_T/get2.coma` | 1 | 3 | 4 |
| `header/map/as_header_name/impl_Sealed_for_HeaderName/find.coma` | 1 | 1 | 2 |
| `header/map/as_header_name/impl_Sealed_for_HeaderName/find__refines.coma` | 1 | 0 | 1 |
| `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find.coma` | 1 | 1 | 2 |
| `header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find__refines.coma` | 1 | 0 | 1 |


This is modular body/refinement evidence for the selected five targets. The support stubs are caller summaries; they do not prove the imported callee implementations. It does not close string-based `Sealed::find`, `get`, `get_mut`, `get_all`, `contains_key`, Map mutation, absence completeness, or a crate-integrated proof. `HeaderMap::get2` only guarantees that a returned value corresponds to some in-range stored entry whose key matches the abstract `Sealed::matches_header` observer; `None` remains unconstrained beyond type invariants.

## Contract and vacuity review

The frozen `get2` precondition is `header_map_find_ready(self)` plus the ordinary `HeaderMap<T>` type invariant. It is not `false`: its first branch accepts `entries.len() == 0`, and for nonempty maps the index-table constraints have explicit ordinary witnesses (the existing ready-predicate witness uses mask 1, two slots, one entry, one empty slot, and a matching occupied slot). The selected `HeaderName` and `&HeaderName` methods require the same ready predicate and ordinary key/map invariants. No selected contract contains an impossible external-function precondition. The `false` in the emitted `Option` elimination VC is the structurally unreachable non-`Some` branch after matching `Some`; it is not a precondition or an assumed contradiction.

The public map projections are pure definitions over the actual entries (`header_map_key_at`, `header_map_value_at`); the generic observer remains abstract at `get2` and concrete HeaderName observer bodies reduce to model equality in the selected Sealed COMAs. The COMAs contain no added axiom equating generic key/value model types and no trusted HTTP contract. The result relation is conditional on the imported generic `Sealed::find` contract. The subset avoids `&str`/`String` parsing, so it does not rely on the pending Name parser-helper body; the full string-key batch will remain conditional until its own parser-related proof chain closes.
