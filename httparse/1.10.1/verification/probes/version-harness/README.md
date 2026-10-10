# `parse_version` verification checkpoint

## Shared source and model

The production crate includes `src/parse_version.rs` from `src/lib.rs`. This
harness imports that same file, the actual `Bytes` implementation in
`src/iter.rs`, the actual macros, `Status`, and `Error`. It does not substitute
the parser or its result types. The independent model is
`src/verification/version.rs`.

The model carries the absolute `input`, `mark`, `cursor`, and `end` values.
For inputs with at least eight bytes left, it returns version 0 for
`HTTP/1.0`, version 1 for `HTTP/1.1`, or `Error::Version`; every result
consumes exactly eight bytes. For shorter inputs, it consumes matching bytes
from `HTTP/1.` one by one. A mismatching byte is consumed before the error,
EOF returns `Partial` at the end, and seven matching bytes return `Partial` at
`start + 7`. `mark`, `input`, and `end` are preserved.

The source body was extracted from base commit `75017a176124aacdb4f1e3d89110ede5979ab960`.
The control flow, three `u64::from_ne_bytes` calls, call order, and eight-byte
advance location are preserved. Two local `[u8; 8]` arrays spell the version
bytes explicitly because the isolated translator ICEs on byte-string literals
inside proof snapshots. The two proof-only calls to the generic packing
injectivity lemma are under `cfg(creusot)` and are erased from normal builds.
No parser behavior or version-specific axiom was added.

## Standard-library boundary

The active compiler is `rustc 1.95.0-nightly (6a979b3e32522049d0acb4a47f7ae44b7c8abfd5)`,
with Rust source file
`/workspace/proof-tools/rustup/toolchains/nightly-2026-02-27-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs`
(SHA-256 `189d28513c634d88df6a365d3a509ec7aade0d7eeec5ddb3976f22e91ae6e572`).
The `u64` implementation at lines 3989–4030 documents native-endian array
interpretation and implements `from_ne_bytes` as
`unsafe { mem::transmute(bytes) }`. The audited safety premise is that Rust
`u64` is an eight-byte integer with no invalid bit patterns; this contract is
for every `[u8; 8]` input to `u64::from_ne_bytes`, not only the three parser
call sites.

`creusot-libs/creusot-std/src/std/num.rs` adds the one explicit standard-library
trust boundary: the conversion result equals a finite Horner expression over
the bytes in target native-endian order. The little-endian and big-endian
branches use `target_endian`; compilation fails if target endianness is
unknown. The finite open logic definition and the generic byte-packing lemmas
are separate from the trusted conversion contract. No generic `transmute`
contract, parser axiom, or `HTTP/1.0` / `HTTP/1.1` equality axiom is added.

The body-checked arithmetic chain peels eight base-256 digits from equal packed
values. The parser calls that generic lemma only in the two successful fast
path branches. The general `from_ne_bytes` contract supplies its hypotheses;
the model and byte comparisons remain body-checked. The current translation
was produced for the little-endian host. The big-endian source branch is
present but has not been separately compiled or proved.

The parser's exact outcome model is visible in the current translation.
`parse_version_model` is open, and the short-input model is an open finite
unrolling of its seven comparisons rather than a recursive function. The
generated parser COMA includes both model bodies, so branch results refine the
actual `Http10`, `Http11`, `Partial`, and `Error` outcomes. Their own model
obligations remain body-checked; no outcome axiom was added.

## Direct proof checkpoint

`verify.sh translate` uses the isolated string-model compiler environment so
the harness includes the actual `Error` source without the normal compiler's
string-literal ICE. It translated 63 `.coma` files, including the actual
`parse_version` body, exact outcome model, generic packing lemmas, and actual
`Bytes` dependencies. The generated `parse_version.coma` contains all three
actual `from_ne_bytes` calls and the explicit result contract.

Fresh direct Why3 runs completed for the selected closure. They used the
isolated Why3 package/config and `Z3,4.15.3`, with one prover, 1000 MiB, and a
30-second per-goal limit. Each batch ran type-only checks before proof through
`httparse/1.10.1/run-proof.bash` and the read-only
`spaces-harness/proof-child.sh` runner. The result logs and frozen input
hashes are in these evidence directories:

| Bundle | Scope | Direct Why3 result |
|---|---|---|
| `evidence/direct-why3-helpers-20261005T060000Z` and `evidence/direct-why3-model-20261005T055500Z` | 6 arithmetic/model targets | 61/61 Valid leaves |
| `evidence/direct-why3-bytes-20261005T060100Z` | 29 actual `Bytes`/iterator targets | 140/140 Valid leaves |
| `evidence/direct-why3-parse-version-20261005T060300Z` | actual `parse_version` body | 62/62 Valid leaves |
| `evidence/direct-why3-array8-20261005T055700Z` | external `array8_prefix` caller needed by `peek_array8` | 7/7 Valid leaves |

The selected 38-target main closure therefore has 263 Valid leaves across 36
nontrivial targets. Two open logic definitions simplify to literal `true` and
create no solver task; they are recorded in
`evidence/direct-why3-helpers-20261005T060000Z/TRIVIAL_GOALS.tsv` and are not
counted as Valid leaves. The external `array8_prefix` result is reported
separately. The earlier active-byte snapshot used a different COMA tree for
the same `src/iter.rs` source and recorded 139 recursive leaves; this fresh
direct version-harness run reports 140. The array8 COMA hash is unchanged from
its earlier snapshot, which recorded 9 recursive leaves; the fresh direct run
reports 7. These are separate splitting counts, and the fresh direct Why3
outputs above are the result of this checkpoint.

### Reproduce the translation and direct proof batches

To force a fresh package translation rather than reuse Cargo's cached
artifacts, run from the repository checkout:

```bash
source /workspace/proof-tools/activate.sh
source /workspace/scratch/httparse-string-model/creusot-env.sh
cd /workspace/rust-crate-proofs/httparse/1.10.1/verification/probes/version-harness
cargo clean -p httparse-version-harness
cargo creusot --simple-triggers=false
```

`verify.sh translate` performs the same environment setup and invokes the same
`cargo creusot` command, but does not clean the package first.

For direct replay of the frozen COMAs, each batch's `targets.txt` names the
COMA targets and `run-identity.txt` records the isolated config, package,
compiler, Z3, and resource limits. The commands below create new temporary
logs and reuse the checked-in COMA tree without overwriting frozen evidence:

```bash
set -euo pipefail
repo=/workspace/rust-crate-proofs
crate=$repo/httparse/1.10.1
probes=$crate/verification/probes
harness=$probes/version-harness
child=$probes/spaces-harness/proof-child.sh

replay() {
  local name=$1 source=$2 targets_file=$3 first=${4:-1} last=${5:-999}
  local run
  run=$(mktemp -d "/tmp/httparse-version-${name}.XXXXXX")
  mkdir -p "$run/context" "$run/logs"
  ln -s "$source" "$run/context/string"
  mapfile -t targets < <(sed -n "${first},${last}p" "$targets_file")
  "$crate/run-proof.bash" bash "$child" "$run/context" "$run" "${targets[@]}"
  printf 'Replay logs: %s\n' "$run"
}

evidence=$harness/evidence
replay helpers "$harness" "$evidence/direct-why3-helpers-20261005T060000Z/targets.txt" 1 4
replay models "$harness" "$evidence/direct-why3-model-20261005T055500Z/targets.txt"
replay bytes "$harness" "$evidence/direct-why3-bytes-20261005T060100Z/targets.txt"
replay parser "$harness" "$evidence/direct-why3-parse-version-20261005T060300Z/targets.txt"
replay array8 "$probes/memory-array-conversion" "$evidence/direct-why3-array8-20261005T055700Z/targets.txt"
```

The helper `targets.txt` also lists `version_prefix_byte` and
`matches_version_bytes`; both COMAs reduce to literal `true`, so they produce
no solver task and are intentionally excluded from the helper replay and
Valid leaf totals. Their hashes and classifications are recorded in
`direct-why3-helpers-20261005T060000Z/TRIVIAL_GOALS.tsv`. The model replay
contains the nontrivial short and full outcome models. `array8_prefix` is
replayed from its separate memory-array-conversion COMA tree.

`evidence/selected-targets-20261005/COMA-TREE-SHA256SUMS` fingerprints all 63
translated COMAs. `SOURCE-INPUTS-SHA256SUMS` fingerprints the 29 Rust files
referenced by their source spans and source headers. Verify them from the
repository root and harness directory respectively:

```bash
cd /workspace/rust-crate-proofs
sha256sum -c httparse/1.10.1/verification/probes/version-harness/evidence/selected-targets-20261005/SOURCE-INPUTS-SHA256SUMS
cd httparse/1.10.1/verification/probes/version-harness
sha256sum -c evidence/selected-targets-20261005/COMA-TREE-SHA256SUMS
```

The proof covers the actual extracted helper body and its selected `Bytes`
dependencies. The outer `Request::parse` and `Response::parse` callers, the
other parser bodies, and full-crate integration remain open. The standard
`u64::from_ne_bytes([u8; 8])` conversion contract is trusted for every input
array; its Rust `transmute` implementation is not body-proved here. This
translation and proof select the little-endian branch. The big-endian branch
and target configuration have not been separately compiled or proved.

Existing crate tests passed after extraction and literal-array adjustment:

- Default features: 101 unit tests, 263 URI tests, and 6 doc tests passed.
- `--no-default-features`: 97 unit tests, 263 URI tests, and 6 doc tests passed.

No runtime change was made during the direct proof checkpoint. Request and
Response caller proofs remain separate.

## Snapshot fingerprints

The source and selected translation target hashes for this little-endian
translation are:

| File | SHA-256 |
|---|---|
| `src/parse_version.rs` | `5b6a1c2a8b05ebb4643b7f682bd018000b5770c5faad5e4636969e713d0bd812` |
| `src/verification/version.rs` | `a2a3a65047a5ec0c141eec59f0f2ea045c14454d821839022c1452071ff0fd97` |
| `src/lib.rs` | `60be3d56fc440a1b76c39fb11d5e367e4d3e67a6c26d5f3f645e742143ee60a2` |
| `creusot-libs/creusot-std/src/std/num.rs` | `b8b83412491707e01c46a59cb60ca5da114ada63bf0bac0bf3e52464b678929c` |
| `verif/httparse_version_harness_rlib/parse_version.coma` | `4c93488fa24e4618276210c9e15b5de3c20c32195776f022fbeb249dee60e168` |
| `verif/httparse_version_harness_rlib/verification_version/short_version_model.coma` | `5c51a48b2491c266a53c7b6686dbd4fdc8a661bf29ef28d730bee3b340550610` |
| `verif/httparse_version_harness_rlib/verification_version/parse_version_model.coma` | `aa0a196d1e0a7648235e91015c4580ec46ccf9af62b7c001800265c02e47bc9b` |
| `verif/httparse_version_harness_rlib/verification_version/native_pack8_injective.coma` | `9c04dfb9e383d7b40da7148156003e2849506f1e7c6b3fa2eb7584b520364d66` |
| `verif/httparse_version_harness_rlib/verification_version/base256_head_injective.coma` | `37a5b73681442ce6530320d8429840a6fccd57fbfb3d32ed506809c8899991d4` |

These `.coma` files were emitted after cleaning the version-harness Cargo
target. The Rust source hash and toolchain are listed above. The repository
base commit for the extracted parser is
`75017a176124aacdb4f1e3d89110ede5979ab960`.
