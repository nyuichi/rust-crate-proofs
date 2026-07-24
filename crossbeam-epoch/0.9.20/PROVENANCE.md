# crossbeam-epoch 0.9.20 provenance and verification scope

**Verification status: exact exclusive epoch state model (partial).**

This source tree is copied from the `crossbeam-epoch` 0.9.20 package published
on crates.io. The published archive has SHA-256 checksum
`2d6914041f254d6e9176c01941b21115dcfb7089e55135a35411081bd106ef3f`.
Its `.cargo_vcs_info.json` records upstream revision
`239bae00257967a109911b9ebe7c0554d6333501`.

Ordinary Rust builds retain the complete upstream implementation and API. The
Creusot build substitutes an explicit, single-owner collector representation
because the runtime implementation depends on atomic variables, raw and tagged
pointers, thread-local state, deferred closures, and concurrent linked lists
that are not represented by this proof. The projection establishes the core
participant/epoch transition rules; it is not a reclamation-safety,
linearizability, or lock-freedom proof.

## Specification map

The verification `Collector` has a view `(epoch, participant_stamps)`. A stamp
of zero means unpinned; a nonzero stamp is one plus the epoch observed by the
participant when it pinned. Its invariant bounds the epoch and participant
count by `usize`, and requires every stamp to be no newer than the current
epoch. This gives one canonical representation for both pin state and the
participant's observed epoch.

The proved transition graph is:

```text
new/default
  -> register_mut (append an unpinned participant)
  -> pin_mut / unpin_mut (update exactly one stamp)
  -> can_advance (all pinned stamps equal current epoch + 1)
  -> try_advance_mut (advance by exactly one iff the gate holds)
  -> retirement_epoch / is_reclaimable (two-advancement threshold)
```

There are no trusted bodies in this projection. Removing the projection itself
requires connecting the runtime atomic and ownership protocols to this logical
state machine.

## Established contracts

The body proofs establish:

- empty construction at epoch zero and `Default` equivalence;
- exact current-epoch and participant-count observations;
- registration at the logical tail with a stable numeric participant ID;
- exact pin and unpin updates with all other stamps unchanged;
- exact pinned and pinned-in-current-epoch observations;
- an exact universal characterization of the advancement gate;
- unchanged participant state across advancement, with the epoch incremented
  exactly once on success and unchanged on failure;
- capture of the exact epoch attached to a retirement event;
- reclamation eligibility exactly when two advancements have elapsed.

The model deliberately excludes numeric wraparound. `try_advance_mut` therefore
requires room for the next modeled epoch, and `is_reclaimable` requires room to
form the two-epoch threshold.

| Component | Contract reviewed | Body proved | Trusted | Integrated run |
|---|---:|---:|---:|---:|
| construction and observations | yes | yes | no | yes |
| participant registration | yes | yes | no | yes |
| pin/unpin transitions | yes | yes | no | yes |
| advancement gate and transition | yes | yes | no | yes |
| retirement stamp and two-epoch threshold | yes | yes | no | yes |
| concurrent runtime implementation | no | no | excluded | no |

## Explicit exclusions and removal conditions

The runtime `Collector`, `LocalHandle`, and `Guard` bodies; global and local
participant registries; nested guards; atomic compare-exchange and memory
ordering; epoch wraparound arithmetic; tagged `Atomic`, `Owned`, and `Shared`
pointers; allocation and destruction; deferred closure execution; flushing;
thread-local default collection; drops; synchronization queues and lists;
linearization points; reclamation safety; and progress guarantees remain
outside Creusot translation.

Removing these boundaries requires a concurrency logic for atomic permissions
and synchronization histories, a lifetime/ownership model connecting guards to
protected shared pointers, and a proof that deferred objects are unreachable by
every participant before destruction. The runtime tests support the upstream
implementation but are not substitutes for those proofs.

Run `./verify-all.bash` in this directory to reproduce the `no_std + alloc`,
default `std`, and all-features proof matrix. Each configuration reports
`Proved (14 files)`. The ordinary all-features suite passes 40 unit tests and 53
documentation tests, with 5 documentation tests ignored. The loom integration
target contains no tests unless the upstream `crossbeam_loom` cfg is enabled.
Generated Why3 and Cargo build artifacts are intentionally not tracked.
