# itoa 1.0.18 runtime verification status

## Scope map

The production `runtime` module contains the upstream optimized implementation: a
`MaybeUninit` output buffer, the decimal-pair lookup table, reciprocal division
by 100, four-digit chunking, the specialized `u128` path, and output string
construction. The current `cfg(creusot)` build selects `verification.rs`
for the public formatting API and also compiles the shared `divmod100` module.
The verification module proves a separate recursive digit writer over an
initialized byte array. That proof remains useful and must stay intact, but it
does not currently translate or prove the rest of the production algorithm.

The first shared implementation boundary is `divmod100`: normal runtime callers
use this body, and Creusot translates and proves the same executable body as an
independent leaf. Later phases can connect the lookup-table writes and optimized
digit/chunk writers to the already-proved decimal model.

## Proof status

| Component | Contract reviewed | Body proved | Trusted | Integrated runtime proof |
| --- | --- | --- | --- | --- |
| Existing recursive decimal model and public verification-facing API | yes | yes (per current provenance record) | pre-existing ASCII-slice-to-`str` leaf | baseline only; does not cover runtime code |
| Shared optimized `divmod100` (Phase 1) | yes | yes | none added | yes; leaf only, callers remain pending |
| `DECIMAL_PAIRS` representation and indexed lookup lemma | yes | yes (2 VCs) | none | data bridge via CTFE; formatter writes pending |
| Four-digit / two-digit output and loop invariants | pending | no | none | pending |
| `u128` reciprocal division and chunk encoder | pending | no | none | pending |
| Runtime buffer, sign, and output-string integration | pending | no | no new boundary planned | pending |

Phase 1 proves only the leaf contract: for `value < 10_000`, the returned
pair equals Euclidean quotient/remainder by 100, the remainder is below 100,
and the quotient-remainder reconstruction equals the input. The executable
statements are kept unchanged and single-sourced for normal Rust and Creusot.
No trusted declaration is introduced for this phase. The pre-existing trusted
string-construction leaf belongs to the separate recursive verification model;
it is not used to justify the `divmod100` body.

Phase 2 moves the production table declaration into the shared `decimal_pairs`
module while preserving the original runtime `static` and its byte-string
initializer. The formatter body continues to use the same indexed reads.

Creusot 0.11.0-dev cannot translate the safe immutable static definition; the
driver reports `Static { safety: Safe, mutability: Not, nested: false }` as an
unsupported definition kind. Changing the table to a `const` initialized by
`DecimalPairs(*b"...")` also fails translation with
`Unsupported constant value: Scalar(alloc94) of type &'?2 [u8; 200_usize]`.
These are Creusot translation limits for statics and the byte-string array
constant.

The module therefore has an explicit cfg representation difference: normal Rust
uses the unchanged byte-string literal in the runtime static; Creusot uses a
`const` built from an explicit 200-byte scalar array. Both scalar initializers
come from one macro. In the normal cfg, a const-evaluated loop checks all 200
scalar bytes against the 200 literal bytes, and a compile-time assertion fails
on any mismatch. Thus normal Rust compilation ties the Creusot const to the exact
runtime table value without a trusted axiom.

`decimal_pair_correct(n)` requires `n < 100`. Its postcondition proves the
actual proof-table cells at indices `2*n` and `2*n+1` equal
`48 + n/10` and `48 + n%10`, proves the latter index is below 200, and ties its
returned bytes to those cells. The helper reads the array cells directly. The
Creusot run proves both the constant setter and this indexed lookup contract (2
VCs); the runtime formatter's unsafe write sites remain a later proof phase.

Immediately after Phase 1 and before the Phase 2 table patch, the default and
`--all-features` `verify-all.bash` runs both passed. Each configuration produced
69 proof units and 190 split goals with zero failures. The `magic_quotient`
arithmetic lemma and the actual shared `divmod100` body each discharged one VC.
The crate's established recursive-model obligations also remained green.

## Runtime test checks

After extracting the shared leaf, `cargo test --manifest-path Cargo.toml`
passes all 11 integration tests and 2 doctests. The release test command
`cargo test --manifest-path Cargo.toml --tests --all-features --release`
passes all 11 integration tests. `--tests` excludes doctests from that
all-features release run. These normal and release commands also passed after
Phase 2 extracted the table declaration; a normal `cargo check` passed the
compile-time table equality assertion. Setup also recorded that
`cargo test --all-features` fails at the `no-panic` linker step in debug, and
`cargo test --all-features --release` passes integration tests but fails while
linking doctests. The all-features proof configuration separately enables
`no-panic`; it does not run the upstream test suite.

## Baseline and run record

The crate-scoped verification command is `./verify-all.bash`, run from
`itoa/1.0.18`. It invokes both the default and `--all-features` Creusot proof
configurations, with Cargo networking disabled. The all-feature run enables the
optional `no-panic` dependency; it is not an upstream test-suite run.

Baseline at source commit `c6d8352aecd423d311f26e64bef3ed37ec950b84`:
`./verify-all.bash` from this directory. Both the default and `--all-features`
configurations passed before Phase 1 changes: each translated 67 libraries
and discharged 188 verification conditions with zero failures. The driver was
`cargo-creusot 0.11.0-dev` from upstream commit
`437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`, using `nightly-2026-02-27`
(`rustc 1.95.0-nightly (6a979b3e3 2026-02-26)`, with `rustc-dev` and
`llvm-tools`). The root README records `creusot-libs` commit
`7a48f5a5b1cb15a11c4e744568ca187331a30025`, while the vendored library declares
`0.11.0-dev`; the matching driver above is built from the latter source line.

Why3 was `1.8.2+git` at commit
`2c0f2992af85f82f3eda0f158dcf10e62e0db875`; Why3find was `v1.2.0+dev` at
`3a98fc320b9cbf2e71860da1c8dc188a966eee96`. Solver versions were Alt-Ergo
2.6.2, Z3 4.15.3, CVC4 1.8, and CVC5 1.3.1.

The proof environment used `/tmp/rustup-home`, `/tmp/cargo-home`,
`/tmp/creusot-data`, `/tmp/creusot-cache`, and `/tmp/creusot-config` (symlinked
to `/workspace/proof-tools` to avoid `/tmp` disk limits), with
`LD_LIBRARY_PATH` including the nightly toolchain's `lib` and
`/tmp/local-ocaml/usr/lib/x86_64-linux-gnu`. `verify-all.bash` sets
`CARGO_NET_OFFLINE=true`; Why3 proof runs were outside the sandbox because they
use Unix sockets. Baseline used
`CARGO_TARGET_DIR=/workspace/proof-tools/targets/itoa-baseline`; Phase 1 used
`CARGO_TARGET_DIR=/workspace/proof-tools/targets/phase1`.
After Phase 2, `./verify-all.bash` passes in both default and `--all-features`
configurations. Each run has 70 generated proof units and 192 split goals with
zero failures (up from 69 units / 190 split goals after Phase 1). The two new
table goals are the constant setter and `decimal_pair_correct`. The Phase 2
change adds no trusted declarations. The proof configuration change to a scalar
`const` is paired with the normal-config CTFE byte equality assertion described
above.

