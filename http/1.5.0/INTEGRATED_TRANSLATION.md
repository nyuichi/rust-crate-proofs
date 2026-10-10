# Integrated Creusot translation attempts

Translation was attempted for the complete production crate with the ordinary
module graph, retaining every runtime implementation. No `cfg(creusot)`
substitute module, trusted HTTP runtime body, or API exclusion was used.

## Default features

`./verify-all.bash` was run. It invokes `scripts/run-proof.sh` for each Cargo
command, using the shared Why3 lock, one prover, and a 1024 MiB limit. The
default-feature translation stopped before VC generation with exit status
101. The first diagnostic is:

```text
error: support for trait objects (dyn) is limited and experimental
   --> src/convert.rs:8:24
   ::: src/header/value.rs:167:9
   = note: the lint level is defined here
   --> src/lib.rs:156:9
   = note: `#[deny(creusot::experimental)]` implied by `#[deny(warnings)]`
```

The initial expansion is `if_downcast_into!` in `HeaderValue::from_maybe_shared`;
the macro casts a generic value to `&mut dyn std::any::Any`. The same
translation diagnostic appears at the other URI and header macro expansions,
then in `Error` and `Extensions` where production code projects `dyn Error`,
`dyn Any`, and `dyn AnyClone`. The complete command output is preserved in
[`verification/integrated-creusot.log`](verification/integrated-creusot.log).

After reporting 22 translation errors, rustc also panicked at
`creusot/src/translation/traits.rs:356:17`:

```text
not implemented: Cannot handle builtin implementation of `std::clone::Clone` for `()`
```

The temporary rustc ICE dump was removed after recording the complete command
log; no integrated crate VC proof was produced. Read-only tool triage
confirmed the independent source-shaped probes in
[`TOOL_BLOCKERS.md`](TOOL_BLOCKERS.md) fail in Creusot's translator for
`dyn Any` / `dyn AnyClone` even without the crate's `deny(warnings)` lint.
Removing or suppressing the lint would not add sound dynamic-type support.
The later `Clone for ()` ICE is a separate translator capability blocker
during purity validation; the trace does not identify an HTTP source location
for its trigger, so it is not attributed to a particular derive. No VC was
generated or discharged before either blocker.

A future tool fix should model the exact built-in `Clone` behavior for `()`
(unit result and no effects), preserve normal behavior for arbitrary user
`Clone` implementations, and report unsupported built-ins as diagnostics
instead of panicking. A minimal follow-up reproducer is a direct
`<() as Clone>::clone` call, followed by a derived struct with a unit field
and a generic instance.

## All features

The separate all-features attempt also exited 101 with the same first
diagnostic, 22 translation errors, and the same `Clone for ()` ICE. The crate's
only feature is `std`, so `--all-features` selects the same production feature
set as default. Its complete output is
[`verification/integrated-creusot-all-features.log`](verification/integrated-creusot-all-features.log).

These failures are not VC failures and do not count as integrated verification.
The runtime matrix and accepted source-leaf evidence are recorded separately
in [`PROVENANCE.md`](PROVENANCE.md). The independently reproduced `dyn Any`,
`AnyClone`, `TypeId`, and raw-pointer gates are detailed in
[`TOOL_BLOCKERS.md`](TOOL_BLOCKERS.md).
