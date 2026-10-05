# HTTP Method verification

This leaf harness includes the production `src/method.rs` and `src/ascii.rs` directly. It proves the emitted source bodies and their refinements; it does not replace the runtime implementation.

## Current proof result

At the source snapshot recorded in the evidence file, a clean translation emitted 103 targets. The fresh `--no-cache` proof run passed all 103 targets and 877 leaves: 737 own-goal leaves and 140 callee-contract leaves. The target-by-target COMA and proof JSON hashes, exact source hashes, solver versions, and run details are in [final-2026-10-05.json](evidence/final-2026-10-05.json).

After the proof batch, the standard-library `convert.rs` and `partial_ord.rs` sources changed. A package-scoped clean translation against those current sources produced byte-identical COMA files for all 103 targets, matching the proven COMA hashes. The existing proof JSON therefore applies to the same task bytes; the exact source and re-emission comparison is recorded in the evidence file.

The 515-leaf `canonical_method_text_is_injective` proof is a finite case split over the 12 `MethodModel` variants on both sides. It completed in the bounded run. The ASCII comparison bridge and all nine fixed-method builder constructors were also freshly proved.

## Covered behavior

The proof set covers the ten known-method classifiers and materializers; `Method::from_bytes`, `as_str`, `is_safe`, and `is_idempotent`; inline and allocated extension constructors; and exact text/invariant refinements for cloning, `From<&Method>`, `Default` (GET), and `TryFrom`/`FromStr` conversions.

It also covers structural `Method` equality against the byte-sequence model, equality with `str`, `&str`, and `&Method` in both directions, and `Ord`/`PartialOrd` against lexicographic byte-model order. Method `Debug` and `Display` have exact successful-output refinements. `InvalidMethod` `Debug` and `Display` prove formatter append preservation.

The nine crate-private `builder_*` constructors return the exact GET, POST, PUT, DELETE, HEAD, CONNECT, PATCH, OPTIONS, and TRACE values with their canonical-model and invariant postconditions. The HTTP Request harness uses these constructors to model its fixed-method shortcuts.

The `Hash` bodies preserve the valid hasher invariant. They do not claim a digest value or general hash coherence. The manual `Inner` body preserves the original derived callback order; a recording-hasher regression compares all 12 variants with a test-only enum that derives `Hash`.

## Runtime regression

From this directory, the final runtime suite passed 10 tests (5 unit, 1 hash protocol, and 4 token/extension tests):

```sh
RUSTFLAGS='-Zcrate-attr=feature(hasher_prefixfree_extras)' cargo test --locked --offline
```

The tests cover all 256 byte values against an independent RFC token allowlist, the inline/allocated boundary and `From<&Method>`, and the derived Hash callback trace.

## Reproduction

With the repository tool environment activated, emit and prove the leaf targets with:

```sh
CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http cargo clean -p http-method-proofs --offline
CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http cargo creusot --simple-triggers=false -- --locked --offline
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache -- --locked --offline
```

The shared proof wrapper uses one prover and 1024 MiB per prover. This is an isolated Method/ASCII source proof, not an integrated proof of every HTTP module or dependency. Standard-library hasher callback contracts are explicit boundaries; no digest semantics are assumed.
