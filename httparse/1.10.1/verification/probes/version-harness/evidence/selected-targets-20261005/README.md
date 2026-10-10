# `parse_version` selected target plan

This manifest freezes the selected targets from the 63-COMA version-harness
translation and the direct Why3 checkpoint. `TARGETS.tsv` selects 38 targets:
29 `iter/*` Bytes/Iterator targets, eight version model/packing targets, and
the actual `parse_version` target. Across the 36 nontrivial main targets, the
fresh direct runs proved 263 leaves Valid. The two literal-true logic targets
created no solver task and are listed separately in
`../direct-why3-helpers-20261005T060000Z/TRIVIAL_GOALS.tsv`. The generated
COMA hashes identify the isolated string-model translation. The last two hash
columns connect the Bytes target IDs to the separate active-byte memory
refresh; those COMA hashes are intentionally not treated as interchangeable
across compiler profiles.

`EXTERNAL_TARGETS.tsv` records the two standard slice-to-array conversion
caller results in the memory refresh. The fresh version checkpoint also
replayed `array8_prefix` from the separate memory-array-conversion COMA tree,
as recorded in `direct-why3-array8-20261005T055700Z`. `array8_prefix` is a
direct dependency of `Bytes::peek_array8`; `array4_prefix` belongs to the
complete reusable Bytes memory bundle but is not called by `parse_version`.

## Bounded proof order

1. `verification_version/base256_head_injective`
2. `verification_version/unfold_native_pack_tail` and
   `verification_version/native_pack_tail_end`
3. `verification_version/native_pack8_injective`
4. `verification_version/version_prefix_byte`, `matches_version_bytes`,
   `short_version_model`, and `parse_version_model`
5. The 29 `iter/*` targets, with `memory-array-conversion/array8_prefix`
   retained as the standard conversion caller dependency
6. `parse_version`

There is no distinct COMA target for a representative parser call to
`native_pack8_injective`: both successful fast-path calls are inside the
single actual `parse_version` target. The 25 unselected files include
independent Token/Span parser model targets, Error/Status derived-trait and
display/description obligations, and `VersionOutcome`/`VersionResult` Clone
targets. The parser uses the open `valid_cursor` definition from
`verification_model` in its invariant; that definition is present in the
generated COMA contexts without requiring the separate Token/Span targets.
This does not close the independent `Error` `Eq`, `Debug`, `Display`, or
description trait proofs, and it does not claim proof of outer Request/Response
callers.

The exact outcome-model correction is proof-only: `parse_version_model` is
open, and the recursive short model has been replaced with the equivalent
finite seven-comparison unrolling. A fresh translation now shows both model
bodies in `parse_version.coma`; the corresponding model targets still carry
body-check obligations for their declared range and mark properties. The
model's exact outcome branches are therefore available to the parser target
without an outcome axiom.

The pack-injectivity target follows the recurrence and tail-end body checks in
the formal closure order. There is no separate parser target for a
representative pack caller: both successful fast-path calls are in
`parse_version` itself.

## Standard pack definition visibility

The translated source is the repository `creusot-libs/creusot-std/src/std/num.rs`
(the generated spans point there, not to a scratch library copy). Its
`u64_from_ne_bytes_tail` is `#[logic(open)]` and is a finite eight-branch
Horner definition, not a recursive definition. In the three COMAs for
`unfold_native_pack_tail`, `native_pack_tail_end`, and `native_pack8_injective`,
the target-specific `function u64_from_ne_bytes_tail ... = if offset = ...`
body is present. The empty Rust body of `unfold_native_pack_tail` therefore
creates a body-checking goal for the recurrence; its postcondition is not
accepted as an axiom. There is no recursive Wf/termination goal for this
finite function. `native_pack_tail_end` separately checks the offset-eight
terminal case.

The only new standard-library trust boundary is the audited, target-endian
postcondition for every input to `u64::from_ne_bytes([u8; 8])`. It does not
prove the core `transmute` body. This translation selected the little-endian
branch; the big-endian branch has not been separately compiled or verified.

## Translation fingerprints

| Input | SHA-256 |
|---|---|
| `httparse/1.10.1/src/parse_version.rs` | `5b6a1c2a8b05ebb4643b7f682bd018000b5770c5faad5e4636969e713d0bd812` |
| `httparse/1.10.1/src/iter.rs` | `369662bbf36c68ba814750ce3a91c4aa78ab40e9453af0f7e07f30d869103625` |
| `httparse/1.10.1/src/verification/model.rs` | `cf3a6f426eb55e28f61a2f5b46277639db9ff86ec0b584bf026735c59aa25cb1` |
| `httparse/1.10.1/src/verification/version.rs` | `a2a3a65047a5ec0c141eec59f0f2ea045c14454d821839022c1452071ff0fd97` |
| `httparse/1.10.1/src/status.rs` | `4fc875bc4aae6b4582a55d0c80077804011a44a4731b5274c3220ac6f38aa07d` |
| `httparse/1.10.1/src/error.rs` | `94a4af96f091a6c1f22420c9e4db4caba02f114bab710cb3588c8ed65954d8ac` |
| `httparse/1.10.1/src/macros.rs` | `e5d96c41abe995073ea30507297366657be841c70d9a5f17206dddb3c3f07404` |
| `creusot-libs/creusot-std/src/std/num.rs` | `b8b83412491707e01c46a59cb60ca5da114ada63bf0bac0bf3e52464b678929c` |
| `version-harness/src/lib.rs` | `0175c7988373656f3e70f4f8f34ef38ab3d499401fe8bfedb213cd3cfc2f68f2` |
| `version-harness/Cargo.toml` | `846e5459277e290552a1593672f83b811bc3c4497d756b11cad388d851696b4d` |
| `version-harness/verify.sh` | `be58b98644b4d2758c8ffb4628a466493ca65a67ba9f4d756121f68c9b7ef604` |
| `version-harness/why3find.json` | `a5609701e7760be9f89cada433de5274372ea89856f2e01803c619586e808aca` |

The fresh COMA target hashes in `TARGETS.tsv` were regenerated after this
translation. The harness is pinned to base commit
`75017a176124aacdb4f1e3d89110ede5979ab960` and uses the isolated
string-model compiler environment documented in the parent version-harness
README. `SHA256SUMS` fingerprints these notes and both target manifests.
