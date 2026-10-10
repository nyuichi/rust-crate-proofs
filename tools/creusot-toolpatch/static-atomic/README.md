# Static `AtomicU8` Gate A audit

This target-local Creusot patch adds an opt-in audit for private immutable
`static` values of the resolved sysroot `std::sync::atomic::AtomicU8` type. Gate
A records and rejects Rust MIR shapes. It does not add a static resource, atomic
history rule, cache-value invariant VC, or proof that a registered predicate is
true. It does not trust or prove any `httparse` dispatch result.

## Accepted MIR shape

Each manifest registration must resolve to exactly one local immutable,
non-exported `AtomicU8` static initialized by one direct
`AtomicU8::new(const_u8)` call. In executable bodies that reference a registered
static, Gate A accepts only direct `AtomicU8::load` and `AtomicU8::store` calls
with `Ordering::Relaxed`. It tracks the static identity through simple local
copy/move and shared-reference reborrow MIR. It rejects raw pointer conversion,
return or capture of a reference, aggregates, helper/generic/indirect calls,
ambiguous static joins, alias reassignment, other atomic methods, and unresolved
or non-Relaxed orderings. The accepted operation body is still only inventoried;
Gate B must establish the value invariant and atomic semantics.

Proof-cfg mode additionally resolves the manifest predicate and checks that it
is a checked, nonprophetic, nontrusted logical free function with exact
signature `fn(u8) -> bool`. This is a shape check only. The predicate body is not
proved or used as an assumption by this patch.

## Modes and manifest

The new `creusot-rustc` driver path is selected with these environment values:

```text
CREUSOT_STATIC_ATOMIC_MODE=normal-audit|proof-audit
CREUSOT_STATIC_ATOMIC_MANIFEST=/absolute/path/to/manifest.tsv
CREUSOT_STATIC_ATOMIC_OUTPUT=/absolute/path/to/output.audit
CREUSOT_STATIC_ATOMIC_SOURCE_SHA256=<64 lowercase hex digits>
```

The version 2 UTF-8 manifest is tab separated:

```text
creusot-static-atomic-v2
crate\t<exact rustc crate name>
manifest_dir\t<exact Cargo package directory>
target\t<exact rustc target triple>
source_sha256\t<same digest supplied in the environment>
run_id\t<fresh 32-byte random value, lowercase hex>
normal_input_sha256\t<normal-mode input/context digest>
proof_input_sha256\t<proof-mode input/context digest>
compiler_sha256\t<target-local creusot-rustc binary digest>
patch_sha256\t<Gate A patch digest>
fixture_sha256\t<checked-in fixture source digest>
runner_sha256\t<fixture runner digest>
static\t<local DefPath without the crate prefix>\t<predicate DefPath>
```

The compiler checks the crate name, canonical package directory, target triple,
and that the digest field agrees with the digest supplied by the driver. It does
not independently compute source, compiler, patch, fixture, or runner digests.
The fixture runner computes those hashes independently, checks them before and
after each compiler invocation, and validates that every report carries the
same run id and context. Each fixture is compiled from a frozen copy whose hash
must match the checked-in fixture. The runner detects copied reports from an
earlier run; these metadata fields are not a cryptographic signature against an
adversary who can rewrite both a report and its manifest. A target integration
must independently hash its normal/proof source inputs and toolchain artifacts,
preserve the same source snapshot, and compare the emitted `access` entries and
`begin_mir`/`end_mir` sections before using this as cfg-equivalence evidence.

`normal-audit` retains ordinary cfg, routes only the named target through
`NormalAuditCallbacks`, and compiles other crates with the default callbacks.
`proof-audit` enables Creusot's proof cfg and captures target MIR in either the
direct audit callback or the existing `ToWhy`/`WithoutContracts` callbacks.
Both modes write registrations and access records; the MIR included for each
accessing body is rustc's built MIR before optimization.

## Build and fixtures

The patch applies after `creusot-narrowcast-backend.patch` on pinned Creusot
baseline `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc`. The isolated source import
used for this check has local Git commit
`e81a5f61311dffaf8c38f970b2460662d6efdc7b` for that baseline. Toolchain:
`nightly-2026-02-27` with `rustc-dev` and `rust-src`.

```sh
source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_TARGET_DIR=/workspace/proof-tools/targets/httparse-static-atomic-gate-a
cargo build --offline --manifest-path Cargo.toml -p creusot-rustc
```

From the repository root, run the direct compiler fixtures with:

```sh
tools/creusot-toolpatch/static-atomic/run-gate-a-fixtures.sh
```

Set `CREUSOT_STATIC_ATOMIC_RUSTC` to use a different isolated compiler binary.
The runner checks positive normal/proof MIR identity (including two distinct
function-local statics), deliberate cfg drift, package/target/digest mismatch,
and negative alias/ordering/initializer cases. It invokes no solver.

Patch SHA-256: `e075f03f479f7efab0967345e2a25e84d063c0a8357ece399f097ece17d2e044`.

## Gate B still required

This audit does not implement the planned pure value invariant
`I(v) = predicate(v)`. A sound Gate B must add compiler obligations for the
initializer value, nondeterministic load values satisfying `I`, and every
reachable store value satisfying `I`, while establishing that normal and proof
cfg execute identical bodies. It must keep all static identities distinct and
must not mint, duplicate, reset, or reopen exclusive history resources. No
Creusot `Static` translation semantics or standard-library atomic history TCB
is extended here. The standard library's implementation/atomic contract and
Rust static initialization semantics remain outside this Gate A audit.
