# AU canonical archive independent audit

## Result

**PASS for the bounded AU admission by exact proof reuse.** The archive
`au-positive-reuse-canonical-v1.tar.gz` has SHA-256
`0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701`.
Its manifest SHA-256 is
`7b36a3e407207433d1ae640e5439c38f519df1e9edd92b40ecf581c4a4fd5ef1`.
All 1,670 archive members are safe, unique regular files whose bytes match the
manifest.

The archive binds all 157 COMA/proof pairs: 1,447 prover leaves, zero null
leaves, zero structural leaves, no exclusions, and no enabled features. The
target policy retains the diagnostic flag and checker exit status 2 from the
original proof run. A separate current correspondence record passes with exit
status 0.

## Reused proofs and current admission

I compared every current COMA and proof file against both the embedded origin
archive and the current AU worktree. All 314 hashes match the identity record
and the origin receipt. The origin archive
`au-full-diagnostic-v1.tar.gz` has SHA-256
`ffc00ffe50a408f971f2a56e1d58f7173d3d9856ebeb65e97c7a725d9e9393bd`; its
receipt SHA-256 is
`6a57cba12daee897e79a4214e8627ba7ecc933133545abd8e742737a3c7956ee`.
The origin run log contains `Proved (157 files)`. Its correspondence was
skipped (exit status 2), so the origin remains diagnostic. The archive does
not record the prover process exit status, and this admission claims no second
prover run.

I reconstructed the AU and captured ancestry paths using only bytes in the
canonical archive, then independently ran:

```text
python3 check_correspondence.py --audit-compiled-capture-only
```

The checker passed with scope “AU complete source/native/compiled capture
audit; no live Cargo target.” It checked 31 selected MIR bodies, including 29
production bodies, and bound the native `clone_suffix` call to its separately
body-proved `clone_suffix_checked` summary and caller proof.

The fresh structural control, `refreshed_native_callee_identity`, rejected the
forged native callee as expected. Its receipt has SHA-256
`0507783731a2215c0d0ef8c23d94596332b6b5d1352ae35363d9f5cb57da8b74`; it
records one expected rejection and no accepted controls or checker errors.

## Cargo artifacts and external tools

The archive contains four actual Cargo build artifacts plus their receipt. The
checker verified their fingerprint inputs, build output, OUT_DIR/root-output
join, and reconstructed public records. The captured artifact hashes are:

| Artifact | SHA-256 |
| --- | --- |
| `public_records.rs` | `41eeb72bd7a4e3b59f042311508a436033c1a01e227035beb955bec749a8169b` |
| `cargo-run-build-fingerprint.json` | `6f88ebcbeb029082dc15d150facc148edaffcb9cec90848f827ed550a50349d5` |
| `cargo-build-output.txt` | `0cab1e3220b72008729103a994de3b490da5f9fd8e7148efd9f8dd5f9f508344` |
| `cargo-root-output.txt` | `61839eed3d157111f6864c0bab5c4cfd1a19b7b1151622ec8d155d2cd2a36224` |

The receipt records absolute paths under
`/workspace/bytes-proof-tools/targets/bytes`. This is location-bound build
provenance. The independent replay used only the captured artifacts and did
not read the live Cargo target directory.

All 61 captured production source files, three Cargo manifests, 110 private
Creusot Std files, and seven tool/config inputs match the current inputs. I
also hashed the eight installed external executables; all match the archived
installation manifest. Executable payloads are not bundled in the archive.
Four tools differ from an earlier bootstrap manifest, as the current
installation manifest records; none differ from the manifest captured here.

The captured configuration specifies a 1,024 MiB memory limit, one concurrent
prover, a five-second time limit, offline Cargo, and sc-drf disabled. This
audit did not rerun a proof.

## Published AT ancestry and reused controls

The complete AT canonical v2 archive is embedded with SHA-256
`958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d`, 1,544
members, and proof statistics 155/1,417/0/0. Its checker and source tree are
present. AU's archive-only checker replay also replays the AT source, native,
compiled-capture, and AS/AR/AQ/AP ancestry gates.

AU carries the published AT control receipts rather than rerunning those
controls: the main source checker receipt records 52/52 expected rejections;
the native receipt records 45/45. The main receipt SHA is
`f6ac4bdcd2811893bdecb3614b240cfe6bab06e268328ce56ae8501f872b8873`; the
native receipt SHA is
`9591a9416a2a19ec23d48235c8f9a7f4acffa03086777872b351f3166a6d636a`.

## Archive-only restore layout

The archive stores inputs under evidence prefixes; the checker uses
repository-relative sibling paths. I reconstructed them without outside
inputs:

1. Restore `inputs/repository/bytes/1.11.1` as the crate root and its captured
probe trees under `bytes/1.11.1/verification/probes`.
2. Place AU's `probe/` files at the AU probe path. Copy
   `inputs/au-proof-origin/au-full-diagnostic-v1.tar.gz` and `.json` to AU's
   `evidence/` directory, which is where AU's checker reads them.
3. Restore AT from the embedded AT v2 archive's `probe/` members and the
   separately embedded AT audit/receipt files. Restore AS, AP, AQ, and AR from
   their embedded canonical archives. For historical ancestry comparisons,
   materialize the captured repository snapshot from AT v2's
   `inputs/repository/` members.
4. Run AU's `check_correspondence.py --audit-compiled-capture-only` from the
   restored AU path. This uses the archived Cargo capture and never invokes
   Cargo or a prover.

## Scope

This result covers one selected modular borrowed-input/owning-return client
with a named, body-proved Clone/advance summary. It does not admit general
Clone or From, arbitrary call graphs or escaping owners, concurrency, unwind,
or the full `bytes` crate. The toolchain remains external to the archive, and
the original prover process exit status is not recorded in the archive.
