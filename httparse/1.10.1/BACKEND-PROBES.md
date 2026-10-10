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
CARGO_NET_OFFLINE=true cargo creusot -- --manifest-path Cargo.toml --features cpu-detect
```

The SWAR, SSE, and isolated CPU-detection commands exit successfully after
generating Coma. The dispatch command exits with a compiler error. These
commands do not invoke `why3find` or start a prover. Tool identities reported
by `cargo creusot version` were
`cargo-creusot 0.11.0-dev`, `nightly-2026-02-27`, Why3 `1.8.2+git`, and
why3find `1.2.0+dev`.

## Results

| Backend slice | Translation result | What the generated code establishes |
| --- | --- | --- |
| SWAR block | Rust translation exits 0, with warnings for `usize::from_ne_bytes` and `usize::to_ne_bytes`: “calling external function … with no contract will yield an impossible precondition.” | The full block's Coma function starts with `false any`; it does not preserve the load, byte mask, or first-invalid-byte result. Its `offsetnz` body retains the zero check but its byte extraction/iteration path becomes `false any`. An isolated word-input function does preserve `bw_and`, `bw_or`, `bw_xor`, and `bw_not`, and calls the Creusot `wrapping_sub` model. That expression starts after the byte-to-word representation step, so it does not bridge the actual block body. No proof was run for this arithmetic. |
| SSE | Rust translation exits 0 with: `calling external function '_mm_movemask_epi8' with no contract will yield an impossible precondition`. | The Coma function has the `__m128i` argument type, but its body is `false any`; no movemask semantics are emitted. This does not refine the runtime intrinsic. |
| Runtime dispatch | Translation fails at the global cache declaration with `error: unsupported definition kind ...::dispatch::RUNTIME_FEATURE) Static { safety: Safe, mutability: Not, nested: false }`. | No dispatch Coma function is generated. The actual `AtomicU8` cache cannot cross this translation boundary in the current toolchain. |
| CPU feature macro | Translation exits 0 for the isolated `is_x86_feature_detected!("sse4.2")` function. | On the pinned x86-64 target, Rust macro expansion is `false || ::std_detect::detect::__is_feature_detected::sse4_2()`. The generated Coma has no contract or body for that `std_detect` call and reduces the result to an unconstrained `Any.any_l`. It therefore does not establish the runtime CPUID result or the strict feature condition. |

The probe source keeps these examples small: `match_uri_mask_word` and
`match_uri_char_8_swar` isolate the SWAR arithmetic and its byte conversion;
`sse_intrinsic_probe` makes one `_mm_movemask_epi8` call;
`get_runtime_feature` contains the detector plus relaxed atomic cache; and
`detect_sse42` isolates the CPU-feature macro without the cache. The contracts
in this probe only request translation of those bodies. They are not
assumptions added to httparse itself.

The macro expansion was inspected separately with
`RUSTC_BOOTSTRAP=1 CARGO_NET_OFFLINE=true cargo rustc --lib --features cpu-detect -- -Zunpretty=expanded`.
This command exits 0 and shows that the non-compile-time path calls
`::std_detect::detect::__is_feature_detected::sse4_2()`. Creusot translates the
wrapper but leaves that call unconstrained, so the CPU feature check and its
strict runtime condition remain open even when the static cache is factored
away.

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

## Pure SSE URI-prefix probe

The standalone package in [`verification/probes/backend-sse-prefix`](verification/probes/backend-sse-prefix) isolates the URI-block sequence from `src/simd/sse42.rs::match_url_char_16_sse`, beginning with a preloaded `__m128i`. It does not include the runtime unaligned load or dispatch. The target-local Nth-overlay driver restores selected standard Why3 bit-index axioms removed by the stock driver; it adds no axioms and does not modify the installed Why3 files.

On 2026-10-05, a bounded bottom-up direct Why3 run discharged 9 selected targets and all 18 VCs: `mask_complement_preserves_low_bits` (1), `trailing_zeros_shift_to_nth` (1), `prefix_len_from_mask` (2), `movemask_narrow_preserves_bits` (1), the u16 constant/bounds and u32 boundary checks (3 total), `uri_allowed_mask_16_sse` (8), and `match_uri_char_16_sse_pure` (2). The prefix caller was run after the complement and trailing-zero bridge; URI mask was run after narrowing and bit-width checks. Full target hashes, per-VC outcomes, commands and raw logs are recorded in [`why3-driver/evidence/2026-10-05-bounded-closure-smoke/RUN.md`](verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/RUN.md) and the target snapshot manifest.

The URI-mask probe relies on five local `_mm_*` `extern_spec` contracts over opaque `__m128i` lanes: `_mm_set1_epi8`, `_mm_max_epu8`, `_mm_cmpeq_epi8`, `_mm_andnot_si128`, and `_mm_movemask_epi8`. These are trusted assumptions in this probe; the intrinsic implementations are not proved. The nine direct Why3 results prove selected extracted bodies under their translated contracts. They do not establish `why3find` crate-level closure, production scanner refinement, runtime loads, target-feature checks, or dispatch. The SSE/AVX2 full-scanner ledger row therefore remains OPEN.

Four authorized false-boundary diagnostics (`expected_invalid_u16_wrong_high_bit`, `expected_invalid_u16_width_index_wrap`, `expected_invalid_u32_negative_index_wrap`, and `expected_invalid_u32_two_to_width_wrap`) each returned `Timeout` at 10 seconds, with no counterexample or model. These outcomes are inconclusive, not SAT or `Invalid`. Other targets outside this bounded set remain Pending. The separate backend-translation and SWAR probes retain their existing open status.

## Local Why3 bit-index driver diagnostic

The stock Why3 driver removes standard bit-index relations needed by the translated bitvector goals: the SMT generators remove `Nth_bw_not`, `nth_out_of_bound`, shift-to-`nth` properties, and the Z3 `Nth_bv_is_nth` / `Nth_bv_is_nth2` bridge. A stock-driver export consequently declared `nth` as an uninterpreted function for the complement goal. The target-local overlay comments out only those removals in copies of the installed driver files. It adds no axiom; the restored Why3 standard-library theory remains in the trusted base.

The current source snapshot and direct run status are documented in [`verification/probes/backend-sse-prefix/why3-driver/README.md`](verification/probes/backend-sse-prefix/why3-driver/README.md). The bounded run proved 9 selected positive targets (18 VCs) with the local overlay. Four false-boundary checks timed out without models, so they are not counterexamples. The exact source, driver, target, command and artifact hashes are retained in the probe manifests. Older target-local `proof.json` and session XML files predate the bounded run and are not fresh evidence. These standalone Why3 results do not amount to a `why3find` proof of httparse or close any production scanner path.

## SWAR native-endian word conversion probe

[`verification/probes/backend-swar`](verification/probes/backend-swar) isolates the current 64-bit little-endian `usize::from_ne_bytes` / `to_ne_bytes` bridge used by the SWAR URI block. The first local contracts used an explicit weighted byte sum for decoding and a quantified quotient/remainder formula for encoding. Translation succeeded without an impossible-precondition warning. One proof run passed the generated contract obligations for each external conversion spec but timed out only on the composed `roundtrip ensures result == bytes` goal; those spec VCs do not check the standard-library implementations, whose semantics remain assumed TCB.

The exact command for that first caller attempt was:

```sh
source /workspace/proof-tools/activate.sh
/workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot --no-check-version --simple-triggers=false prove \
  native_endian_roundtrip --why3session --no-cache -- --offline
```

The retained session reports `vc_from_ne_bytes` and `vc_to_ne_bytes` valid; the round-trip postcondition remains unknown at `vc_native_endian_roundtrip.1.1` after about 5.3 seconds. The session XML and proof summary remain under `verification/probes/backend-swar/verif/.../native_endian_roundtrip/`; its `.coma` file has since been regenerated for the second interface, so it no longer matches that first session. The current source changes the model to a fixed eight-limb Horner `pack8`, specifies decoding as `from_ne_bytes(bytes)@ == pack8(bytes)` and encoding as `pack8(to_ne_bytes(word)) == word@`, and adds a body-checked `pack8_injective` lemma. This second interface translates cleanly and preserves the actual conversion calls, but has not been proved yet. It is currently specific to x86-64 little-endian; other widths and endiannesses remain open.
