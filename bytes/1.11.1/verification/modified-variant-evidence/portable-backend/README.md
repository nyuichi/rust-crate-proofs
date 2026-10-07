# Actual portable backend experiment

The isolated current modified bytes library passed `verified,std,extra-platforms`
on x86_64: **263 proof files, zero unresolved leaves, native 37 tests**. This
selects portable-atomic 1.15.0 for the runtime counter and Acquire fence. It is a
normal-return host configuration result, not verification of portable-atomic's
implementation or complete target/configuration coverage.

The positive source is in `../runs/positive/portable-atomic-host-263/evidence.tar.gz`
(SHA256 `350bca93c4b3b5060c6ac8b5b4a9937b9ecd70dd74b1aaa5013eaa7079e15b56`).
The negative source is in
`../runs/negative/portable-atomic-missing-acquire-263/evidence.tar.gz`
(SHA256 `b25d8daf7dca3c5a3f41e8e81d6b34cf12628fe620b9b03168158a50aab31639`).
All 583 positive and 579 negative archive members were independently rehashed.

## Changed source and assumptions

Only `verified/atomic.rs` and its fence import in `verified/retirement.rs` changed
from the recorded source baseline. The native counter now conditionally imports
portable AtomicUsize/Ordering. The portable Acquire-fence adapter has precisely
the stock generic `acq_view@ == *result` contract and calls portable fence(Acquire).
The core configuration continues to reexport the stock fence primitive.

This is an explicit generic primitive TCB mapping: portable-atomic guarantees the
same memory order API semantics. No bytes ownership/refcount/last-owner statement
is trusted. ModelAtomic, Perm, Committer, SyncView, Release RMW publication rule,
all resource capabilities and all retirement protocol bodies/contracts are
unchanged. The primitive implementations are not body-proved. Pinned dependency
source and relevant backend selection files are archived and hashed separately.
On x86, portable's non-SeqCst fence delegates to core's fence; no order is
strengthened and sc-drf is disabled.

## Control and restore

The negative replaces only the selected final `fence_acquire(...)` with the
current thread's view. Translation succeeds, then the full gate exits 1 with
`vc_retire_T: 64/66`: two unresolved leaves in that one file. The proof paths are:

- `proofs/Coma/vc_retire_T/children/0/children/10/children/1/children/0/children/0`
- `proofs/Coma/vc_retire_T/children/0/children/10/children/1/children/1/children/1`

The actual peer recovery invokes `AtView::sync` at retirement.rs:158, requiring
`other.view() <= current`; the current view has no Acquire upgrade from the
pending publication. Original and transformed diagnostic tasks are preserved in
`negative-diagnostic-tasks.tar.gz`. The diagnostic export is not an additional
proof or a complete leaf-to-source mapping. The negative is a missing-Acquire
rejection, not a proof of the generic weak-memory rule's adequacy.

Both changed scratch sources were restored byte-for-byte to the positive archive
after negative capture. Production source and installed Creusot libraries were
not modified by this experiment.

## Reproduce

Extract a positive archive into a disposable directory; use its production-source
and captured wrapper/configuration. Activate the pinned bytes toolchain and run
from the crate directory:

```sh
cargo test --locked --lib --no-default-features --features verified,std,extra-platforms
BYTES_VERIFIED_FEATURES=verified,std,extra-platforms ./verify-all.bash verified
```

The proof wrapper requires elevated execution, one shared lock, one solver and
1024 MiB. To reproduce the control, substitute the negative archived retirement
source and run the same proof command. Actual effective proof features additionally
include cargo-creusot's implicit creusot-std/creusot and nightly flags; both graphs
are captured. MSP alloc-only configurations exclude scoped atomics and are not
portable concurrency evidence. Future platform claims require actual target
UInt/Slice/endian-selected VCs.

## Environment incident

An accidental default-feature sandbox wrapper survived termination of its sandbox
parent, then its Why3 server died and emitted timer.ml assertion failures. The
whole owned orphan tree was stopped and preserved under `environment-failure/`;
no semantic conclusion uses that run. The separately elevated portable-feature
run was held during that capture and then completed successfully. The original
misnamed `wrong-default-never-used` log did run; `invocation-note.txt` corrects
that preliminary assessment. No unrelated processes were terminated.
