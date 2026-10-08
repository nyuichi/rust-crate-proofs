# Original public constructor gate — 2026-10-09

This bounded D-AD reopening extracts the actual `From<Vec<u8>>`,
`From<Box<[u8]>>`, `Bytes::new`, `Bytes::from_static`, `as_slice`, and
`AsRef<[u8]>` source bodies under `bytes_original_constructor_gate`. The
extracted `Bytes` record has a proof-only `Ghost<OriginalBytesProof>` sum for
`Shared`, `PromotableRaw`, and `Static`, with one validity predicate and one
byte-content model. The selected `From` contracts have no constructor-gate
precondition and preserve the complete source content.

The actual `From<Vec<u8>>` keeps both branches: `len < capacity` uses the
previously selected Shared proof component, while `len == capacity` calls
`into_boxed_slice()` and reaches the actual `From<Box<[u8]>>`. Empty boxed input
uses the actual `Bytes::new`/static route. Nonempty boxed input uses the
production even/odd pointer-tag and vtable branches. `extract_constructor.py`
extracts these source spans verbatim, checks the branch and `KIND_VEC == 1`
mapping, and records every included constructor refinement in
`generated/source-map.json`.

The nonempty Box sidecar uses `raw_vec::detach_boxed_slice`, whose generic
trusted B1-BOX boundary binds a fresh allocation namespace to the exact
consumed `Box::into_raw` allocation, with capacity equal to length, every
content slot initialized, and matching Recovery plus full PhysicalRegion. It
does not infer pointer identity from Box content. The promotable representation
also carries the bound pointer, expected sequence, read-only data binding, and
closed exact even/odd table identities. The static representation stores the
actual `'static` slice and a Std slice permission tied to its pointer and
length. Generic tag, null-pointer, and table reifications are narrow TCB
boundaries, not Bytes ownership or dispatch laws.

`original_bytes_as_slice` selects a `ReadLease` only inside ghost code. Every
representation then reaches the same ordinary native `from_raw_parts` adapter.
The Shared and boxed arms require exact B1/B4 physical authority, bounds, and
initialized bytes; the static arm requires the actual borrowed slice
permission. The returned slice refines the same common content model in every
branch. The all-domain `arbitrary_vec_from_then_read` driver calls the extracted
`From<Vec>` and `AsRef` and then copies the slice; it does not clone `Bytes` or
use a Clone invariant. The Box driver covers its empty and both alignment
branches. Separate shape drivers cover zero-length spare-capacity and
nonempty full-capacity inputs; they make no allocation-performance claim.

Two negative features mutate the selected vtable or the returned extent and
are expected to fail their corresponding validity/read refinement VC:
`negative_wrong_selected_vtable` and `negative_wrong_constructor_content`.

The probe does not verify `Bytes::clone`, automatic `Drop`, or cleanup. It does
not claim whole-crate architecture admission. Cleanup remains outside this gate
until a branch-aware capability-consuming body is available.

The native B1-BOX checks run with:

```sh
cargo test --locked --features native_b1box
```

They compare the returned pointer with `Box::as_ptr()` captured before detach,
read the contents before cleanup, cover empty and nonempty allocations, check
that two simultaneously live equal-content Boxes have distinct pointers, and
exercise the equal-capacity Vec-to-Box conversion. These tests validate native
behavior only; they are not proof evidence.

Translation only, with the pinned toolchain, runs with:

```sh
source /workspace/bytes-proof-tools/activate.sh
CARGO_NET_OFFLINE=true cargo creusot --only=coma -- --locked --features public_constructor
```

This is not a Why3 run. The complete positive proof runs with `bash run-proof.sh`.
The wrapper takes `/tmp/itoa-creusot-proof.lock`, rejects sc-drf, uses one
prover and the pinned 1024 MiB configuration, and includes every constructor
Coma file and both unrestricted From refinements. Run Why3 outside the sandbox.

## Results and limits

Positive v1: **21 files / 153 actual prover leaves / zero nulls**, no excluded
constructor target. The immutable capture is
`evidence/constructor-positive-v1-2026-10-09.tar.gz`, SHA-256
`b60ebcfb64a4b39483a02a278c63531ff3a80b2782937c5bc277486fdda71652`.
The first attempt had exactly two unproved odd-branch validity VCs; its capture
is retained separately. Ordinary UInt64 theory did not derive `address & 1 == 1`
from the native else guard. The final predicate uses the exact native
`address & 1 != 0`; no trust or ownership condition was added or removed.
Artifact labels retain 2026-10-09; these runs occurred on 2026-10-08 UTC.

The wrong-vtable control is rejected at its representation validity/invariant
VCs (two null leaves); its archive and audit are separate from the positive
capture. The wrong-extent control is also rejected (two null leaves); a fresh positive
replay after both controls passes all 21 targets. See the final capture receipt. Each failed feature is diagnostic, never positive proof.

`evidence.py capture LABEL LOG --status proved|failed|diagnostic` freezes
production and generated sources, all target tasks/proof trees, private Std,
tool provenance/configuration and logs. `evidence.py audit LABEL` independently
checks archive/member/task/proof hashes, every actual constructor/read source
span, both unrestricted refinements, target exclusions and recursive proof
statistics. Namespace freshness does not imply fresh numerical pointer values.

The complete target remains blocked by AD/AF's lost exhaustive issuance history,
the unproved promotable Clone/promotion and branch-aware cleanup integration,
automatic Drop effect lowering, and the remaining actual API. Generic TCB caller
proof does not prove its native adequacy. Constructor success does not discharge
these obligations or admit the whole-crate architecture.


The extent control fails `vc_reject_wrong_constructor_extent` (9/11); the
vtable control fails `vc_reject_wrong_selected_vtable` (4/6). Each has two null
leaves, while the restored positive configuration has none. Switching features
can leave external verification files after a cached Cargo build; the wrapper
now cleans only this package before translation, then removes dangling tasks.
The stale-feature replay was rejected and is not positive evidence.

Review `evidence/constructor-positive-final-2026-10-09.json` and its audit for
the canonical matching-source snapshot, instead of treating historical v1 as
matching later wrapper/documentation changes. Independent source reconstruction
and private-Std audit are recorded in `INDEPENDENT_AUDIT.json`.
