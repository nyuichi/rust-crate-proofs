# Modified bytes 1.11.1 configuration evidence

Full configuration coverage is not complete. Each entry names its actual source
and configuration; later API additions require their own retained-configuration
integration. Native cross compilation is not native execution on that platform.

The committed combined-leaves-183 source (c12e6aa6) passed the host
`verified,std` proof: 183 files, zero unresolved leaves, 19 native tests.
Its identical Rust source also passed native library cross-checks for
`i686-unknown-linux-gnu` (32-bit little endian) and
`powerpc64-unknown-linux-gnu` (64-bit big endian). Those native checks
compile only. Both targets were installed into the pinned nightly tool environment.

The powerpc64 configured proof completed 183 files, exit 0, zero unresolved
leaves. The archive records the target, actual dependency feature tree, native
check log, complete proof log and exact Why3find settings. This is a proof of
that source/configuration under the same physical/atomic/library TCB; it is not
a runtime test or a claim for every architecture, alignment or 16-bit target.
The i686 proof also completed 183 files and was recaptured after an initial
capture argument error. Both complete archives were independently audited.
Their exact hashes follow below.

The wrapper now passes target arguments to `cargo clean --package bytes` as
well as translation. Without this, a repeated cross-target run can reuse Cargo
translation cache after deleting verif and produce no fresh Coma files. The
initial missing temporary-copy Why3find settings and this cache obstruction
were environment/wrapper failures, not failed proof obligations.

An alloc-only no_std candidate passed separately (152 files): bytes std=false in
native and proof builds; native creusot-std/std=false; proof enables only the
existing creusot-std/std contract definitions for the same alloc::Vec type.
Its native no-default-features verified and verified,std library checks pass.
Its 152-file engine result, exact source and both feature graphs are archived.
The candidate
is not yet adopted into production, and the proof-only dependency graph is
not asserted to be a no_std-linkable artifact. Exact source/type/module and
feature-graph correspondence remains required before admission.

Serde, portable-atomic, failure-aware cleanup/allocation and total termination
remain distinct obligations. No bytes-specific ownership/refcount theorem is
trusted to bypass them.

## Independently audited archives

- `combined-leaves-183-i686`: `95753508f082a4eff201de416d96eb50a685d9c693d91ffc2fd79425d5a82252`.
- `combined-leaves-183-powerpc64`: `dfc333c7423e2495c7892f08fe67677acc147f3d5a20d024fa004b8f06134849`.
- `alloc-only-candidate-152`: `724693b94d05f5dcdd178c5b9b5ba716d988bb380d20408387ba33acbbd7889f`.

## Capacity interface experiments

A local generic Vec metadata observer with strengthened standard contracts was
attempted on the same 183 source. The first frontend diagnostic required the
Allocator type parameter; correcting it exposed duplicate existing extern specs.
Both exact candidates and logs are preserved, with no successful body claim.
Astra reviewed a minimal replacement of the existing standard Vec contracts as
the next step. That would extend the generic standard-library TCB explicitly,
not prove the standard implementation or trust a bytes-specific ownership law.
The installed tool/library source has not yet been changed.

## Updated alloc-only candidate (255-source API core)

The exact adapters-public-spec-255 source was copied into an isolated candidate.
Alloc APIs remain at the verified root; the std sharing implementation and its
children move behind a scoped module and retain public reexports. Cargo no
longer forces bytes/std when verified is enabled; creusot-std default features
are disabled, with bytes/std explicitly enabling its std contracts. A scoped
write lifecycle unit test is correctly gated on std; core tests remain enabled.

Native no-default-features verified library check passes, with actual feature
lists bytes=[verified], creusot-std=[]. Fourteen core tests pass. The same
candidate's verified,std library check and all 32 native tests also pass.
The configured alloc-only proof completes 221 Coma/proof JSON files, exit 0,
zero null leaves. Root independently audited all 496 archive members:
alloc-only-candidate-221,
`916e76b2e9ef4452bb45c7b464ba8a20ab948595634f28b3a6db1ce319c00df3`.

The proof deliberately enables creusot-std/std for the existing alloc::Vec
contracts while bytes/std remains false. Actual dependency feature lists are
captured, including implicit creusot/nightly. Std reexports alloc::Vec and these
are the same canonical Rust type; the runtime branch and selected bytes core
source remain the same. This correspondence supports the core body proof, not
a claim that the proof dependency graph is a no_std-linkable binary. This is
still a candidate, pending the relocated std graph proof and final production
adoption. Later public-math and capacity changes require an updated integration;
the 221 count is not the complete production API count.
