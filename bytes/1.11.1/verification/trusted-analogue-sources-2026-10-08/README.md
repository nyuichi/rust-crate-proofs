# Stock synchronization and resource analogues

This directory preserves exact source copies used for the 2026-10-08 generic
trusted-boundary design review. It contains the upstream Creusot mutex example
and the active vendored `creusot-std` 0.13.0 sources for invariants, `FnGhost`,
resource algebras, atomics, committers, and synchronization views. `manifest.json`
records each source origin and SHA-256.

## What these examples establish

`tests/should_succeed/mutex.rs` uses a trusted wrapper contract around the real
`std::sync::Mutex`: `lock` returns a guard carrying an invariant snapshot, and
guard reads/writes are specified against that invariant. The example also marks
construction, lock/guard operations, and thread spawn/join wrappers trusted. It
is a generic synchronization boundary example; it does not verify the standard
mutex implementation or any bytes ownership/refcount law.

`AtomicInvariant::open` is a shared-receiver ghost opening operation, but it
requires explicit namespace `Tokens`. `Atomic` operations instead accept
`FnGhost` callbacks to describe their concrete atomic event. Their `Committer`
contracts remain tied to the actual atomic ward and require its `Perm` for
history updates. `AtView` transports objective payloads subject to view ordering;
it is not a source of affine allocation authority. Resource/authority updates
require mutable resource access.

These are source references for a design review, not new proof results. The
stock contracts and trusted operations remain generic TCB assumptions. This
bundle adds no trusted bytes-specific cloning, refcount, last-owner, or cleanup
law and makes no claim that the generic TCB is adequate for a bytes caller.
No prover was run to create this bundle.

## Recheck hashes

From this directory, run:

```sh
sha256sum -c manifest.sha256
```

`manifest.json` gives the upstream source path and origin revision for each
copied file. The copy paths retain the original subtree layout where practical.
