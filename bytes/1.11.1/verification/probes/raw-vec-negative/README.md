# Raw Vec ownership negative probes

This probe imports the current production B2/B3/B4 interfaces directly from
`src/ownership_proof/owned_region.rs` and `src/ownership_proof/raw_vec.rs`.
Each feature omits one required premise, then calls the actual interface. These
are verifier-level contract checks over abstract sealed values; they do not
construct allocator state or execute an invalid resume, borrow, or deallocation.
The trusted contracts remain the boundary under test.

## Observed verifier failures

| Feature | Function / goal | Missing premise | Expected result |
| --- | --- | --- | --- |
| `wrong_half_region` | `wrong_half_region_recovery` | B2 region covers only `[0, 1)` while the allocation capacity is at least two. | `✘ (13/14)`: one split leaf rejected for full allocation coverage. |
| `wrong_unknown_prefix` | `wrong_unknown_prefix_resume` | B2 requested prefix slot zero is `Unknown`, not `Known(value)`. | `✘ (13/14)`: one split leaf rejected for initialized-prefix knowledge. |
| `wrong_namespace` | `wrong_namespace_resume` | B2 caps and region carry a namespace unrelated to the raw descriptor. | `✘ (12/14)`: two split leaves rejected for raw/recovery and raw/region namespace matches. |
| `wrong_unknown_borrow` | `wrong_unknown_borrow` | B4 requested slot zero is `Unknown`. | `✘ (22/23)`: the `Known`-slot leaf is rejected. |
| `wrong_out_of_region_borrow` | `wrong_out_of_region_borrow` | A one-byte borrow starts at the region's exclusive endpoint. Its requested slot is also outside the ledger domain (`None`). | `✘ (21/23)`: range-bound and `Known`-slot leaves are rejected. |
| `wrong_stale_value_after_mutation` | `wrong_stale_value_after_mutation` | The probe asserts the pre-borrow byte value after writing a different byte through the returned mutable slice. | `✘ (16/17)`: the stale-value assertion is rejected. |
| `wrong_half_deallocate` | `wrong_half_deallocate` | B3 region covers only `[0, 1)` of an allocation whose capacity is at least two. | `✘ (11/12)`: one leaf rejected for full allocation coverage. |
| `wrong_namespace_deallocate` | `wrong_namespace_deallocate` | Recovery belongs to another namespace; the region still has matching namespace and full coverage. | `✘ (11/12)`: one leaf rejected for raw/recovery namespace match. |

The functions quantify over sealed descriptors/capabilities through their
preconditions. They do not fabricate `RawAllocation`, `Recovery`, or
`PhysicalRegion` values. In particular, the proof failures show which stated
premises the production contracts demand; they are not evidence that Rust's
allocator or raw-pointer implementation is safe.

## Current evidence status

The latest replay passed all eight isolated features against one source
snapshot. Each run reached the intended named VC, rejected the intended premise
or assertion, and emitted the corresponding `.coma` and `proof.json`; the
runner reported no compiler errors. The `x/y` figures in the table count the
split proof leaves for that one negative VC, not a global proof percentage.
Per-feature source/config/script snapshots and generated proof outputs are
archived under `verification/artifacts/evidence/raw-vec-negative/<feature>/`;
only `.coma` and `proof.json` outputs are retained in `proof-artifacts/`, while
logs remain under `verification/probes/raw-vec-negative/logs/`. The current
per-run source snapshot and artifact hashes are in
`verification/artifacts/evidence/raw-vec-negative-manifest.json`. The hashes
refer to the source copies used for the proof run; the workspace source may
continue to evolve independently.

The separate native move checks also passed: both fixtures emitted exactly
`E0382`. They were checked, not run. These checks cover Rust move behavior
only; they do not establish ghost separation or the trusted contracts' soundness.

Earlier E0690 and `BoundPtr` translation failures remain in `logs/diagnostics/`
as historical diagnostics. They do not describe the source used by the latest
successful replay. The initial B4 Rust ghost-plumbing compile failures are also
retained there for audit history.

## Replay

From the repository root, replay all eight cases sequentially:

```sh
python3 verification/probes/raw-vec-negative/scripts/check-negative.py
```

A subset can be selected by feature name. Each replay uses the root
`verify-bytes.sh raw-vec-negative` route with one feature at a time. The script
checks that the exact expected VC is rejected, no Rust/translation compiler
error occurred, and the function's `.coma` and `proof.json` were emitted. It
archives the explicit source/config/script snapshot and only the generated
`.coma` and `proof.json` artifacts under
`verification/artifacts/evidence/raw-vec-negative/<feature>/`.

## Rust move checks

`affine-fixtures/` contains two separate compile-fail fixtures:

- `duplicate_descriptor` moves a `RawAllocation` and then tries to use it again.
- `use_after_transfer` consumes a `Ghost<(Recovery, PhysicalRegion)>` with
  `.split()` and then tries to use the original wrapper again.

Replay those checks with:

```sh
python3 verification/probes/raw-vec-negative/scripts/check-affine.py
```

The script activates the repository's pinned `nightly-2026-06-22` environment,
runs `cargo check --offline --locked` for each feature, and expects exactly
`E0382`. The fixtures are never executed. These native Rust checks establish
only the ordinary move/non-`Copy` behavior of the wrapper types; they do not
verify ghost separation, ledger invariants, permissions, or any Creusot proof.
