# Attempt 5 full 15-COMA proof result

**Partial result.** Fresh `--no-cache` proof covered 15 frozen COMAs and 37 independently printed direct roots (10 own function bodies, 5 trait refinements, 22 imported literal-true support stubs). The proof JSON for all 15 targets has exactly the independently printed root set. The Why3 trees contain 40 terminal leaves: 39 successful leaves and one `null` leaf. Fourteen target COMAs have every root fully discharged; only `Sealed::find for &str` has an open split leaf.

The failing root is `vc_find_ref_str` for `impl_Sealed_for_ref_str/find.coma`, annotated `find ensures`. The full log says 3/4 split leaves passed. The failed prover cache entry is Z3 `Timeout` after 3.882 seconds under the configured `time=2` setting; no counterexample or false-precondition result was reported. `String` and `&String` adapter roots pass by consuming the imported `&str` summary, so their functional string-key chain remains conditional on this open body. The bounded five-COMA pilot, including generic `get2`, HeaderName and `&HeaderName` bodies/refinements, separately passed all ten roots.

| Target group | Own roots | Support stubs | Result |
|---|---:|---:|---|
| `HeaderMap::{get,get2,get_mut,get_all,contains_key}` | 5 | 12 | all roots pass |
| `Sealed::find` for `HeaderName` and `&HeaderName`, with refinements | 6 | 2 | all roots pass |
| `Sealed::find` for `&str`, with refinement | 2 | 5 | body has one open split leaf; refinement passes |
| `Sealed::find` for `String` and `&String`, with refinements | 4 | 3 | selected roots pass; `&str` summary remains unclosed |
| **Total** | **15** | **22** | **39/40 terminal leaves pass** |

Support stubs are imported caller summaries only; they do not prove their callee bodies. In particular, the imported `HeaderMap::find` and hash helper bodies are not in this selection. The source warnings for `RandomState::new`, `BuildHasher::build_hasher`, and `Hasher::finish` occur in unselected hash branches, so lookup closure remains conditional. This batch does not prove `None iff absent`, mutation preservation, the `get_mut` frame/readiness property, or the integrated HTTP crate. At the public API boundary, `get` has no functional return postcondition (its body root proves only the call/index obligations against the imported `get2` summary); `get_mut` likewise has no value-preservation, prophecy, frame, or readiness postcondition. `get2` has the Some-only stored-value/key relation described below. `get_all` only relates a Some returned index to an in-range matching key; it does not specify the full multi-value view. `contains_key` proves only that `true` implies an in-range matching stored key, with no converse.

## Open `&str` bridge

The frozen failing COMA declares the imported `PartialEqModel<NameModel, ParsedModel>::eq_model` as an opaque predicate. The `HdrName::from_bytes` callback summary provides that opaque relation, while `text_matches_header_model` needs the concrete `header_name_matches_hdr_name` predicate. They are not definitionally connected in this proof package. The missing fact is a visibility/bridge lemma, not evidence of an impossible precondition. The sound repair under evaluation is a visible delegating Name-side `eq_model` definition or an independently proved Name-side bridge lemma. No Map-side trusted axiom was added.

## Contracts and vacuity

The frozen `get2` precondition is `header_map_find_ready(self)` plus the ordinary `HeaderMap<T>` type invariant. It is satisfiable: the empty-entry branch is direct, and a nonempty witness can use mask 1, two slots, one entry, one empty slot, and one occupied slot whose hash matches that entry. The selected HeaderName adapter methods use the same ready predicate and ordinary type invariants. None of the selected entrypoint requires is `false`; the only `false` in the emitted family is the unreachable non-`Some` branch used to discharge the standard `Option` elimination.

The `get2` postcondition for `Some` yields an in-range index, value equality with `header_map_value_at`, and the abstract `Sealed::matches_header` key observer. `None` has no absence-completeness claim. The Map observer functions are pure definitions over actual entry fields; the generic key observer remains abstract in `get2` and concrete HeaderName observers reduce to model equality in the selected body COMAs. Generic `K` and value `T` remain distinct types; no axiom equates arbitrary models.

The proof was run from fresh frozen COMAs with the repository wrapper: one prover at a time, 1024 MiB per prover, `--no-cache`, Why3 `split_vc`, and exact feature/config flags recorded in `full15-proof-command.sh`. The frozen proof inputs remain Map source SHA `76322738120be3b4a616cd27b4e4fa48fa5b76ba3a5f8db8cfed9869da82a3d8` and Name source SHA `966107ba5e8df15aa82c4d34220ab3a7b8e4e7da7b4dfc4a050cff1232dcb064`. A later Map-only import cleanup is outside this frozen proof snapshot and must not be represented as verified until its updated input/emission is reconciled.
