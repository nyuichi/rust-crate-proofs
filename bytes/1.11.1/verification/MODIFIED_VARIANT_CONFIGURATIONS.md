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

The same candidate's relocated std graph subsequently completes 255 proof
files, exit 0, zero null leaves, with 32 native tests. Root independently checks
its exact source/configuration archive, relocated-scoped-candidate-255:
`7e6839878d1a9a0eb929ea874f838dd9206b0a120b6b74cff30c4238ce3dedd3`.
This proves the module relocation preserves the full saved 255-source graph;
production adoption and composition with later public-math/capacity changes
remain separate steps.

The same candidate's relocated std graph then passes the complete configured
proof: 255 Coma/proof JSON files, zero null leaves, engine exit 0. Archive
relocated-scoped-candidate-255:
`7e6839878d1a9a0eb929ea874f838dd9206b0a120b6b74cff30c4238ce3dedd3`.
Root independently audited all captured members. This establishes the actual
sharing protocol's composition after relocation; it does not combine a disconnected
core model with a different std representation. Production adoption is pending
the concurrent public-math source gate's capture; its source must be preserved.

## Production alloc core adoption (current public read models)

The scoped relocation is now applied to the actual production source, retaining
the public read models introduced in 04fa51a1. Its alloc runtime core passes
221 proof files, exit 0, zero null leaves, and 14 native tests. Native std
tests also pass (32); the production std proof is queued separately and is not
claimed by this core result. Original default and no-default library compilation
also pass, without reopening the frozen original frontend proof.

The proof feature list is verified,creusot-std/std: bytes/std is disabled but the
stock proof dependency still enables std to expose alloc models. This is not
a genuinely alloc-only proof dependency graph. A small isolated alloc-model
exposure experiment, including actual 16-bit translation, remains in progress.
Production evidence production-alloc-core-221 is independently hash/proof audited:
`9d12b34ab533b7d95612e4782a33905b0034077b50ee287df836f1bde9afff65`.

## Actual 16-bit stock-model prerequisite failure

The isolated 255-source alloc runtime compiles for msp430-none-elf with
-Zbuild-std=core,alloc. The stock proof dependency requires std for Vec models,
and the proof translation reports missing std before producing Coma files. The
run was explicitly interrupted after that fatal prerequisite diagnostic; its
record is not a completed VC failure. A separate attempt to build actual std
fails because the target lacks alloc::sync::Arc (no pointer atomics).
The exact logs/source/status are independently archived in
alloc-only-msp430-std-model-frontier-255, SHA256
`6c3d7c123f716cca65955cf07cd8b53342738c74cc0fb5530e4214dfcfa4e184`.

This does not reject 16-bit arithmetic. A small cfg/import/feature port of
existing alloc contracts has Astra review and fresh actual UInt16 translation
in an isolated candidate; integrated proof is still pending. No backend changes,
new semantic axioms or atomic emulation are authorized by that experiment.
