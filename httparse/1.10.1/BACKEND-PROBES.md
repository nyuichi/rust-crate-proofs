# httparse backend translation probes

These tiny probes check whether Creusot 0.11.0-dev preserves the runtime
backend expressions while translating them. They are translation diagnostics,
not proofs, refinement bridges, or evidence of crate-level verification. They
use the actual SWAR URI mask and dispatch shapes from `src/simd/swar.rs` and
`src/simd/runtime.rs`, plus a single SSE intrinsic from `src/simd/sse42.rs`.

The standalone package is in [`verification/probes/backend-translation`](verification/probes/backend-translation). Its generated `verif/` and Cargo `target/` directories are ignored.

## Reproduction

Run each command from `verification/probes/backend-translation` after loading
the pinned tool environment:

```sh
source /workspace/proof-tools/activate.sh
CARGO_NET_OFFLINE=true cargo creusot -- --manifest-path Cargo.toml --features swar
CARGO_NET_OFFLINE=true cargo creusot -- --manifest-path Cargo.toml --features sse
CARGO_NET_OFFLINE=true cargo creusot -- --manifest-path Cargo.toml --features dispatch
```

The first two commands exit successfully after generating Coma. The dispatch
command exits with a compiler error. These commands do not invoke `why3find` or
start a prover. Tool identities reported by `cargo creusot version` were
`cargo-creusot 0.11.0-dev`, `nightly-2026-02-27`, Why3 `1.8.2+git`, and
why3find `1.2.0+dev`.

## Results

| Backend slice | Translation result | What the generated code establishes |
| --- | --- | --- |
| SWAR block | Rust translation exits 0, with warnings for `usize::from_ne_bytes` and `usize::to_ne_bytes`: “calling external function … with no contract will yield an impossible precondition.” | The full block's Coma function starts with `false any`; it does not preserve the load, byte mask, or first-invalid-byte result. Its `offsetnz` body retains the zero check but its byte extraction/iteration path becomes `false any`. An isolated word-input function does preserve `bw_and`, `bw_or`, `bw_xor`, and `bw_not`, and calls the Creusot `wrapping_sub` model. That expression starts after the byte-to-word representation step, so it does not bridge the actual block body. No proof was run for this arithmetic. |
| SSE | Rust translation exits 0 with: `calling external function '_mm_movemask_epi8' with no contract will yield an impossible precondition`. | The Coma function has the `__m128i` argument type, but its body is `false any`; no movemask semantics are emitted. This does not refine the runtime intrinsic. |
| Runtime dispatch | Translation fails at the global cache declaration with `error: unsupported definition kind ...::dispatch::RUNTIME_FEATURE) Static { safety: Safe, mutability: Not, nested: false }`. | No dispatch Coma function is generated. The actual `AtomicU8` cache cannot cross this translation boundary in the current toolchain. |

The probe source keeps these examples small: `match_uri_mask_word` and
`match_uri_char_8_swar` isolate the SWAR arithmetic and its byte conversion;
`sse_intrinsic_probe` makes one `_mm_movemask_epi8` call; and
`get_runtime_feature` contains the runtime detector plus the relaxed atomic
load/store cache pattern. The contracts in this probe only request translation
of those bodies. They are not assumptions added to httparse itself.

## Status and consequence

These results confirm that a parser model or a proof of `match_uri_mask_word`
alone would not establish the requested optimized-backend behavior. The exact
byte-to-word relation, SSE intrinsic semantics, and runtime atomic dispatch
remain unbridged. Do not count any of these paths as proved or mechanically
refined until the generated backend operations have checked semantics tied to
the executable expressions. No intrinsic or atomic contract was added to the
crate's TCB here.

## Exact SSE intrinsic contract probe

The `sse-exact` feature in [`backend-translation/src/lib.rs`](verification/probes/backend-translation/src/lib.rs) adds a local abstract lane view and precise `extern_spec!` contracts for a tiny intrinsic composition. Its view has type `[i8; 16]`, so it fixes the lane count and the signed byte range. The contract for `_mm_set1_epi8(a)` says each lane is `a`. The contract for `_mm_movemask_epi8(v)` defines bit `i` as 1 exactly when lane `i` is negative, which is the sign bit of that byte, and places it at weight `2^i`.

These postconditions follow the x86-64 API descriptions for [`_mm_set1_epi8`](https://doc.rust-lang.org/core/arch/x86_64/fn._mm_set1_epi8.html), which broadcasts one 8-bit integer to all 16 lanes, and [`_mm_movemask_epi8`](https://doc.rust-lang.org/core/arch/x86_64/fn._mm_movemask_epi8.html), which copies each lane's most-significant bit into the corresponding low result bit. The one caller splats `-1` and returns the resulting mask. It has no added function precondition; its unsafe target-feature boundary remains the caller's obligation, as in the runtime SSE module.

Translation succeeds without contractless-intrinsic warnings. The generated caller Coma contains calls to `_mm_set1_epi8` and `_mm_movemask_epi8`; the call bodies are abstract `any` operations constrained by the two local contracts, not translations of machine instructions. The `__m128i` type and generated `inv___m128i` predicate remain abstract. The set1 contract establishes that predicate for its result, which is then consumed as the movemask input. The opaque lane view is a proof model, not a runtime wrapper or an independently checked Rust `View` implementation.

The exact target-scoped proof command, run through the shared one-prover queue from the probe directory, was:

```sh
/workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot --no-check-version --simple-triggers=false prove \
  all_ones_movemask --why3session --no-cache -- --offline --features sse-exact
```

Result: `Theory verif.httparse_backend_translation_probe_rlib.sse_exact.all_ones_movemask.Coma: ✔ (3)` with one Z3 4.15.3 prover and a 1000 MiB limit. This checks the caller against the local external specifications. It does not prove that the upstream intrinsic implementations satisfy them, or that a parser vector load, all SSE instructions, runtime feature detection, AVX2, or NEON is refined by this model.

The lane model is nonempty and consistent with the specified operations: a vector whose 16 signed lanes are all `-1` satisfies the set1 postcondition, and the movemask model yields `1 + 2 + ... + 32768 = 65535`. This is a reviewed model witness, not a separate solver satisfiability query. The abstract `inv___m128i` predicate is deliberately recorded as an additional unresolved type-model boundary. The exact external contracts are local to this probe and have not been installed in `creusot-std`.
